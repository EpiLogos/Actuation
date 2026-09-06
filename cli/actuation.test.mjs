import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { executeCommand, matchRoute } from "./actuation.mjs";
import { COMMANDS, discoverTestFiles } from "./commands.mjs";
import { ACTUATION_CLI_VERSION, ACTUATION_CLI_SURFACE } from "./surface.mjs";
import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { REALISED_ACTUATION_VERSION } from "../contracts/realised-actuation.mjs";
import { ACTUATION_STREAM_VERSION, appendActuationStreamEvent } from "../contracts/actuation-stream.mjs";
import { activityFromActuationStream } from "../contracts/activity.mjs";
import { ACTUATION_INSTANTIATION_VERSION } from "../contracts/instantiation.mjs";
import { CATALOG_REVISION } from "../detection/catalog.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const agency = {
  binding: {
    schema: AGENCY_CONTRACT_VERSION,
    binding_ref: "world-binding:test",
    agent_ref: "agent:test",
    agency_ref: "agency:test",
    world_ref: "world:test",
    scope_ref: "scope:test",
  },
  root_scope: {
    schema: AGENCY_CONTRACT_VERSION,
    scope_ref: "scope:test",
    enclosing_world_ref: "world:test",
  },
};

const realised = {
  schema: REALISED_ACTUATION_VERSION,
  realised_ref: "realised/agent-loop/demo/1",
  actuation_ref: "actuation/demo",
  agent_ref: "agent/demo",
  agency_ref: "agency/demo/project-coding",
  world_binding_ref: "world/demo-project",
  loop: {
    recurrence: "turn-based",
    acting: true,
    entrypoint_ref: "native:cli/invoke",
    observed_faculties: ["model-call", "filesystem", "shell"],
  },
  body: {
    model_condition_ref: "actuation:model-bearing/demo",
  },
  observation: {
    state: "observed",
    evidence_refs: ["evidence/native-invocation/1"],
  },
};

function baseStream() {
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: "stream:dev-1",
    actuation_ref: "actuation:dev-1",
    agency_ref: "agency:development",
    agent_session_ref: "agent-session:development",
    world_binding_ref: "world:project:oi:development",
    participating_loci: ["locus:builder"],
    surface_refs: ["surface:desktop"],
    lifecycle: { state: "open", started_at: "2026-08-31T10:00:00Z" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
    provenance: ["O:I #155 fixture"],
  };
}

function activityFixture() {
  let stream = appendActuationStreamEvent(baseStream(), {
    event_ref: "event:1",
    sequence: 1,
    kind: "tool-request",
    observed_at: "2026-08-31T10:00:01Z",
    actor: { locus_ref: "locus:builder", agency_ref: "agency:development", agent_ref: "agent:builder" },
    disclosure: "portable",
    native_trace_ref: "trace:tool/1",
    resource_refs: ["action:factory.apply"],
  });
  stream = appendActuationStreamEvent(stream, {
    event_ref: "event:2",
    sequence: 2,
    kind: "tool-result",
    observed_at: "2026-08-31T10:00:02Z",
    actor: { locus_ref: "locus:builder", agency_ref: "agency:development", agent_ref: "agent:builder" },
    disclosure: "portable",
    native_trace_ref: "trace:tool/1",
    resource_refs: ["result:factory.apply:1"],
  });
  return activityFromActuationStream(stream, {
    activityRef: "activity:cli-test-1",
    subjectRef: "journey:cli-test",
    nativeOwner: "actuation",
    actionRef: "action:factory.apply",
    invocationRef: "invocation:factory.apply:1",
    resultRef: "result:factory.apply:1",
    verb: "implemented",
    object: "cli surface",
    summary: "CLI fixture completed a bounded act and returned evidence.",
    evidenceRefs: ["evidence:test-green"],
  });
}

function instantiationReceiptFixture() {
  const schema = ACTUATION_INSTANTIATION_VERSION;
  return {
    schema,
    actuation_ref: "actuation:cli-test-run-1",
    agency_ref: "agency:cli-test",
    world_binding_ref: "world-binding:cli-test",
    model_relation: {
      schema,
      model_ref: "model:fixture",
      material: { placement: "local", facts: { accelerator: "metal" } },
      inference_surface: { contract_ref: "contract:openai-compatible-chat/v1" },
    },
    access_profile: {
      schema,
      inference: { allowed: ["invoke"], denied: [] },
      control: { allowed: [], denied: ["acquire"] },
      interior: { depth: "behavioral", allowed: [], denied: [] },
    },
    observed_at: "2026-09-06T00:00:00Z",
  };
}

