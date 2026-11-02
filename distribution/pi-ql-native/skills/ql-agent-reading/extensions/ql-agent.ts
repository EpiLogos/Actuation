import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { Type } from "typebox";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { registerQlEventContext } from "./ql-event-context.ts";

// Carry the exact QL-owned wire definitions. This extension has no lens,
// relation, operation, completion, modal or musical interpretation tables.
const schemaBytes = readFileSync(new URL("../ql-agent-contracts-v1.schema.json", import.meta.url));
const provenance = JSON.parse(readFileSync(new URL("../provenance.json", import.meta.url), "utf8"));
const schemaHash = createHash("sha256").update(schemaBytes).digest("hex");
if (schemaHash !== provenance.source_sha256["schemas/ql-agent-contracts-v1.schema.json"]) {
  throw new Error("QL tool contract differs from the retained owner source");
}
const definitions = JSON.parse(schemaBytes.toString("utf8")).$defs;
// Some provider tool-schema carriers omit $defs. Inline owner references
// mechanically so the acting model sees the actual object shape. The retained
// contract remains the source; this performs no QL interpretation or repair.
function inlineOwnerSchema(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(inlineOwnerSchema);
  if (!value || typeof value !== "object") return value;
  const record = value as Record<string, unknown>;
  if (typeof record.$ref === "string") {
    const prefix = "#/$defs/";
    const name = record.$ref.startsWith(prefix) ? record.$ref.slice(prefix.length) : "";
    if (!Object.hasOwn(definitions, name)) throw new Error("Unresolved QL owner schema reference");
    const siblings = Object.fromEntries(Object.entries(record).filter(([key]) => key !== "$ref"));
    return inlineOwnerSchema({ ...definitions[name], ...siblings });
  }
  return Object.fromEntries(Object.entries(record).map(([key, item]) => [key, inlineOwnerSchema(item)]));
}
type ProjectionRequest = { event: unknown; requested_heads?: string[] };
const projectionSchema = {
  type: "object",
  additionalProperties: false,
  required: ["event"],
  properties: {
    event: { $ref: "#/$defs/event" },
    requested_heads: {
      type: "array", items: { type: "string", minLength: 1 }, uniqueItems: true,
      description: "Request only relevant semantic heads; the QL owner validates their names and eligibility. Omit for deterministic-only projection.",
    },
  },
};
const projectionParameters = Type.Unsafe<ProjectionRequest>(inlineOwnerSchema(projectionSchema) as Record<string, unknown>);
const admissionParameters = Type.Unsafe<{ projection: ProjectionRequest; response: unknown }>(inlineOwnerSchema({
  type: "object", additionalProperties: false, required: ["projection", "response"],
  properties: { projection: projectionSchema, response: { type: "object" } },
}) as Record<string, unknown>);

type Operation = "project" | "frame" | "harmonic" | "validate" | "invoke" | "decide" | "encounter";
const receiptSchemas: Record<Operation, string> = {
  project: "ql.agent-projection/v1", frame: "ql.agent-decision-frame/v1",
  harmonic: "ql.harmonic-event/v1", validate: "ql.agent-decision-admission/v1",
  invoke: "ql.epi-logos-agent-invocation-result/v1",
  decide: "ql.agent-projection/v1",
  encounter: "ql.agent-encounter/v1",
};

