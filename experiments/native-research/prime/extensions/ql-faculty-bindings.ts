import { existsSync, readFileSync, realpathSync, statSync } from "node:fs";
import { createHash } from "node:crypto";
import { homedir } from "node:os";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

// Machine-local installation bindings only. QL meaning, operation membership,
// decision admission and provider execution remain with their native owners.
export default async function (prime: unknown) {
  const path = process.env.ACTUATION_RESEARCH_INSTALLATION
    || join(homedir(), ".workcell/tools/actuation-ql-agent/current.json");
  if (!existsSync(path)) return;
  const meta = statSync(path);
  if (!meta.isFile() || meta.size > 65536) throw new Error("Invalid native QL faculty installation binding");
  const binding = JSON.parse(readFileSync(path, "utf8"));
  if (binding.schema !== "actuation.prime-faculty-installation/v1"
      || !isAbsolute(binding.binary) || !isAbsolute(binding.configuration)) {
    throw new Error("Invalid native QL faculty installation paths");
  }
  process.env.ACTUATION_RESEARCH_BIN ||= binding.binary;
  process.env.ACTUATION_RESEARCH_FACULTY_CONFIG ||= binding.configuration;
  if (binding.owner_extension) {
    const owner = binding.owner_extension;
    if (typeof owner.module !== "string" || !isAbsolute(owner.module)
        || typeof owner.root !== "string" || !isAbsolute(owner.root)
        || typeof owner.sha256 !== "string" || !/^[0-9a-f]{64}$/.test(owner.sha256)
        || !owner.source_files || typeof owner.source_files !== "object"
        || Array.isArray(owner.source_files)) {
      throw new Error("Invalid shared QL owner extension binding");
    }
    const root = realpathSync(owner.root);
    const sources = Object.entries(owner.source_files);
    if (sources.length === 0 || sources.length > 64) throw new Error("Invalid owner Source closure");
    // This installation ABI names the owner module's complete code and data
    // dependencies. These are assets, never QL laws or classifier labels.
    const required = ["extensions/ql-agent.ts", "extensions/ql-event-context.ts",
      "ql-agent-contracts-v1.schema.json", "provenance.json", "package.json"];
    if (required.some(name => !Object.hasOwn(owner.source_files, name))) {
      throw new Error("Incomplete shared QL owner Source closure");
    }
    let bytes = 0;
    for (const [name, digest] of sources) {
      if (!name || isAbsolute(name) || name.split(/[\\/]/).includes("..")
          || typeof digest !== "string" || !/^[0-9a-f]{64}$/.test(digest)) {
        throw new Error("Invalid owner Source member");
      }
      const file = realpathSync(resolve(root, name));
      const fromRoot = relative(root, file);
      const meta = statSync(file);
      bytes += meta.size;
      if (fromRoot.startsWith(`..${sep}`) || fromRoot === ".." || isAbsolute(fromRoot)
          || !meta.isFile() || meta.size > 1048576 || bytes > 2097152
          || createHash("sha256").update(readFileSync(file)).digest("hex") !== digest) {
        throw new Error("Shared QL owner extension differs from its installed Source");
      }
    }
    const module = realpathSync(owner.module);
    if (relative(root, module) !== "extensions/ql-agent.ts"
        || owner.source_files[relative(root, module)] !== owner.sha256) {
      throw new Error("Shared QL entry is absent from its Source closure");
    }
    const extension = await import(pathToFileURL(module).href);
    if (typeof extension.default !== "function") throw new Error("Shared QL owner extension has no native entry");
    await extension.default(prime, "prime");
  }
  // Loading this extension launches no native process, classifier or kernel.
}
