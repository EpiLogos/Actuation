// Probe effects for harness detection. Every effect is injectable so tests
// run hermetically; the defaults hit the real machine.
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, statSync, readdirSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, join, resolve } from "node:path";

const VERSION_TIMEOUT_MS = 5000;
const SERVICE_TIMEOUT_MS = 4000;

function expandHome(path) {
  return path.startsWith("~") ? join(homedir(), path.slice(1)) : resolve(path);
}

function resolveExecutable(names) {
  for (const name of names) {
    const segments = (process.env.PATH ?? "").split(delimiter).filter(Boolean);
    for (const segment of segments) {
      const candidate = join(segment, name);
      if (isExecutable(candidate)) {
        return { found: true, path: candidate };
      }
    }
  }
  return { found: false, path: null };
}

function isExecutable(path) {
  try {
    execFileSync("test", ["-x", path], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function versionProbe(path, args) {
  try {
    const run = spawnSync(path, args, { encoding: "utf8", timeout: VERSION_TIMEOUT_MS });
    if (run.error) {
      return { ok: false, reason: `spawn failed: ${run.error.message}` };
    }
    if (run.status !== 0) {
      return { ok: false, reason: `exit ${run.status}: ${(run.stderr || run.stdout || "").trim().slice(0, 200)}` };
    }
    const line = (run.stdout || "").trim().split("\n")[0] || "(no output)";
    return { ok: true, version: line.slice(0, 120) };
  } catch (error) {
    return { ok: false, reason: error.message };
  }
}

function statProbe(path) {
  try {
    const info = statSync(path);
    return { exists: true, mtimeMs: info.mtimeMs, size: info.size, isDir: info.isDirectory() };
  } catch {
    return { exists: false };
  }
}

function hashProbe(path) {
  try {
    return { ok: true, sha256: createHash("sha256").update(readFileSync(path)).digest("hex") };
  } catch {
    return { ok: false };
  }
}

function dirCountProbe(path) {
  try {
    const entries = readdirSync(path, { withFileTypes: true });
    return { exists: true, count: entries.length };
  } catch (error) {
    if (error.code === "ENOENT") return { exists: false };
    return { exists: false, error: error.message };
  }
}

function envProbe(names, env = process.env) {
  const matched = {};
  try {
    for (const name of names) {
      if (env[name] != null && env[name] !== "") {
        matched[name] = env[name];
      }
    }
    return { ok: true, matched };
  } catch (error) {
    return { ok: false, error: error.message };
  }
}

// Service probes stay synchronous and injectable like every other effect.
// Result convention mirrors the executable probe: ok:true with a
// presence-shaped detail ("http 200 from ...", "daemon x running") is live
// evidence; ok:true with an absence-shaped detail ("no listener at ...",
// "daemon x not running") ran cleanly and found nothing; ok:false means the
// probe mechanism itself could not run and must never read as absence.
function httpServiceProbe(spec) {
  const url = spec.default_url ?? spec.url;
  if (!url) return { ok: false, reason: "http service probe requires default_url" };
  const run = spawnSync("curl", ["-sS", "--max-time", "2", "-o", "/dev/null", "-w", "%{http_code}", url], {
    encoding: "utf8",
    timeout: SERVICE_TIMEOUT_MS,
  });
  if (run.error) return { ok: false, reason: `curl unavailable: ${run.error.message}` };
  if (run.status === 0) {
    const code = (run.stdout || "").trim();
    return /^\d{3}$/.test(code)
      ? { ok: true, detail: `http ${code} from ${url}` }
      : { ok: false, reason: `unparseable curl output: ${code || "(empty)"}` };
  }
  if (run.status === 7) return { ok: true, detail: `no listener at ${url}` };
  return { ok: false, reason: `curl exit ${run.status}: ${(run.stderr || "").trim().slice(0, 200)}` };
}

function daemonServiceProbe(spec) {
  if (!spec.name) return { ok: false, reason: "daemon service probe requires name" };
  const run = spawnSync("pgrep", ["-x", spec.name], { encoding: "utf8", timeout: SERVICE_TIMEOUT_MS });
  if (run.error) return { ok: false, reason: `pgrep unavailable: ${run.error.message}` };
  if (run.status === 0) return { ok: true, detail: `daemon ${spec.name} running` };
  if (run.status === 1) return { ok: true, detail: `daemon ${spec.name} not running` };
  return { ok: false, reason: `pgrep exit ${run.status}` };
}

function serviceProbe(spec) {
  if (spec?.kind === "http") return httpServiceProbe(spec);
  if (spec?.kind === "daemon") return daemonServiceProbe(spec);
  return { ok: false, reason: `unsupported service kind: ${spec?.kind ?? "(none)"}` };
}

export function realEffects() {
  return {
    resolveExecutable,
    versionProbe,
    statProbe,
    hashProbe,
    dirCountProbe,
    envProbe,
    serviceProbe,
    expandHome,
  };
}