async function nativeReceipt(operation: Operation, request: unknown, cwd: string, signal?: AbortSignal) {
  if (signal?.aborted) throw new Error("QL native invocation cancelled before launch");
  const input = Buffer.from(JSON.stringify(request), "utf8");
  if (input.length > 1024 * 1024) throw new Error("QL request exceeds the 1 MiB transport bound");
  const executable = operation === "encounter" ? process.env.QL_AGENT_ENCOUNTER_BIN || "ql-agent-encounter"
    : operation === "decide" ? process.env.QL_AGENT_DECIDE_BIN || "ql-agent-decide"
    : process.env.QL_AGENT_BIN || "ql-agent";
  const arguments_ = operation === "decide" || operation === "encounter" ? []
    : [operation === "invoke" ? "epi-agent" : "agent-event", operation, "-", "--json"];
  const output = await new Promise<Buffer>((resolve, reject) => {
    const child = spawn(executable, arguments_, {
      cwd, shell: false, stdio: ["pipe", "pipe", "pipe"],
    });
    let failure: Error | undefined;
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let hardStop: ReturnType<typeof setTimeout> | undefined;
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    const stop = (error: Error) => {
      if (failure) return;
      failure = error;
      child.kill("SIGTERM");
      // The shared carrier needs up to one second to reap a separately grouped
      // client after TERM. Preserve that opportunity before the outer KILL.
      hardStop = setTimeout(() => child.kill("SIGKILL"), 2500);
      hardStop.unref();
    };
    const abort = () => stop(new Error("QL native invocation cancelled"));
    const timeoutMs = operation === "decide" ? 190000 : 5000;
    const timeout = setTimeout(() => stop(new Error(`QL native invocation exceeded ${timeoutMs / 1000} seconds`)), timeoutMs);
    timeout.unref();
    signal?.addEventListener("abort", abort, { once: true });
    if (signal?.aborted) abort();
    const clean = () => {
      clearTimeout(timeout);
      if (hardStop) clearTimeout(hardStop);
      signal?.removeEventListener("abort", abort);
    };
    child.stdout.on("data", (bytes: Buffer) => {
      stdoutBytes += bytes.length;
      if (stdoutBytes > 8 * 1024 * 1024) stop(new Error("QL receipt exceeds the 8 MiB transport bound"));
      else stdout.push(bytes);
    });
    child.stderr.on("data", (bytes: Buffer) => {
      stderrBytes += bytes.length;
      if (stderrBytes > 256 * 1024) stop(new Error("QL diagnostic exceeds the transport bound"));
      else stderr.push(bytes);
    });
    child.stdin.on("error", (error: Error) => stop(error));
    child.on("error", (error: Error) => {
      failure ||= error;
    });
    child.on("close", (code, exitSignal) => {
      clean();
      if (failure) reject(failure);
      else if (code !== 0) reject(new Error(`QL native ${operation} refused (exit ${code}, signal ${exitSignal}): ${Buffer.concat(stderr).toString("utf8").trim()}`));
      else resolve(Buffer.concat(stdout));
    });
    child.stdin.end(input);
  });
  const text = new TextDecoder("utf-8", { fatal: true }).decode(output);
  const receipt = JSON.parse(text);
  if (!receipt || Array.isArray(receipt) || receipt.schema !== receiptSchemas[operation]) {
    throw new Error("QL executable returned an unexpected native receipt schema");
  }
  if (operation === "encounter" && (receipt.provider_calls !== 0
      || receipt.projector_revision !== "sha256:" + provenance.source_sha256["scripts/ql_agent_encounter.py"])) {
    throw new Error("QL encounter differs from its installed deterministic owner provenance");
  }
  if (operation !== "invoke" && (operation !== "encounter" || receipt.disposition === "projected")) {
    const basis = receipt.kernel_basis || receipt.frame?.kernel_basis || receipt.projection?.frame?.kernel_basis;
    if (basis?.digest !== provenance.kernel_digest) {
      throw new Error("QL native receipt differs from the installed kernel provenance");
    }
  }
  return {
    content: [{ type: "text" as const, text }],
    details: { native_receipt: receipt, executable, operation, input_bytes: input.length, output_bytes: output.length },
  };
}