test("version and capabilities expose one maintained native surface", () => {
  assert.equal(executeCommand(["--version"]).stdout, `actuation ${ACTUATION_CLI_VERSION}`);
  const value = JSON.parse(executeCommand(["capabilities", "--json"]).stdout);
  assert.equal(value.contract, "actuation.cli/v1");
  assert.equal(value.executable, "actuation");
  assert.equal(value.version, ACTUATION_CLI_VERSION);
  assert.deepEqual(value.commands, COMMANDS.map((entry) => entry.name));
  assert.ok(value.commands.includes("harness.self"));
  assert.equal(value.native_contracts.agency, AGENCY_CONTRACT_VERSION);
  assert.ok(value.revision === "unknown" || /^[0-9a-f]{7,40}$/.test(value.revision));
});

test("help documents exactly the routes the command table declares", () => {
  const help = executeCommand(["help"]).stdout;
  const routes = [];
  for (const line of help.split("\n")) {
    if (!line.startsWith("  actuation ")) continue;
    const cleaned = line
      .replace(/\[[^\]]*\]/g, "")
      .replace(/<[^>]*>/g, "")
      .replace(/\s+/g, " ")
      .trim();
    const words = cleaned.split(" ").slice(1);
    if (words[0]?.startsWith("--")) continue;
    routes.push(words.join(" "));
  }
  assert.deepEqual(
    new Set(routes),
    new Set(COMMANDS.map((entry) => entry.route.join(" "))),
    "help usage lines and the command table have diverged",
  );
});

test("every declared command route matches the dispatcher", () => {
  for (const entry of COMMANDS) {
    assert.equal(matchRoute([...entry.route])?.entry.name, entry.name, `${entry.name} does not match its own route`);
  }
  // Longest route wins: "instantiation record" must not fall through to read.
  assert.equal(matchRoute(["instantiation", "record", "-", "--json"]).entry.name, "instantiation.record");
  assert.equal(matchRoute(["instantiation", "-", "--json"]).entry.name, "instantiation.read");
});

test("bare harness names its subcommands instead of a generic unknown", () => {
  assert.throws(() => executeCommand(["harness"]), /expected catalog, detect or self/);
  assert.throws(() => executeCommand(["harness", "teleport"]), /expected catalog, detect or self/);
});

test("invalid command fails rather than fabricating a fallback", () => {
  assert.throws(() => executeCommand(["unknown"]), /unknown command/);
});

test("contract list names every native contract", () => {
  const result = executeCommand(["contract", "list", "--json"]);
  assert.deepEqual(JSON.parse(result.stdout), ACTUATION_CLI_SURFACE.native_contracts);
  assert.match(executeCommand(["contract", "list"]).stdout, /harness_detection\tactuation\.harness-detection\/v1/);
});

test("agency command enters the native Agency read model", () => {
  const result = executeCommand(["agency", "-", "--json"], { stdin: JSON.stringify(agency) });
  const value = JSON.parse(result.stdout);
  assert.equal(value.agency_ref, "agency:test");
  assert.equal(value.agent_ref, "agent:test");
  assert.equal(value.root_for_scope, true);
});

test("realised command enters the native realised-actuation read model", () => {
  const result = executeCommand(["realised", "-", "--json"], { stdin: JSON.stringify(realised) });
  const value = JSON.parse(result.stdout);
  assert.equal(value.realised_ref, "realised/agent-loop/demo/1");
  assert.match(executeCommand(["realised", "-"], { stdin: JSON.stringify(realised) }).stdout, /Observation: observed/);
});

test("stream command enters the native ActuationStream read model", () => {
  const stream = baseStream();
  const result = executeCommand(["stream", "-", "--json"], { stdin: JSON.stringify(stream) });
  const value = JSON.parse(result.stdout);
  assert.equal(value.stream_ref, "stream:dev-1");
  assert.equal(value.lifecycle.state, "open");
  assert.deepEqual(value.events, []);
});

test("activity command validates and projects the Activity contract", () => {
  const result = executeCommand(["activity", "-", "--json"], { stdin: JSON.stringify(activityFixture()) });
  const value = JSON.parse(result.stdout);
  assert.equal(value.activity_ref, "activity:cli-test-1");
  assert.match(executeCommand(["activity", "-"], { stdin: JSON.stringify(activityFixture()) }).stdout, /Activity activity:cli-test-1/);
});

