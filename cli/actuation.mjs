import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { agencyReadModel } from "../contracts/agency.mjs";
import { realisedActuationReadModel } from "../contracts/realised-actuation.mjs";
import { actuationStreamReadModel } from "../contracts/actuation-stream.mjs";
import { validateActivity } from "../contracts/activity.mjs";
import { modelBearingReceipt } from "../contracts/model-bearing.mjs";
import { ACTUATION_CLI_SURFACE } from "./surface.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const INPUT_COMMANDS = new Set(["agency", "realised", "stream", "activity", "model"]);

export function executeCommand(argv, { stdin = "" } = {}) {
  const args = [...argv];
  const json = removeFlag(args, "--json");
  const command = args[0];

  if (command == null || ["help", "--help", "-h"].includes(command)) {
    return { code: 0, stdout: helpText() };
  }
  if (["--version", "version"].includes(command)) {
    return { code: 0, stdout: `actuation ${ACTUATION_CLI_SURFACE.version}` };
  }
  if (command === "capabilities") {
    return output(ACTUATION_CLI_SURFACE, json, humanCapabilities);
  }
  if (command === "contract" && args[1] === "list") {
    return output(ACTUATION_CLI_SURFACE.native_contracts, json, humanContracts);
  }
  if (command === "agency") {
    const input = readJsonInput(args[1] ?? "-", stdin);
    return output(agencyReadModel(input), json, humanAgency);
  }
  if (command === "realised") {
    const input = readJsonInput(args[1] ?? "-", stdin);
    return output(realisedActuationReadModel(input), json, humanRealised);
  }
  if (command === "stream") {
    const input = readJsonInput(args[1] ?? "-", stdin);
    return output(actuationStreamReadModel(input), json, humanStream);
  }
  if (command === "activity") {
    const input = readJsonInput(args[1] ?? "-", stdin);
    return output(structuredClone(validateActivity(input)), json, humanActivity);
  }
  if (command === "model") {
    const input = readJsonInput(args[1] ?? "-", stdin);
    return output(modelBearingReceipt(input), json, humanModel);
  }
  if (command === "verify") {
    return verify(json);
  }
  throw new TypeError(`unknown command ${command}; run actuation help`);
}

export function main(argv = process.argv.slice(2)) {
  try {
    const stdin = commandNeedsStdin(argv) ? readFileSync(0, "utf8") : "";
    const result = executeCommand(argv, { stdin });
    if (result.stdout) process.stdout.write(`${result.stdout}\n`);
    if (result.stderr) process.stderr.write(`${result.stderr}\n`);
    return result.code;
  } catch (error) {
    process.stderr.write(`actuation: ${error.message}\n`);
    return 2;
  }
}

function helpText() {
  return `Actuation ${ACTUATION_CLI_SURFACE.version}

Usage:
  actuation --version
  actuation capabilities [--json]
  actuation contract list [--json]
  actuation agency [file|-] [--json]
  actuation realised [file|-] [--json]
  actuation stream [file|-] [--json]
  actuation activity [file|-] [--json]
  actuation model [file|-] [--json]
  actuation verify [--json]

The command projects Actuation-owned Agency, realised Actuation, stream, Activity and model-bearing contracts.`;
}

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

function humanCapabilities(surface) {
  return `Actuation ${surface.version}\ncommands: ${surface.commands.join(", ")}\ncontracts: ${Object.values(surface.native_contracts).join(", ")}`;
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

function humanModel(value) {
  const relation = value.model_relation;
  return `Model-bearing Actuation ${value.actuation_ref}\nAgency: ${value.agency_ref}\nModel: ${relation.model_ref}\nPlacement: ${relation.material?.placement ?? "unspecified"}\nInference surface: ${relation.inference_surface.contract_ref}`;
}

function verify(json) {
  const tests = [
    "contracts/agency.test.mjs",
    "contracts/model-bearing.test.mjs",
    "contracts/realised-actuation.test.mjs",
    "contracts/actuation-stream.test.mjs",
    "contracts/activity.test.mjs",
    "contracts/harness-detection.test.mjs",
    "detection/catalog.test.mjs",
    "cli/actuation.test.mjs",
  ];
  const run = spawnSync(process.execPath, ["--test", ...tests], {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  const receipt = {
    contract: ACTUATION_CLI_SURFACE.contract,
    product: ACTUATION_CLI_SURFACE.product,
    version: ACTUATION_CLI_SURFACE.version,
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

function commandNeedsStdin(argv) {
  const args = argv.filter((arg) => arg !== "--json");
  const command = args[0];
  return INPUT_COMMANDS.has(command) && (args[1] == null || args[1] === "-");
}

function removeFlag(args, flag) {
  const index = args.indexOf(flag);
  if (index === -1) return false;
  args.splice(index, 1);
  return true;
}