// Composition. A body whose QL constitution holds `vak.compose` has it as a
// tool of its own, named for the operation the constitution names. A member
// of a whole is a thought standing at a T level its reading holds: given in
// words, it is placed in the call by the body's Night placement (L4′ alone);
// given as a reading the body took, that reading's holds stand. The T levels
// are the L4′ sub-slots, and a reading's holds are the only coordinates
// admitted. The tool is offered once the Night placement is, with the levels
// it reads, so this extension carries no T table. The kernel evaluates each request
// in memory, so the session's earlier composition is replayed before the new
// steps; grain, recognition, relation and frame are all the kernel's.
const COMPOSE = "vak_compose";
const NIGHT_READING = "actuation.night-reading/v1";
// The Night extension's placement offer on the session's event bus.
const PLACE = "actuation.night/place";
const PLACE_ASKED = "actuation.night/place-asked";
type Placement = {
  levels: { t: number; label: string; meaning: string }[];
  place: (subject: string, cwd: string, signal?: AbortSignal) => Promise<any>;
};
const FRAME = { basis: "chromatic", face: "direct", id: "CF2", lens: "L4'", positions: "local" };
type Agent = { name?: string; position?: string; operations?: string[] };
type ComposeStep = {
  op: "whole" | "recognise" | "reopen" | "read"; name: string; route?: string;
  members?: { reading?: number; thought?: string; t: number }[];
  against?: { thought: number; against: number; why: string }[];
};
type Held = { number: number; id: string; subject: string; t: Map<number, string> };
type Named = { whole: string; current: string; recognised: boolean; members: { t: number; label: string; reading?: number; subject: string }[]; evidence: string[] };

const composeParameters = Type.Object({
  steps: Type.Array(Type.Object({
    op: Type.Unsafe<ComposeStep["op"]>({ type: "string", enum: ["whole", "recognise", "reopen", "read"],
      description: "whole: gather thoughts into a whole. recognise: carry a closed whole as its 0/1. reopen: open a recognised whole to its exact basis. read: its frame pitch and intervals." }),
    name: Type.String({ pattern: "^[A-Za-z0-9_-]{1,48}$", description: "The whole's name, given when it is made and used to recognise, reopen or read it." }),
    members: Type.Optional(Type.Array(Type.Object({
      thought: Type.Optional(Type.String({ minLength: 1, maxLength: 600, description: "The thought in words; QL reads where it can stand." })),
      reading: Type.Optional(Type.Integer({ minimum: 1, description: "Or a thought you have read, by its reading's number." })),
      t: Type.Integer({ minimum: 0, maximum: 5, description: "The T level you see it standing at." }),
    }, { additionalProperties: false }), { minItems: 1, maxItems: 6, description: "For a whole: each thought once, in words or as a reading, each T level once." })),
    against: Type.Optional(Type.Array(Type.Object({
      thought: Type.Integer({ minimum: 0, maximum: 5, description: "The T level of the thought that is set against another." }),
      against: Type.Integer({ minimum: 0, maximum: 5, description: "The T level of the thought it is set against." }),
      why: Type.String({ minLength: 1, maxLength: 400 }),
    }, { additionalProperties: false }), { maxItems: 15, description: "For a whole: which member is set against which, and why. The kernel reads each pair." })),
    route: Type.Optional(Type.Unsafe<string>({ type: "string", enum: ["4+2", "3+3", "pairs-A", "pairs-B", "pairs-C"],
      description: "For recognising a sixfold: the reading of six (4+2 when left out)." })),
  }, { additionalProperties: false }), { minItems: 1, maxItems: 8 }),
}, { additionalProperties: false });

const toolResults = (ctx: any) => ctx.sessionManager.getBranch()
  .filter((entry: any) => entry.type === "message" && entry.message?.role === "toolResult" && !entry.message.isError)
  .map((entry: any) => entry.message);

