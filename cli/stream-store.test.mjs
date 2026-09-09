import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

import { executeCommand } from "./actuation.mjs";
import { capabilityDescriptorBySlug } from "../detection/catalog.mjs";

function store() {
  return mkdtempSync(join(tmpdir(), "actuation-stream-cli-"));
}

const identity = {
  stream_ref: "actuation:stream:cli-test",
  actuation_ref: "actuation:cli-test",
  agency_ref: "agency:cli-test",
  agent_session_ref: "agent-session:cli-test",
};

function run(args, stdin = "") {
  const argv = [...args, "--json"];
  const result = executeCommand(argv, { stdin });
  assert.equal(result.code, 0, `actuation ${args.join(" ")} failed: ${result.stdout ?? result.stderr}`);
  return JSON.parse(result.stdout);
}

test("the durable stream commands answer over the CLI, end to end", () => {
  const root = store();
  const descriptorEvent = capabilityDescriptorBySlug("zcode").native_events.find((event) => event.event !== "custom");

  const opened = run(["stream", "open", "--store", root, "-"], JSON.stringify(identity));
  assert.equal(opened.stream_ref, identity.stream_ref);
  assert.equal(opened.lifecycle.state, "open");

  const recorded = run(
    ["stream", "record", "--store", root, "-"],
    JSON.stringify({
      stream_ref: identity.stream_ref,
      harness: "zcode",
      native_event: descriptorEvent.native_name,
      observed_at: "2026-09-06T12:00:00Z",
    }),
  );
  assert.equal(recorded.event.sequence, 1);
  assert.equal(recorded.event.metadata.boundary, descriptorEvent.event);

  const replayed = run(["stream", "replay", identity.stream_ref, "--store", root]);
  assert.deepEqual(replayed.events.map((event) => event.sequence), [1]);
  assert.equal(replayed.cursor.has_more, false);

  const closed = run(["stream", "close", identity.stream_ref, "--store", root]);
  assert.equal(closed.lifecycle.state, "closed");
  assert.ok(closed.lifecycle.ended_at);
});

test("stream record refuses an undeclared native event through the CLI", () => {
  const root = store();
  assert.throws(
    () => executeCommand(
      ["stream", "record", "--store", root, "-", "--json"],
      { stdin: JSON.stringify({ ...identity, harness: "zcode", native_event: "Notification" }) },
    ),
    /Notification/,
    "an undeclared native event is a refusal, not a silent record",
  );
});

test("stream usage normalizes a native provider record, persists no content, and deduplicates replay", () => {
  const root = store();
  const document = {
    adapter: "claude-code-transcript",
    stream_ref: identity.stream_ref,
    identity,
    correlation: {
      actuation_ref: identity.actuation_ref,
      agent_session_ref: identity.agent_session_ref,
      native_trace_ref: "trace:claude-code:cli-line-1",
    },
    native_event: {
      type: "assistant",
      sessionId: "session-cli-native",
      requestId: "request-cli-native",
      timestamp: "2026-09-08T20:00:00Z",
      message: {
        id: "message-cli-native",
        model: "claude-fable-5",
        stop_reason: "end_turn",
        content: [{ type: "text", text: "must not persist" }],
        usage: { input_tokens: 17, output_tokens: 5, cache_read_input_tokens: 13 },
      },
    },
  };
  const first = run(["stream", "usage", "--store", root, "-"], JSON.stringify(document));
  const replay = run(["stream", "usage", "--store", root, "-"], JSON.stringify(document));
  assert.equal(first.deduplicated, false);
  assert.equal(replay.deduplicated, true);
  assert.equal(replay.cursor.last_sequence, 1);
  assert.equal(replay.event.model_usage.tokens.input, 17);
  assert.equal(JSON.stringify(replay).includes("must not persist"), false);
});

// A pre-normalised actuation.model-usage/v1 observation needs no native-format
// translation — the "observation" adapter's whole job is to validate it and
// hand it to the exact same durable-store path every other adapter uses.
function normalisedObservation(overrides = {}) {
  return {
    schema: "actuation.model-usage/v1",
    usage_ref: "model-usage:test:observation-adapter-1",
    actuation_ref: identity.actuation_ref,
    invocation_ref: "invocation:test:observation-adapter-1",
    correlation: { agent_session_ref: identity.agent_session_ref },
    provider: { standing: "provider-reported", ref: "provider:anthropic" },
    model: { standing: "provider-reported", name: "claude-fable-5" },
    tokens: { standing: "provider-reported", input: 42, output: 7 },
    cache: { standing: "not-reported" },
    timing: { latency: { standing: "not-reported" } },
    cost: { standing: "not-reported" },
    outcome: { state: "completed", standing: "provider-reported" },
    provenance: {
      reporter_ref: "test-harness",
      native_event_ref: "test:event:observation-adapter-1",
      native_schema: "test.native-schema/v1",
      observed_at: "2026-09-09T00:00:00Z",
      raw_evidence_refs: ["trace:test:observation-adapter-1"],
    },
    ...overrides,
  };
}

test("stream usage adapter 'observation' records an already-normalised observation and dedups a replay", () => {
  const root = store();
  const document = {
    adapter: "observation",
    stream_ref: identity.stream_ref,
    identity,
    native_event: normalisedObservation(),
  };
  const first = run(["stream", "usage", "--store", root, "-"], JSON.stringify(document));
  const replay = run(["stream", "usage", "--store", root, "-"], JSON.stringify(document));
  assert.equal(first.deduplicated, false);
  assert.equal(replay.deduplicated, true);
  assert.equal(replay.cursor.last_sequence, 1);
  assert.equal(replay.event.model_usage.tokens.input, 42);
  assert.equal(replay.event.model_usage.provider.ref, "provider:anthropic");
});

test("stream usage adapter 'observation' refuses an invalid observation", () => {
  const root = store();
  const document = {
    adapter: "observation",
    stream_ref: identity.stream_ref,
    identity,
    native_event: { schema: "actuation.model-usage/v1" },
  };
  assert.throws(
    () => executeCommand(["stream", "usage", "--store", root, "-", "--json"], { stdin: JSON.stringify(document) }),
    /ModelUsageObservation/,
    "an invalid observation must be refused before it ever reaches the durable store",
  );
});

test("stream usage refuses an unknown adapter and names the accepted ones", () => {
  const root = store();
  const document = {
    adapter: "some-other-route",
    stream_ref: identity.stream_ref,
    identity,
    native_event: normalisedObservation(),
  };
  assert.throws(
    () => executeCommand(["stream", "usage", "--store", root, "-", "--json"], { stdin: JSON.stringify(document) }),
    /stream usage adapter must be one of claude-code-transcript, observation/,
  );
});

test("the served binary reads stream usage from stdin when --store precedes the input marker", () => {
  const root = store();
  const document = {
    adapter: "claude-code-transcript",
    stream_ref: identity.stream_ref,
    identity,
    correlation: { actuation_ref: identity.actuation_ref, native_trace_ref: "trace:served-cli" },
    native_event: {
      type: "assistant",
      sessionId: "session-served-cli",
      timestamp: "2026-09-08T20:00:00Z",
      message: { id: "message-served-cli", model: "claude-fable-5", stop_reason: "end_turn", usage: { input_tokens: 3, output_tokens: 1 } },
    },
  };
  const result = spawnSync(process.execPath, ["bin/actuation", "stream", "usage", "--store", root, "-", "--json"], {
    cwd: new URL("..", import.meta.url),
    input: JSON.stringify(document),
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).event.model_usage.tokens.input, 3);
});
