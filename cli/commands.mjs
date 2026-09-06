// The single source of truth for the Actuation command surface. Each entry
// carries its route, usage line and handler; help text, the capabilities
// listing and dispatch are all derived from this table, so a command cannot
// exist in one representation and be missing from another. Adding a command
// is one entry here — nothing else.
import { readFileSync, readdirSync, appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { ACTUATION_CLI_SURFACE } from "./surface.mjs";
import { agencyReadModel } from "../contracts/agency.mjs";
import { realisedActuationReadModel } from "../contracts/realised-actuation.mjs";
import { actuationStreamReadModel } from "../contracts/actuation-stream.mjs";
import { validateActivity } from "../contracts/activity.mjs";
import { instantiationReceipt, attachDetectionEvidence } from "../contracts/instantiation.mjs";
import { harnessCatalog as harnessCatalogDocument } from "../contracts/harness-detection.mjs";
import { runDetection } from "../detection/detect.mjs";
import { resolveSelf } from "../detection/self.mjs";
import { harnessDescriptors, CATALOG_REVISION } from "../detection/catalog.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// Where native test suites live; verify() discovers them at runtime so a new
// suite can never be silently outside the verification gate.
const TEST_DIRS = ["contracts", "detection", "cli"];

function readJsonInput(path, stdin) {
  const text = path === "-" ? stdin : readFileSync(path, "utf8");
  if (typeof text !== "string" || text.trim() === "") {
    throw new TypeError(`no JSON input supplied for ${path}`);
  }
  return JSON.parse(text);
}

function output(value, json, humanRenderer) {
  return {
    code: 0,
    stdout: json ? JSON.stringify(value, null, 2) : humanRenderer(value),
  };
}

function removeFlag(args, flag) {
  const index = args.indexOf(flag);
  if (index === -1) return false;
  args.splice(index, 1);
  return true;
}

function flagValue(args, flag) {
  const index = args.indexOf(flag);
  if (index === -1) return undefined;
  const value = args[index + 1];
  if (value == null || value.startsWith("--")) {
    throw new TypeError(`${flag} requires a value`);
  }
  args.splice(index, 2);
  return value;
}

// Truthful revision claim: resolved from git at read time, never transcribed.
// A build outside a checkout reports "unknown" rather than a stale sha.
function gitRevision() {
  const run = spawnSync("git", ["rev-parse", "HEAD"], { cwd: REPO_ROOT, encoding: "utf8" });
  const sha = run.status === 0 ? run.stdout.trim() : "";
  return /^[0-9a-f]{7,40}$/.test(sha) ? sha : "unknown";
}

export function discoverTestFiles() {
  const files = [];
  for (const dir of TEST_DIRS) {
    for (const name of readdirSync(join(REPO_ROOT, dir)).sort()) {
      if (name.endsWith(".test.mjs")) files.push(`${dir}/${name}`);
    }
  }
  if (files.length === 0) {
    throw new TypeError(`no test suites discovered under ${TEST_DIRS.join(", ")}; the verification gate refuses to pass empty`);
  }
  return files;
}

function humanCapabilities(value) {
  return `Actuation ${value.version}\ncommands: ${value.commands.join(", ")}\ncontracts: ${Object.values(value.native_contracts).join(", ")}\nrevision: ${value.revision}`;
}

function humanContracts(contracts) {
  return Object.entries(contracts).map(([name, version]) => `${name}\t${version}`).join("\n");
}

function humanAgency(value) {
  return `Agency ${value.agency_ref}\nAgent: ${value.agent_ref}\nWorld: ${value.world_ref}\nScope: ${value.scope_ref}\nRoot for scope: ${value.root_for_scope ? "yes" : "no"}\nMetagency: ${value.metagency.available ? value.metagency.operations.join(", ") : "none"}`;
}

function humanRealised(value) {
  return `Realised Actuation ${value.realised_ref}\nActuation: ${value.actuation_ref}\nAgency: ${value.agency_ref}\nRecurrence: ${value.recurrence}\nObservation: ${value.observation.state}`;
}

function humanStream(value) {
  return `ActuationStream ${value.stream_ref}\nActuation: ${value.actuation_ref}\nAgency: ${value.agency_ref}\nState: ${value.lifecycle.state}\nEvents: ${value.events.length}`;
}

function humanActivity(value) {
  return `Activity ${value.activity_ref}\n${value.summary}\n${value.phase} / ${value.outcome}\nSubject: ${value.subject_ref}\nOwner: ${value.native_owner}`;
}

function humanInstantiation(value) {
  const relation = value.model_relation;
  return `Instantiation receipt ${value.actuation_ref}\nAgency: ${value.agency_ref}\nHarness: ${value.harness_ref ?? "unattributed"}\nModel: ${relation.model_ref}\nPlacement: ${relation.material?.placement ?? "unspecified"}\nInference surface: ${relation.inference_surface.contract_ref}`;
}

function humanCatalog(value) {
  const lines = value.descriptors.map((descriptor) => {
    const probes = Object.keys(descriptor.probe).join("+");
    const facets = descriptor.facets ? Object.keys(descriptor.facets).join(",") : "-";
    return `  ${descriptor.slug.padEnd(20)} ${descriptor.native_kind.padEnd(15)} probes: ${probes.padEnd(28)} facets: ${facets}`;
  });
  return `Harness catalog (r${value.catalog_revision}, ${value.descriptors.length} declared)\n${lines.join("\n")}`;
}

export const COMMANDS = Object.freeze([
  {
    name: "capabilities",
    route: ["capabilities"],
    usage: "actuation capabilities [--json]",
    input: false,
    run: ({ json }) => output(
      { ...ACTUATION_CLI_SURFACE, commands: COMMANDS.map((entry) => entry.name), revision: gitRevision() },
      json,
      humanCapabilities,
    ),
  },
  {
    name: "contract.list",
    route: ["contract", "list"],
    usage: "actuation contract list [--json]",
    input: false,
    run: ({ json }) => output(ACTUATION_CLI_SURFACE.native_contracts, json, humanContracts),
  },
  {
    name: "agency.read",
    route: ["agency"],
    usage: "actuation agency [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => output(agencyReadModel(readJsonInput(args[0] ?? "-", stdin)), json, humanAgency),
  },
  {
    name: "realised.read",
    route: ["realised"],
    usage: "actuation realised [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => output(realisedActuationReadModel(readJsonInput(args[0] ?? "-", stdin)), json, humanRealised),
  },
  {
    name: "stream.read",
    route: ["stream"],
    usage: "actuation stream [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => output(actuationStreamReadModel(readJsonInput(args[0] ?? "-", stdin)), json, humanStream),
  },
  {
    name: "activity.read",
    route: ["activity"],
    usage: "actuation activity [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => output(structuredClone(validateActivity(readJsonInput(args[0] ?? "-", stdin))), json, humanActivity),
  },
  {
    name: "instantiation.read",
    route: ["instantiation"],
    usage: "actuation instantiation [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => output(instantiationReceipt(readJsonInput(args[0] ?? "-", stdin)), json, humanInstantiation),
  },
  {
    name: "instantiation.record",
    route: ["instantiation", "record"],
    usage: "actuation instantiation record [--allow-unattributed] [--out <file>] [file|-] [--json]",
    input: true,
    run: ({ args, json, stdin }) => instantiationRecord(args, stdin, json),
  },
  {
    name: "harness.catalog",
    route: ["harness", "catalog"],
    usage: "actuation harness catalog [--json]",
    input: false,
    run: ({ json }) => output(
      harnessCatalogDocument({
        schema: "actuation.harness-detection/v1",
        document: "catalog",
        catalog_revision: CATALOG_REVISION,
        descriptors: harnessDescriptors(),
      }),
      json,
      humanCatalog,
    ),
  },
  {
    name: "harness.detect",
    route: ["harness", "detect"],
    usage: "actuation harness detect [--only <slugs>] [--versions] [--json]",
    input: false,
    run: ({ args, json }) => harnessDetect(args, json),
  },
  {
    name: "harness.self",
    route: ["harness", "self"],
    usage: "actuation harness self [--json]",
    input: false,
    run: ({ json }) => harnessSelfCommand(json),
  },
  {
    name: "verify",
    route: ["verify"],
    usage: "actuation verify [--json]",
    input: false,
    run: ({ json }) => verify(json),
  },
]);

function instantiationRecord(args, stdin, json) {
  const allowUnattributed = removeFlag(args, "--allow-unattributed");
  const outPath = flagValue(args, "--out");
  const inputPath = args.find((arg) => !arg.startsWith("--"));
  const input = readJsonInput(inputPath ?? "-", stdin);
  const receipt = instantiationReceipt(input);
  if (!receipt.harness_ref && !allowUnattributed) {
    throw new TypeError("receipt has no harness_ref; pass --allow-unattributed to record an unattributed instantiation");
  }
  const record = runDetection({ descriptors: harnessDescriptors() });
  const bound = attachDetectionEvidence(receipt, record);
  if (outPath) {
    appendFileSync(outPath, `${JSON.stringify(bound)}\n`, { encoding: "utf8" });
  }
  if (json) return { code: 0, stdout: JSON.stringify(bound, null, 2) };
  const line = bound.unattributed
    ? "unattributed instantiation (no harness binding claimed)"
    : `bound to ${bound.harness_ref} via ${bound.detection_ref}`;
  const persisted = outPath ? `; appended to ${outPath}` : "";
  return { code: 0, stdout: `Instantiation recorded: ${line}; model ${bound.model_relation.model_ref}${persisted}` };
}

function harnessDetect(args, json) {
  const probeVersions = removeFlag(args, "--versions");
  const only = flagValue(args, "--only");
  let descriptors = harnessDescriptors();
  if (only) {
    const wanted = only.split(",").map((slug) => slug.trim()).filter(Boolean);
    const known = descriptors.map((descriptor) => descriptor.slug);
    const unknown = wanted.filter((slug) => !known.includes(slug));
    if (unknown.length) {
      throw new TypeError(`unknown harness slug(s): ${unknown.join(", ")}; catalog r${CATALOG_REVISION} declares: ${known.join(", ")}`);
    }
    descriptors = descriptors.filter((descriptor) => wanted.includes(descriptor.slug));
  }
  const record = runDetection({ descriptors, probeVersions });
  if (json) return { code: 0, stdout: JSON.stringify(record, null, 2) };
  const lines = record.harnesses.map((entry) => {
    const badge = entry.state === "detected"
      ? `${entry.version ?? entry.receipts?.executable ?? ""}`
      : entry.state === "unavailable"
        ? entry.unavailable_reason
        : "not installed";
    return `  ${entry.slug.padEnd(18)} ${entry.state.padEnd(14)} ${badge}`;
  });
  const disclosure = record.disclosure?.length ? `\nDisclosure:\n${record.disclosure.map((line) => `  ${line}`).join("\n")}` : "";
  return {
    code: 0,
    stdout: `Harness detection (catalog r${record.catalog_revision}, ${record.availability})\n${lines.join("\n")}${disclosure}`,
  };
}

function harnessSelfCommand(json) {
  const self = resolveSelf({ descriptors: harnessDescriptors() });
  if (json) return { code: 0, stdout: JSON.stringify(self, null, 2) };
  const lines = self.matched.map((match) =>
    `  ${match.slug.padEnd(18)} ${match.harness_ref}  markers: ${match.markers.join(", ")}`);
  const headline = self.resolved
    ? `Running inside ${self.resolved.harness_ref} (${self.resolved.markers.join(", ")})`
    : self.ambiguity
      ? "Ambiguous harness identity — multiple markers matched; nested harnesses are real, the innermost is not guessed"
      : "No catalogued harness markers matched this environment";
  const crossCheck = self.resolved && self.detection.states[self.resolved.slug] !== "detected"
    ? `\n  disclosure: ${self.resolved.slug} is not detected on this machine (state ${self.detection.states[self.resolved.slug]}) — marker identity and machine presence disagree`
    : "";
  return {
    code: 0,
    stdout: `Harness self (catalog r${self.catalog_revision}, ${self.detection_ref})\n${headline}${crossCheck}${lines.length ? `\n${lines.join("\n")}` : ""}`,
  };
}

// NOTE: verify spawns the full test suite, which includes this CLI's own
// tests; never invoke the verify command from inside a test — the suites
// would recurse. Test discoverTestFiles() directly instead.
function verify(json) {
  const tests = discoverTestFiles();
  const run = spawnSync(process.execPath, ["--test", ...tests], {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  const receipt = {
    contract: ACTUATION_CLI_SURFACE.contract,
    product: ACTUATION_CLI_SURFACE.product,
    version: ACTUATION_CLI_SURFACE.version,
    revision: gitRevision(),
    status: run.status === 0 ? "ok" : "failed",
    tests,
  };
  if (run.status !== 0) {
    return {
      code: run.status ?? 1,
      stdout: json ? JSON.stringify(receipt, null, 2) : "Actuation native verification: failed",
      stderr: run.stderr?.trim() || run.stdout?.trim() || "native test process failed",
    };
  }
  return {
    code: 0,
    stdout: json ? JSON.stringify(receipt, null, 2) : `Actuation native verification: ok (${tests.length} suites)`,
  };
}