// The readings this conversation holds, by number, with the T levels each
// holds and the words it read.
function heldReadings(ctx: any): Held[] {
  const asked = new Map<string, string>();
  for (const entry of ctx.sessionManager.getBranch()) {
    if (entry.type !== "message" || entry.message?.role !== "assistant") continue;
    for (const part of entry.message.content ?? []) {
      if (part?.type === "toolCall" && typeof part.arguments?.subject === "string") asked.set(part.id, part.arguments.subject);
    }
  }
  const held: Held[] = [];
  for (const message of toolResults(ctx)) {
    const night = message.details?.night_reading;
    if (night?.schema !== NIGHT_READING) continue;
    held.push({ number: message.details.reading_number ?? held.length + 1, id: message.toolCallId, subject: asked.get(message.toolCallId) ?? "", t: standing(night) ?? new Map() });
  }
  return held;
}

// Where a read thought can stand in a whole: the L4′ sub-slots its reading
// holds (L4.n′ is Tn), by label; none when the reading did not read L4′.
function standing(night: any): Map<number, string> | undefined {
  const reading = night.reading ?? {};
  const profiles = [...(reading.lenses ?? []), ...(reading.positions ?? []).flatMap((p: any) => p.lenses ?? [])];
  const l4 = profiles.find((p: any) => p.lens === "L4'");
  if (!l4) return undefined;
  const t = new Map<number, string>();
  for (const ref of l4?.holds ?? []) {
    const slot = l4.slots.find((s: any) => s.ref === ref);
    const level = /^L4\.([0-5])'$/.exec(slot?.slot ?? "");
    if (level) t.set(Number(level[1]), slot.label);
  }
  return t;
}

const excerpt = (text: string) => (text.length > 90 ? `${text.slice(0, 87)}…` : text);
const levels = (t: Map<number, string>) => (t.size ? [...t].sort(([a], [b]) => a - b).map(([n, label]) => `T${n} ${label}`).join(", ") : "no T level (it did not hold at L4′)");

