import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { resolve } from "node:path";

// A real fresh Prime CLI session, its persistent Python kernel and the installed
// Actuation faculty. Requires an explicitly permitted agent provider; no SDK,
// owner, classifier or tool execution is simulated.
const [cwd, requestFile, outputFile, operation = "ql_project_event"] = process.argv.slice(2);
assert(cwd && requestFile && outputFile);
assert(["ql_project_event", "ql_decide", "ql_invoke"].includes(operation));
const path = resolve(outputFile);
const prompt = `In the installed Prime Python kernel, import json and ql_relational. Read the exact JSON request at ${resolve(requestFile)}. Await ql_relational.${operation}(request). Save only its complete returned native receipt as JSON to ${path}. Retain any unresolved/refused result. Do not implement an alternative calculation. Return one short factual sentence.`;
const start = performance.now();
const trace = [];
const stderr = [];
let bytes = 0;
await new Promise((resolvePromise, reject) => {
  const child = spawn(process.env.PRIME_AGENT_BIN || "prime-agent", ["--print", "--mode", "json", "--no-session",
    "--offline", "--no-context-files", "--no-themes", "--no-prompt-templates", "--thinking", "low",
    "--tools", "ipython", prompt], {cwd, detached:true, shell:false, stdio:["ignore","pipe","pipe"]});
  let failure;
  let hardStop;
  const stop = error => {
    if (failure) return;
    failure = error;
    try { process.kill(-child.pid, "SIGTERM"); } catch (e) { if (e.code !== "ESRCH") failure = e; }
    hardStop = setTimeout(() => { try { process.kill(-child.pid,"SIGKILL"); } catch {} },5000);
    hardStop.unref();
  };
  const timeout = setTimeout(() => stop(new Error("Prime native session exceeded 210 seconds")),210000);
  child.stdout.on("data",b => {bytes+=b.length;if(bytes>8*1024*1024)stop(new Error("Prime trace exceeds bound"));else trace.push(b);});
  child.stderr.on("data",b => {bytes+=b.length;if(bytes>8*1024*1024)stop(new Error("Prime trace exceeds bound"));else stderr.push(b);});
  child.on("error",e => {failure ||= e;});
  child.on("close",code => {
    clearTimeout(timeout);if(hardStop)clearTimeout(hardStop);
    if(failure)reject(failure);else if(code!==0)reject(new Error(Buffer.concat(stderr).toString()));else resolvePromise();
  });
});
await writeFile(path+".agent.jsonl",Buffer.concat(trace));
const rows = Buffer.concat(trace).toString().split("\n").filter(Boolean).map(l => JSON.parse(l));
assert(!rows.some(r => r.type === "message_end" && r.message?.stopReason === "error"),"agent provider failed");
assert(rows.some(r => r.type === "tool_execution_end" && r.toolName === "ipython"),"native Prime Python must actually execute");
const receipt = JSON.parse(await readFile(path,"utf8"));
assert.equal(receipt.schema,operation === "ql_invoke" ? "ql.epi-logos-agent-invocation-result/v1" : "ql.agent-projection/v1");
process.stdout.write(JSON.stringify({schema:"actuation.prime-native-session-ql/v1",operation,
  duration_ms:performance.now()-start,receipt:path,native_schema:receipt.schema})+"\n");
