import { readFileSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join } from "node:path";

// Machine-local installation bindings only. QL meaning, operation membership,
// decision admission and provider execution remain with their native owners.
export default function (_prime: unknown) {
  if (process.env.ACTUATION_RESEARCH_BIN && process.env.ACTUATION_RESEARCH_FACULTY_CONFIG) return;
  const path = process.env.ACTUATION_RESEARCH_INSTALLATION
    || join(homedir(), ".workcell/tools/actuation-ql-agent/current.json");
  const meta = statSync(path);
  if (!meta.isFile() || meta.size > 65536) throw new Error("Invalid native QL faculty installation binding");
  const binding = JSON.parse(readFileSync(path, "utf8"));
  if (binding.schema !== "actuation.prime-faculty-installation/v1"
      || !isAbsolute(binding.binary) || !isAbsolute(binding.configuration)) {
    throw new Error("Invalid native QL faculty installation paths");
  }
  process.env.ACTUATION_RESEARCH_BIN ||= binding.binary;
  process.env.ACTUATION_RESEARCH_FACULTY_CONFIG ||= binding.configuration;
  // Loading this extension launches no native process, classifier or kernel.
}