function registerComposition(pi: ExtensionAPI, agent: Agent, offer: Placement) {
  const caller = `agent/${String(agent.name).toLowerCase()}`;
  const tLevels = offer.levels.map((l) => `T${l.t} ${l.label} (${l.meaning})`).join(", ");
  // One placement per thought per session: the same words read again give the same profile.
  const placings = new Map<string, Promise<any>>();
  pi.registerTool({
    name: COMPOSE, label: "Compose thoughts into wholes",
    description: `Whether thoughts hold together, and as what whole. Give each thought in words at the T level you see it standing — ${tLevels} — and QL reads it through L4′, refusing a level it does not hold and saying where it can stand; a thought you have already read can be given by its reading's number instead. QL names the whole the members make — two, 123, 450, 1234, 4+1, the sixfold — or says they make none, and reads each pair you set against each other. Recognise a whole to carry it as one 0/1 that keeps its basis; reopen it when the talk returns to it; read it for its frame pitch and intervals.`,
    promptGuidelines: ["vak_compose answers whether thoughts hold together and how each pair stands; compose when that would change what you say. A recognised whole travels as its 0/1 and reopens to its exact basis."],
    parameters: composeParameters,
    async execute(_id, args, signal, _update, ctx) {
      const readings = heldReadings(ctx);
      const placements: { thought: string; reading: unknown }[] = [];
      const place = (words: string) => {
        const key = `${ctx.sessionManager.getSessionId()}\n${words}`;
        let placing = placings.get(key);
        if (!placing) {
          placing = offer.place(words, ctx.cwd, signal);
          placings.set(key, placing);
          placing.catch(() => placings.delete(key));
        }
        return placing;
      };
      const earlier = toolResults(ctx).filter((m: any) => m.toolName === COMPOSE && m.details?.request).at(-1)?.details;
      const prior: unknown[] = earlier?.request?.input?.steps ?? [];
      const named: Record<string, Named> = structuredClone(earlier?.composition ?? {});
      const taken = new Set(prior.map((s: any) => s.into ?? s.useRef));
      const fresh = (base: string) => { let ref = base; for (let k = 2; taken.has(ref); k += 1) ref = `${base}~${k}`; taken.add(ref); return ref; };
      const ground = `prime-session:${ctx.sessionManager.getSessionId()}`;
      const basis = (evidence: string[]) => ({ caller, evidence, revision: ctx.sessionManager.getLeafId() ?? "root", source: ground, standing: "PROPOSED" });
      const steps: any[] = [];
      for (const step of args.steps as ComposeStep[]) {
        const whole = named[step.name];
        if (step.op === "whole") {
          if (whole) throw new Error(`there is already a whole named ${step.name}; recognise, reopen or read it, or name the new one otherwise`);
          const members = step.members ?? [];
          if (!members.length) throw new Error(`whole ${step.name} needs its members: each a thought in words or a reading by number, at a T level`);
          // Every member is placed before any refusal, so one answer names every misplaced thought.
          const settled = await Promise.allSettled(members.map(async ({ reading, thought, t }, i) => {
            if ((thought === undefined) === (reading === undefined)) {
              throw new Error(`member ${i + 1} of ${step.name}: give the thought in words or a reading's number, one of the two`);
            }
            if (reading !== undefined) {
              const held = readings.find((r) => r.number === reading);
              if (!held) throw new Error(`there is no reading ${reading}; the readings so far are ${readings.map((r) => r.number).join(", ") || "none"}`);
              if (!held.t.has(t)) throw new Error(`reading ${reading} does not hold T${t}; it holds ${levels(held.t)}`);
              return { t, label: held.t.get(t)!, reading, subject: held.subject, ref: `reading:${held.id}` };
            }
            const words = thought!.trim();
            const night = await place(words);
            const at = standing(night) ?? new Map<number, string>();
            if (!at.size) throw new Error(`“${excerpt(words)}” holds no T level, so it cannot stand in a whole`);
            if (!at.has(t)) throw new Error(`“${excerpt(words)}” does not hold T${t}; it can stand at ${levels(at)}`);
            placements.push({ thought: words, reading: night.reading });
            return { t, label: at.get(t)!, subject: words, ref: `thought:${createHash("sha256").update(words).digest("hex").slice(0, 16)}` };
          }));
          const refused = settled.filter((r): r is PromiseRejectedResult => r.status === "rejected");
          if (refused.length) throw new Error(refused.map((r) => String(r.reason?.message ?? r.reason)).join("; "));
          const placed = settled.map((r) => (r as PromiseFulfilledResult<{ t: number; label: string; reading?: number; subject: string; ref: string }>).value);
          const seen = new Set<string>();
          for (const m of placed) {
            if (seen.has(m.ref)) throw new Error(`${m.reading ? `reading ${m.reading}` : `“${excerpt(m.subject)}”`} is already in ${step.name}: a thought stands at one T level in a whole`);
            seen.add(m.ref);
          }
          const at = new Set(placed.map((m) => m.t));
          for (const pair of step.against ?? []) {
            if (!at.has(pair.thought) || !at.has(pair.against)) throw new Error(`in ${step.name}, T${pair.thought} set against T${pair.against} names a level no member stands at`);
          }
          const evidence = placed.map((m) => m.ref);
          const useRef = fresh(step.name);
          steps.push({
            op: "whole", basis: basis(evidence), category: "T", frame: FRAME, groundFace: "direct", groundRef: ground,
            members: placed.map((m) => ({ face: "direct", position: m.t, subjectRef: m.ref })),
            ...(step.against?.length ? { relations: step.against.map((pair) => ({
              row: { face: "direct", position: pair.thought }, column: { face: "direct", position: pair.against },
              relationRef: "epii:set-against", evidence: [pair.why] })) } : {}),
            subjectRef: `${ground}#${step.name}`, useRef, wholeRef: `anchor:${useRef}`,
          });
          named[step.name] = { whole: useRef, current: useRef, recognised: false, evidence,
            members: placed.map(({ t, label, reading, subject }) => ({ t, label, reading, subject })) };
          continue;
        }
        if (!whole) throw new Error(`there is no whole named ${step.name}; the wholes so far are ${Object.keys(named).join(", ") || "none"}`);
        if (step.op === "recognise") {
          if (whole.recognised) throw new Error(`${step.name} is already recognised as its 0/1; reopen it first`);
          const into = fresh(`${whole.whole}.0/1`);
          steps.push({ op: "recognise", basis: basis(whole.evidence), from: whole.current, into, ...(step.route ? { route: step.route } : {}) });
          Object.assign(whole, { current: into, recognised: true });
        } else if (step.op === "reopen") {
          if (!whole.recognised) throw new Error(`${step.name} is open already; only a recognised whole is reopened`);
          const into = fresh(`${whole.whole}.open`);
          steps.push({ op: "reopen", basis: basis(whole.evidence), from: whole.current, into });
          Object.assign(whole, { current: into, recognised: false });
        } else {
          steps.push({ op: "read", lens: "L4'", useRef: whole.current });
        }
      }
      const request = { schema: "ql.epi-logos-agent-invocation/v1", position: agent.position,
        operation: "vak.compose", input: { contract: "ql.vak-composition/v1", steps: [...prior, ...steps] } };
      let receipt;
      try {
        receipt = await nativeReceipt("invoke", request, ctx.cwd, signal);
      } catch (error) {
        // The kernel counts the replayed steps too; name the step as the caller sent it.
        throw new Error(String((error as Error).message).replace(/step (\d+)/g, (_, n) =>
          Number(n) < prior.length ? "an earlier step" : `step ${Number(n) - prior.length + 1}`));
      }
      const results = receipt.details.native_receipt.result.results.slice(prior.length);
      return {
        content: [{ type: "text" as const, text: results.map((r: any, i: number) => giving(args.steps[i], r, named)).join("\n") }],
        details: { ...receipt.details, request, replayed: prior.length, composition: named, placements },
      };
    },
  });
  // A reading says where its thought can stand in a whole, so a whole can be
  // made from it without decoding the L4′ block.
  pi.on("tool_result", (event: any) => {
    const night = event.details?.night_reading;
    if (event.isError || night?.schema !== NIGHT_READING) return;
    const t = standing(night);
    const number = event.details.reading_number;
    const line = !t ? `Reading ${number} did not read L4′, so it gives its thought no place in a whole.`
      : t.size ? `In a whole, reading ${number} can stand at ${levels(t)}.` : `Reading ${number} holds no T level, so it cannot stand in a whole.`;
    return { content: [...event.content, { type: "text" as const, text: line }] };
  });
}

