// Dispatch, help and process plumbing. Everything command-shaped (routes,
// usage lines, handlers) lives in the COMMANDS table in commands.mjs; this
// file only matches argv against that table and renders its consequences.
import { readFileSync } from "node:fs";
import { ACTUATION_CLI_SURFACE } from "./surface.mjs";
import { COMMANDS } from "./commands.mjs";

// Longest route first so "instantiation record" wins over "instantiation".
const ROUTES = [...COMMANDS].sort((a, b) => b.route.length - a.route.length);

export function matchRoute(args) {
  for (const entry of ROUTES) {
    if (entry.route.every((word, index) => args[index] === word)) {
      return { entry, rest: args.slice(entry.route.length) };
    }
  }
  return null;
}

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
  const match = matchRoute(args);
  if (!match) {
    if (command === "harness") {
      throw new TypeError(`unknown harness subcommand ${args[1] ?? "(none)"}; expected catalog, detect or self`);
    }
    throw new TypeError(`unknown command ${command}; run actuation help`);
  }
  return match.entry.run({ args: match.rest, json, stdin });
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
  const usage = COMMANDS.map((entry) => `  ${entry.usage}`).join("\n");
  return `Actuation ${ACTUATION_CLI_SURFACE.version}

Usage:
  actuation --version
${usage}

Read-model commands project the matching Actuation contract; "harness catalog" declares what this product can detect, "harness detect" proves which of them exist on this machine, and "harness self" identifies which one this process runs inside.`;
}

function commandNeedsStdin(argv) {
  const args = argv.filter((arg) => arg !== "--json");
  const match = matchRoute(args);
  return Boolean(match?.entry.input) && (match.rest[0] == null || match.rest[0] === "-");
}

function removeFlag(args, flag) {
  const index = args.indexOf(flag);
  if (index === -1) return false;
  args.splice(index, 1);
  return true;
}
