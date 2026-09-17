import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync, readdirSync } from "node:fs";
const RESEARCH = "/Users/admin/.cache/actuation/acceptance/target/release/actuation-research";
const NODE = "/opt/homebrew/opt/node@24/bin/node";
const OWNER = "/Users/admin/.cache/actuation/acceptance/experiments/ql-runtime/native-owner-instrument/target/release/actuation-ql-owner-instrument";
const BASE = "/Users/admin/.cache/actuation/round5-experiments";
const HOST_REVISION = "9f8e3e2";
const TASKS = process.env.TASKS?.split(",").filter(Boolean) ?? ["S1-RESTRAINT-001"];
const CONDS = (process.env.CONDS ?? "classic,ql-direct,ql-deep").split(",");
function req(task, dir) {
  return { dir, request: {
    operation: "comparison.run",
    trace_ref: `trace:actuation:round5accept:${task.toLowerCase()}`,
    root: dir, repetitions: 1, conditions: CONDS,
    basis: { host_id: "rust-native-acceptance", host_revision: HOST_REVISION,
      benchmark_revision: "series1-round5-accept-2026-09-17",
      runner_revision: "actuation-research-0.2.0",
      review_contract_revision: "ql-series1-run/0.3",
      network_policy_digest: "outbound-https-zai-only" },
    run: { task_id: task, max_steps: 64, max_calls: 64, node: NODE,
      limits: { max_steps: 64, max_depth: 2, max_contexts: 32, max_trace_bytes: 67108864, max_reentries: 0 },
      body: { fixture_provider: false, protocol: "one-shot-json",
        source_basis: { program: "/tmp/glm-body.mjs", adapter: "one-shot-json/glm-body.mjs", provider: "zai", model: "glm-5.3-flash" },
        configuration: { provider: "zai", model: "glm-5.3-flash" },
        process: { program: NODE, args: ["/tmp/glm-body.mjs"], cwd: "/tmp",
          environment: { ZAI_API_KEY: process.env.ZAI_API_KEY, QL_MODEL_NAME: "glm-5.3-flash" },
          timeout_ms: 300000, output_limit: 67108864 } },
      owner: { process: { program: OWNER, args: [], cwd: "/tmp", environment: {}, timeout_ms: 30000, output_limit: 16777216 },
        revision: "08d14e89c6427cb885e117c2e9bc9dc61a0f90b7" } } } };
}
for (const task of TASKS) {
  const dir = `${BASE}/${task}`;
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  if (readdirSync(dir).length > 0) { console.log(`SKIP not empty: ${dir}`); continue; }
  const { request } = req(task, dir);
  console.log(`START ${task} ${CONDS.join(",")} ${new Date().toISOString()}`);
  const t0 = Date.now();
  const r = spawnSync(RESEARCH, ["--json"], { input: JSON.stringify(request), maxBuffer: 1 << 30, timeout: 3 * 60 * 60 * 1000, env: { ...process.env } });
  const el = ((Date.now() - t0) / 1000).toFixed(1);
  const out = r.stdout?.toString() ?? "";
  try {
    const m = JSON.parse(out);
    writeFileSync(`${dir}/result.json`, JSON.stringify(m, null, 2));
    const line = (m.records ?? []).map(x => `${x.condition}:${x.status}/${x.verification?.objective_checks_pass ? "ok" : "fail"}`).join("  ");
    console.log(`DONE ${task} elapsed=${el}s  ${line}`);
  } catch (e) {
    writeFileSync(`${dir}/error.json`, JSON.stringify({ task, exit: r.status, elapsed_s: Number(el), stderr: (r.stderr?.toString() ?? "").slice(0, 500), stdout_head: out.slice(0, 800) }, null, 2));
    console.log(`FAIL ${task} elapsed=${el}s exit=${r.status} err=${e.message}`);
  }
}
console.log("ACCEPTANCE RUN DONE");