test("instantiation read validates receipts and reads legacy model-bearing documents", () => {
  const receipt = instantiationReceiptFixture();
  const result = executeCommand(["instantiation", "-", "--json"], { stdin: JSON.stringify(receipt) });
  assert.equal(JSON.parse(result.stdout).actuation_ref, "actuation:cli-test-run-1");
  const legacy = structuredClone(receipt);
  legacy.schema = "actuation.model-bearing/v1";
  legacy.model_relation.schema = "actuation.model-bearing/v1";
  legacy.access_profile.schema = "actuation.model-bearing/v1";
  const normalised = JSON.parse(executeCommand(["instantiation", "-", "--json"], { stdin: JSON.stringify(legacy) }).stdout);
  assert.equal(normalised.schema, ACTUATION_INSTANTIATION_VERSION);
});

test("instantiation record refuses unattributed receipts without the flag", () => {
  assert.throws(
    () => executeCommand(["instantiation", "record", "-"], { stdin: JSON.stringify(instantiationReceiptFixture()) }),
    /--allow-unattributed/,
  );
});

test("instantiation record persists bound receipts as JSONL via --out", () => {
  const dir = mkdtempSync(join(tmpdir(), "actuation-record-"));
  const out = join(dir, "receipts.jsonl");
  try {
    const human = executeCommand(
      ["instantiation", "record", "--allow-unattributed", "--out", out, "-"],
      { stdin: JSON.stringify(instantiationReceiptFixture()) },
    );
    assert.match(human.stdout, /unattributed instantiation/);
    assert.match(human.stdout, new RegExp(out.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
    const lines = readFileSync(out, "utf8").trim().split("\n");
    assert.equal(lines.length, 1);
    assert.equal(JSON.parse(lines[0]).unattributed, true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("harness catalog declares the live catalog", () => {
  const value = JSON.parse(executeCommand(["harness", "catalog", "--json"]).stdout);
  assert.equal(value.document, "catalog");
  assert.equal(value.catalog_revision, CATALOG_REVISION);
  assert.ok(value.descriptors.some((descriptor) => descriptor.slug === "zcode"));
  const human = executeCommand(["harness", "catalog"]).stdout;
  assert.match(human, new RegExp(`Harness catalog \\(r${CATALOG_REVISION}, \\d+ declared\\)`));
  assert.match(human, /zcode/);
});

test("harness detect --only narrows the run and rejects unknown slugs", () => {
  const result = executeCommand(["harness", "detect", "--only", "zcode"]);
  const harnessLines = result.stdout.split("\n").filter((line) => line.startsWith("  "));
  assert.equal(harnessLines.length, 1);
  assert.match(harnessLines[0], /zcode/);
  assert.throws(
    () => executeCommand(["harness", "detect", "--only", "nope"]),
    /unknown harness slug\(s\): nope/,
  );
});

test("harness detect runs the whole catalog by default", () => {
  const result = executeCommand(["harness", "detect"]);
  assert.match(result.stdout, new RegExp(`Harness detection \\(catalog r${CATALOG_REVISION}, (complete|partial)\\)`));
  const harnessLines = result.stdout.split("\n").filter((line) => line.startsWith("  "));
  assert.equal(harnessLines.length, catalogSize());
});

function catalogSize() {
  return JSON.parse(executeCommand(["harness", "catalog", "--json"]).stdout).descriptors.length;
}

test("harness self reports identity with the same-run cross-check", () => {
  const result = executeCommand(["harness", "self"]);
  assert.match(result.stdout, new RegExp(`Harness self \\(catalog r${CATALOG_REVISION}`));
  const value = JSON.parse(executeCommand(["harness", "self", "--json"]).stdout);
  assert.equal(value.document, "self");
  assert.equal(typeof value.ambiguity, "boolean");
});

test("README documents every command the table declares", () => {
  const readme = readFileSync(join(REPO_ROOT, "README.md"), "utf8");
  for (const entry of COMMANDS) {
    assert.ok(readme.includes(entry.usage), `${entry.usage} missing from README`);
  }
});

test("verify discovers every native suite instead of a hand-written list", () => {  const discovered = discoverTestFiles();
  for (const name of ["contracts/harness-detection.test.mjs", "detection/self.test.mjs", "detection/detect.test.mjs", "cli/actuation.test.mjs"]) {
    assert.ok(discovered.includes(name), `${name} missing from discovered suites`);
  }
  // Independent oracle: every *.test.mjs in the repo outside ignored trees
  // must be inside the discovered set, so a new suite can never silently
  // fall outside the verification gate.
  const present = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === ".git" || entry.name === "node_modules" || entry.name === "experiments") continue;
      const path = join(dir, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (entry.name.endsWith(".test.mjs")) present.push(path);
    }
  };
  walk(REPO_ROOT);
  const relative = present.map((path) => path.slice(REPO_ROOT.length + 1)).sort();
  assert.deepEqual(relative, [...discovered].sort(), "a test suite exists that verify() does not discover");
});