// What the kernel returned for one step, as the model reads it.
function giving(step: ComposeStep, { op, result }: any, named: Record<string, Named>): string {
  const whole = named[step.name];
  const member = (m: Named["members"][number]) => `T${m.t} ${m.label}: ${m.reading
    ? `reading ${m.reading}${m.subject ? ` (“${excerpt(m.subject)}”)` : ""}` : `“${excerpt(m.subject)}”`}`;
  const operator = (ref: string) => String(ref).replace(/^ql:shape:[0-9.]+:/, "").replace(/^compression:/, "").replace(/^recognition:/, "the reading of ");
  if (op === "whole") {
    const grain = result.body?.grain;
    const lines = [`Whole ${step.name}: ${grain === "other" ? "not one of QL's wholes (grain other); its recognisable parts can be made wholes of their own" : grain}.`,
      ...whole.members.map((m) => `  ${member(m)}`)];
    for (const relation of result.binding?.relations ?? []) {
      const fields = (relation.kernel?.fields ?? []).map((f: any) => `${f.family} pair ${f.pairIndex}${f.reversed ? ", reversed" : ""}`);
      const kinds = (relation.kernel?.relations ?? []).map((k: string) => k.replace(/^ql\.kernel\./, "").replace(/\/v1$/, ""));
      lines.push(`  T${relation.row.position} set against T${relation.column.position}: the kernel reads ${[...fields, ...kinds.filter((k: string) => !k.startsWith("pair."))].join("; ") || "no relation it names"}.`);
    }
    return lines.join("\n");
  }
  if (op === "recognise") return `Recognised ${step.name} as its 0/1 by ${operator(result.binding?.operatorRef)}; it keeps its basis and reopens to it exactly.`;
  if (op === "reopen") return [`Reopened ${step.name} to its basis:`, ...whole.members.map((m) => `  ${member(m)}`)].join("\n");
  const intervals = (result.childIntervals ?? []).length ? `; the members' intervals ${result.childIntervals.join(", ")}` : "";
  return `Read ${step.name}: frame pitch ${result.framePitch}, phase ${result.geometry?.framePhaseDegrees}°${intervals}.`;
}

