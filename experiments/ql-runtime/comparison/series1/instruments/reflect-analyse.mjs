#!/usr/bin/env node
// The analytical step of the reflect chain: jev returns a reading (the
// twelve lenses mapped across their machinery); this step has the model
// ANALYSE the work through that reading — what the map shows about how the
// work thinks, grounded in the work's own sentences. The lenses are
// refraction media of the P/P' positions; they are not the work, and the
// analysis says where the reading holds and where it is off.
// stdin:  {"subject": string, "reading": <the jev-reflect reading object>}
// stdout: {"analysis": string, "latency_ms": n}
// Model: glm-5.3-flash via the ZAI endpoint (the loop model, not the
// classifier — jev reads, the model analyses). Fail-closed.
import { readFileSync } from "node:fs";

const ENDPOINT =
  process.env.QL_MODEL_ENDPOINT ||
  "https://api.z.ai/api/coding/paas/v4/chat/completions";
const MODEL = process.env.QL_MODEL_NAME || "glm-5.3-flash";
const API_KEY = process.env.ZAI_API_KEY;

function fail(message) {
  process.stderr.write(`reflect-analyse: ${String(message).slice(0, 400)}\n`);
  process.exit(2);
}

let req;
try {
  req = JSON.parse(readFileSync(0, "utf8"));
} catch (e) {
  fail(`request parse: ${e.message}`);
}
const subject = typeof req.subject === "string" ? req.subject.trim() : "";
const reading = req.reading ?? null;
if (!subject) fail("request needs a subject");
if (!reading) fail("request needs the jev reading");

const system = `You are the analytical reader in an MEF instrument chain. A classifier (jev) has returned a reading of a presented work: twelve lenses, each mapped across its six defined sub-slots. The lenses are refraction media of the work's responsibilities — purely relational, no inherent manifest content; they are not the work.

Your task: analyse the work THROUGH this reading. Say what the map shows about how the work actually thinks — which of its moves the reading caught and what those dominant slots mean in this work's own terms; where the reading is faithful; where it is off and why. Ground every claim in the work's own sentences; quote the work where it matters. Do not restate the framework; do not pad. Write the analysis as flowing prose a reader of the work would learn from.`;

const user = `THE WORK:\n${subject}\n\nTHE MEF READING (JSON):\n${JSON.stringify(reading)}`;

const t0 = Date.now();
const payload = JSON.stringify({
  model: MODEL,
  temperature: 0,
  stream: true,
  messages: [
    { role: "system", content: system },
    { role: "user", content: user },
  ],
});
// node:https rather than fetch: undici's headers timeout aborts long
// generations; the model may need minutes before the response begins.
import https from "node:https";
const out = await new Promise((resolve, reject) => {
  const u = new URL(ENDPOINT);
  const r = https.request(
    { hostname: u.hostname, path: u.pathname, method: "POST",
      headers: { Authorization: `Bearer ${API_KEY}`, "Content-Type": "application/json",
                 "Content-Length": Buffer.byteLength(payload) },
      timeout: 300000 },
    (res) => {
      if (res.statusCode !== 200) {
        let bodyTxt = "";
        res.on("data", (d) => (bodyTxt += d));
        res.on("end", () => reject(new Error(`zai-api-${res.statusCode}: ${bodyTxt.slice(0, 300)}`)));
        return;
      }
      // SSE: deltas arrive as data: {...} lines; headers return immediately.
      let analysis = "";
      let buffer = "";
      res.on("data", (d) => {
        buffer += d;
        let idx;
        while ((idx = buffer.indexOf("\n")) !== -1) {
          const line = buffer.slice(0, idx).trim();
          buffer = buffer.slice(idx + 1);
          if (!line.startsWith("data:")) continue;
          const data = line.slice(5).trim();
          if (data === "[DONE]") continue;
          try {
            const piece = JSON.parse(data);
            analysis += piece.choices?.[0]?.delta?.content ?? "";
          } catch { /* partial line across chunks */ }
        }
      });
      res.on("end", () => resolve({ choices: [{ message: { content: analysis } }] }));
    });
  r.on("timeout", () => r.destroy(new Error("model request timed out after 300s")));
  r.on("error", reject);
  r.write(payload);
  r.end();
}).catch((e) => fail(`transport: ${e.message}`));
const analysis = out.choices?.[0]?.message?.content;
if (!analysis || !analysis.trim()) fail("empty analysis from the model");
process.stdout.write(JSON.stringify({ analysis, latency_ms: Date.now() - t0 }) + "\n");
