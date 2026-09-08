// Secret-source probe effects for actuation.secret-detection/v1. Every
// effect is injectable so tests run hermetically; the defaults hit the real
// machine. The load-bearing law lives here at the effect layer, not just in
// the contract validator: no effect ever returns a secret value. Env probes
// return names + SHA-256 fingerprints + byte lengths; file probes return
// paths + fingerprints + sizes; vault probes return item metadata only.
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, join, resolve } from "node:path";

const CLI_TIMEOUT_MS = 5000;
const MAX_WALK_DEPTH = 6;
const DEFAULT_MAX_FILES = 500000;
const SKIPPED_DIR_NAMES = new Set([
  "node_modules",
  ".git",
  "target",
  "dist",
  "build",
  ".next",
  "__pycache__",
  ".venv",
  "venv",
]);

function expandHome(path) {
  return path.startsWith("~") ? join(homedir(), path.slice(1)) : resolve(path);
}

function fingerprint(value) {
  return createHash("sha256").update(value).digest("hex");
}

function resolveExecutable(names) {
  for (const name of names) {
    const segments = (process.env.PATH ?? "").split(delimiter).filter(Boolean);
    for (const segment of segments) {
      const candidate = join(segment, name);
      try {
        execFileSync("test", ["-x", candidate], { stdio: "ignore" });
        return { found: true, path: candidate };
      } catch {
        // not executable here, keep walking PATH
      }
    }
  }
  return { found: false, path: null };
}

function cliVersionProbe(path, args = ["--version"]) {
  try {
    const run = spawnSync(path, args, { encoding: "utf8", timeout: CLI_TIMEOUT_MS });
    if (run.error || run.status !== 0) {
      return { ok: false, reason: run.error ? run.error.message : `exit ${run.status}` };
    }
    const line = (run.stdout || "").trim().split("\n")[0] || "(no output)";
    return { ok: true, version: line.slice(0, 120) };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

/**
 * Env probe: match variable NAMES only, fingerprint the values in place.
 * Returns { ok, matched: { name -> { fingerprint_sha256, byte_length } } }.
 * The value never leaves this function except as a digest.
 */
export function envFingerprintProbe(spec, env = process.env) {
  try {
    let names = [];
    if (spec.names != null) {
      names = spec.names.filter((name) => env[name] != null && env[name] !== "");
    } else if (spec.name_pattern != null) {
      const pattern = new RegExp(spec.name_pattern);
      names = Object.keys(env).filter((name) => pattern.test(name) && env[name] !== "");
    }
    const matched = {};
    for (const name of names.sort()) {
      const value = env[name];
      matched[name] = {
        fingerprint_sha256: fingerprint(value),
        byte_length: Buffer.byteLength(value),
      };
    }
    return { ok: true, matched };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

/**
 * File-pattern probe: walk roots (bounded depth, skip dependency/build dirs),
 * match basenames against literal names or glob segments, fingerprint file
 * contents in place. Returns { ok, matched: [{ path, fingerprint_sha256,
 * byte_length }] } — paths are evidence, contents never are.
 */
export function filePatternProbe(spec, fs = { readdirSync, statSync, readFileSync }) {
  try {
    const roots = (spec.roots ?? ["~"]).map(expandHome);
    const patterns = spec.patterns ?? [];
    const maxFiles = spec.max_files ?? DEFAULT_MAX_FILES;
    const matched = [];
    let scanned = 0;

    function matches(basename) {
      return patterns.some((pattern) => {
        if (!pattern.includes("*")) return basename === pattern;
        const regex = new RegExp(
          "^" + pattern.split("*").map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join(".*") + "$",
        );
        return regex.test(basename);
      });
    }

    function walk(dir, depth) {
      if (depth > MAX_WALK_DEPTH || scanned >= maxFiles) return;
      let entries;
      try {
        entries = fs.readdirSync(dir, { withFileTypes: true });
      } catch {
        return;
      }
      for (const entry of entries) {
        if (scanned >= maxFiles) return;
        if (entry.isDirectory()) {
          // Skip only dependency/build/VCS dirs by name. Hidden directories
          // are walked: secret material hides in .secret/, .config/ and
          // friends — blanket-skipping dot-dirs would be a detection hole
          // exactly where the material lives.
          if (!SKIPPED_DIR_NAMES.has(entry.name)) {
            walk(join(dir, entry.name), depth + 1);
          }
          continue;
        }
        scanned += 1;
        if (!entry.isFile() || !matches(entry.name)) continue;
        const path = join(dir, entry.name);
        try {
          const info = fs.statSync(path);
          const content = fs.readFileSync(path);
          matched.push({
            path,
            fingerprint_sha256: createHash("sha256").update(content).digest("hex"),
            byte_length: info.size,
          });
        } catch {
          // unreadable file: absence of evidence, not evidence of absence
        }
      }
    }

    for (const root of roots) walk(root, 0);
    matched.sort((a, b) => a.path.localeCompare(b.path));
    // Truncation is surfaced, never silent: a capped walk must not read as
    // "nothing found" downstream.
    return { ok: true, matched, truncated: scanned >= maxFiles, files_scanned: scanned };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

/**
 * CLI-presence probe: resolve an executable on PATH and capture a version
 * receipt. Same shape as harness detection's executable probe.
 */
export function cliPresenceProbe(spec, deps = {}) {
  try {
    const resolve = deps.resolveExecutable ?? resolveExecutable;
    const versionProbe = deps.versionProbe ?? cliVersionProbe;
    const names = spec.names ?? [];
    for (const name of names) {
      const resolved = resolve([name]);
      if (!resolved.found) continue;
      const version = versionProbe(resolved.path);
      return {
        ok: true,
        found: true,
        path: resolved.path,
        version: version.ok ? version.version : null,
      };
    }
    return { ok: true, found: false, path: null, version: null };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

/**
 * Vault-item probe: prove a 1Password item exists and capture metadata.
 * `op item get` returns field VALUES in its JSON output, so this probe
 * parses the document and re-emits only metadata keys (id, title, vault,
 * updated_at, field count). The fields array is counted, never passed on.
 */
export function vaultItemProbe(spec, deps = {}) {
  const run = deps.spawnSync ?? spawnSync;
  try {
    const result = run("op", ["item", "get", spec.item_ref, "--format", "json"], {
      encoding: "utf8",
      timeout: CLI_TIMEOUT_MS,
    });
    if (result.error) {
      return { ok: false, reason: `spawn failed: ${result.error.message}` };
    }
    if (result.status !== 0) {
      const stderr = (result.stderr || "").trim().slice(0, 200);
      if (/not (found|signed in)|item not found/i.test(stderr)) {
        return { ok: true, found: false };
      }
      return { ok: false, reason: `exit ${result.status}: ${stderr}` };
    }
    const item = JSON.parse(result.stdout);
    return {
      ok: true,
      found: true,
      item_id: typeof item.id === "string" ? item.id : null,
      title: typeof item.title === "string" ? item.title : null,
      vault: item.vault && typeof item.vault.name === "string" ? item.vault.name : null,
      updated_at: typeof item.updated_at === "string" ? item.updated_at : null,
      field_count: Array.isArray(item.fields) ? item.fields.length : null,
    };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

export const defaultEffects = {
  env: envFingerprintProbe,
  "file-pattern": filePatternProbe,
  "cli-presence": cliPresenceProbe,
  "vault-item": vaultItemProbe,
};