export default function (pi: ExtensionAPI, body: "pi" | "prime" = "pi", agent?: Agent) {
  registerQlEventContext(pi, body, (request, cwd, signal) => nativeReceipt("encounter", request, cwd, signal));
  // Registration performs no CLI, provider or model call. These are explicit
  // native tools, so ordinary body entry and tool execution do not wait for ML.
  const readings = [
    ["ql_project_event", "Project QL event", "Project the exact sourced event, complete deterministic QL state and retain unresolved semantic heads.", "project"],
    ["ql_decision_frame", "Inspect QL decision frame", "Read the kernel-owned remaining heads, legal candidates, ambiguity policies and constraints.", "frame"],
    ["ql_harmonic_read", "Read QL harmonics", "Compute the musical rendering of the formal event with the actual QL owner.", "harmonic"],
    ["ql_decide", "Determine QL event", "Complete deterministic QL first, request only remaining semantic heads through an explicitly elected AIKit provider, and admit the answer with the native kernel. An unavailable provider remains unresolved. This explicit tool may take time; it never blocks body entry.", "decide"],
  ] as const;
  for (const [name, label, description, operation] of readings) {
    pi.registerTool({ name, label, description, parameters: projectionParameters,
      async execute(_id, request, signal, _update, ctx) {
        return nativeReceipt(operation, request, ctx.cwd, signal);
      },
    });
  }
  pi.registerTool({
    name: "ql_validate_determination", label: "Validate QL determination",
    description: "Admit an exact provider response against the original QL event and kernel; retain refusal, ambiguity, unavailability and stale results.",
    parameters: admissionParameters,
    async execute(_id, request, signal, _update, ctx) {
      return nativeReceipt("validate", request, ctx.cwd, signal);
    },
  });
  pi.registerTool({
    name: "ql_invoke", label: "Invoke native QL operation",
    description: "Invoke an exact current QL faculty operation with its required input. The native owner checks operation membership and inputs; inspect its receipts and unresolved requirements first.",
    parameters: Type.Object({
      schema: Type.Literal("ql.epi-logos-agent-invocation/v1"),
      position: Type.String({ pattern: "^#[0-5]$" }),
      operation: Type.String({ minLength: 1 }),
      input: Type.Record(Type.String(), Type.Unknown()),
    }, { additionalProperties: false }),
    async execute(_id, request, signal, _update, ctx) {
      return nativeReceipt("invoke", request, ctx.cwd, signal);
    },
  });
  // Composition is offered once the body's Night placement is: whichever extension loads second sets it going.
  if (agent?.operations?.includes("vak.compose") && typeof agent.position === "string" && agent.name) {
    let offered = false;
    pi.events?.on(PLACE, (offer: any) => {
      if (offered || typeof offer?.place !== "function" || !Array.isArray(offer.levels)) return;
      offered = true;
      registerComposition(pi, agent, offer);
    });
    pi.events?.emit(PLACE_ASKED, undefined);
  }
}
