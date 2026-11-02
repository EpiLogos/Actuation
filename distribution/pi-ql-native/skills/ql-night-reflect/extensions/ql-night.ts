import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { Type } from "typebox";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

// The Night (P′) reflect tools as Actuation's toolset defines them:
// night-tools.json is `actuation-research` reflect.tools, pinned (a Rust test
// holds the two equal), with the relational contract the readings are taken
// under as each tool's prompt guideline. Each reading is native — reflect.read
// puts the MEF register's machinery to the decision provider AIKit holds
// elected — and the model reads the operation's own giving: every lens as its
// profile across its sub-slots. This extension carries no lens, register or
// reading law; it keeps only which lenses this session has read, so the frame
// selector's gate holds across calls, and the gate itself is the native
// operation's.
const supply = JSON.parse(readFileSync(new URL("../night-tools.json", import.meta.url), "utf8"));
if (supply.schema !== "actuation.night-tools/v1" || supply.operation !== "reflect.read"
    || !Array.isArray(supply.tools) || supply.tools.length !== 6) {
  throw new Error("Night tool supply is not Actuation's reflect.tools");
}

async function reflect(request: Record<string, unknown>, cwd: string, signal?: AbortSignal) {
  const executable = process.env.ACTUATION_RESEARCH_BIN;
  if (!executable) throw new Error("the reflect tools need ACTUATION_RESEARCH_BIN (the body's installation binds it)");
  if (signal?.aborted) throw new Error("reflect reading cancelled before launch");
  const input = Buffer.from(JSON.stringify(request), "utf8");
  if (input.length > 1024 * 1024) throw new Error("reflect request exceeds the 1 MiB transport bound");
  const output = await new Promise<Buffer>((resolve, reject) => {
    const child = spawn(executable, ["--json"], { cwd, shell: false, stdio: ["pipe", "pipe", "pipe"] });
    let failure: Error | undefined;
    let bytes = 0;
    const stdout: Buffer[] = [];
    const stop = (error: Error) => {
      if (failure) return;
      failure = error;
      child.kill("SIGTERM");
      setTimeout(() => child.kill("SIGKILL"), 2500).unref();
    };
    const abort = () => stop(new Error("reflect reading cancelled"));
    // The provider bounds its own call (two attempts of up to 120 s).
    const timeout = setTimeout(() => stop(new Error("reflect reading exceeded 300 seconds")), 300000);
    timeout.unref();
    signal?.addEventListener("abort", abort, { once: true });
    child.stdout.on("data", (chunk: Buffer) => {
      bytes += chunk.length;
      if (bytes > 8 * 1024 * 1024) stop(new Error("reflect reading exceeds the 8 MiB transport bound"));
      else stdout.push(chunk);
    });
    child.stderr.resume();
    child.stdin.on("error", (error: Error) => stop(error));
    child.on("error", (error: Error) => { failure ||= error; });
    child.on("close", () => {
      clearTimeout(timeout);
      signal?.removeEventListener("abort", abort);
      if (failure) reject(failure);
      else resolve(Buffer.concat(stdout));
    });
    child.stdin.end(input);
  });
  const result = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(output));
  // A refusal (the coverage gate, an unelected or unavailable provider) is
  // the native operation's own words, returned to the model as a tool error.
  if (result?.schema === "actuation.research-error/v1") throw new Error(result.error);
  if (result?.schema !== "actuation.night-reading/v1" || typeof result.giving !== "string") {
    throw new Error("reflect.read returned an unexpected receipt");
  }
  return result;
}

// A reading is numbered in the order taken, so a whole can name it as a
// member. The count is the session's own (readings already in it), and this
// body's last number keeps parallel readings apart.
const readingsSoFar = (ctx: any) => ctx.sessionManager.getBranch().filter((entry: any) =>
  entry.type === "message" && entry.message?.role === "toolResult" && !entry.message.isError
  && entry.message.details?.night_reading?.schema === "actuation.night-reading/v1").length;

// A body's composition places a thought through L4′ (reflect.read, mode
// place: the T levels). The placement is offered on the session's event bus
// with the levels it reads, so the composing extension needs neither this
// binary nor the register; whichever loads second sets the offer going.
const PLACE = "actuation.night/place";
const PLACE_ASKED = "actuation.night/place-asked";

export default function (pi: ExtensionAPI) {
  if (supply.place?.mode === "place" && Array.isArray(supply.place.levels)) {
    const offer = () => pi.events?.emit(PLACE, {
      levels: supply.place.levels,
      place: (subject: string, cwd: string, signal?: AbortSignal) =>
        reflect({ operation: supply.operation, mode: supply.place.mode, subject }, cwd, signal),
    });
    pi.events?.on(PLACE_ASKED, offer);
    offer();
  }
  // Registration launches nothing; a reading runs only when the model calls.
  const covered = new Set<string>();
  let numbered = 0;
  for (const tool of supply.tools) {
    pi.registerTool({
      name: tool.name, label: tool.label, description: tool.description,
      promptGuidelines: tool.prompt_guidelines,
      parameters: Type.Unsafe<{ subject: string; own_disclosure?: string }>(tool.parameters),
      async execute(_id, args, signal, _update, ctx) {
        const reading = await reflect({ operation: supply.operation, tool: tool.name, subject: args.subject,
          own_disclosure: args.own_disclosure, coverage: [...covered] }, ctx.cwd, signal);
        for (const lens of reading.covers ?? []) covered.add(lens);
        numbered = Math.max(numbered, readingsSoFar(ctx)) + 1;
        return {
          content: [{ type: "text" as const, text: `Reading ${numbered}. ${reading.giving}` }],
          details: { night_reading: reading, reading_number: numbered },
        };
      },
    });
  }
}
