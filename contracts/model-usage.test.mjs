import assert from "node:assert/strict";
import test from "node:test";

import {
  modelUsageFromClaudeCodeTranscript,
  modelUsageFromCodexExecEvents,
  validateModelUsageObservation,
} from "./model-usage.mjs";

function codexEvents() {
  return [
    { type: "thread.started", thread_id: "01a08641-70e1-7552-9adc-c969a82504ae" },
    { type: "turn.started" },
    { type: "item.completed", item: { id: "item_0", type: "agent_message", text: "content deliberately ignored" } },
    { type: "turn.completed", usage: { input_tokens: 20213, cached_input_tokens: 12288, cache_write_input_tokens: 0, output_tokens: 9, reasoning_output_tokens: 0 } },
  ];
}

// Shape and fields observed in a real Claude Code transcript on 2026-09-08.
// Identifiers are fixture-local; content is deliberately omitted because it is
// neither needed nor permitted in the portable usage observation.
function nativeEvent(overrides = {}) {
  return {
    type: "assistant",
    sessionId: "37091766-f305-4178-bbc3-4ff417dbcef7",
    uuid: "3a27ab8a-c8b6-4c15-84ec-066ab50e7886",
    requestId: "req_011CdAMUXikJ2yx4sMo5Phue",
    timestamp: "2026-07-18T23:16:15.618Z",
    message: {
      id: "msg_011CdAMUYyuCjjHDEJwMTTeX",
      model: "claude-fable-5",
      role: "assistant",
      stop_reason: "max_tokens",
      usage: {
        input_tokens: 2,
        cache_creation_input_tokens: 53010,
        cache_read_input_tokens: 26788,
        output_tokens: 64000,
        server_tool_use: { web_search_requests: 0, web_fetch_requests: 0 },
        service_tier: "standard",
        speed: "standard",
        inference_geo: "not_available",
      },
    },
    ...overrides,
  };
}

const correlation = {
  actuation_ref: "actuation:usage-test",
  activity_ref: "activity:usage-test",
  agency_ref: "agency:usage-test",
  agent_session_ref: "agent-session:usage-test",
  native_trace_ref: "trace:claude-code:fixture-line-1",
  external_refs: ["factory:run:external-only"],
};

test("real Claude Code provider evidence normalizes exact token, cache, identity and partial-result facts", () => {
  const observed = modelUsageFromClaudeCodeTranscript(nativeEvent(), correlation);

  assert.equal(observed.model.name, "claude-fable-5");
  assert.equal(observed.model.variant, "standard");
  assert.deepEqual(observed.tokens, { standing: "normalized-from-native", input: 2, output: 64000 });
  assert.deepEqual(observed.cache, { standing: "normalized-from-native", read_input: 26788, creation_input: 53010 });
  assert.equal(observed.outcome.state, "partial");
  assert.equal(observed.outcome.reason, "max_tokens");
  assert.equal(observed.provider.standing, "not-reported", "the transcript does not prove Anthropic vs Bedrock/Vertex routing");
  assert.equal(observed.timing.completed_at, nativeEvent().timestamp);
  assert.equal(observed.timing.latency.standing, "not-reported");
  assert.equal(observed.cost.standing, "not-reported");
  assert.deepEqual(observed.correlation.external_refs, ["factory:run:external-only"]);
});

test("an otherwise valid provider event with no usage remains valid and never becomes zero", () => {
  const input = nativeEvent();
  delete input.message.usage;
  input.message.stop_reason = "end_turn";
  const observed = modelUsageFromClaudeCodeTranscript(input, correlation);
  assert.deepEqual(observed.tokens, { standing: "not-reported" });
  assert.deepEqual(observed.cache, { standing: "not-reported" });
  assert.equal(observed.outcome.state, "completed");
});

test("synthetic error rows are refused as provider usage evidence", () => {
  const input = nativeEvent({ isApiErrorMessage: true });
  input.message = { ...input.message, model: "<synthetic>" };
  assert.throws(() => modelUsageFromClaudeCodeTranscript(input, correlation), /synthetic/);
});

test("derived cost requires an exact pricing source, revision and effective date", () => {
  const observed = modelUsageFromClaudeCodeTranscript(nativeEvent(), correlation);
  observed.cost = { standing: "derived", amount: 1.25, currency: "USD" };
  assert.throws(() => validateModelUsageObservation(observed), /pricing_basis/);

  observed.cost.pricing_basis = {
    source_ref: "pricing:anthropic:published",
    revision: "2026-07-01",
    effective_at: "2026-07-01T00:00:00Z",
  };
  assert.equal(validateModelUsageObservation(observed).cost.standing, "derived");
});

test("unavailable and not-reported standing cannot smuggle values", () => {
  const observed = modelUsageFromClaudeCodeTranscript(nativeEvent(), correlation);
  observed.timing.latency = { standing: "not-reported", milliseconds: 12 };
  assert.throws(() => validateModelUsageObservation(observed), /must not carry milliseconds/);
});

test("the bounded provider-fact field refuses arbitrary scalar payloads", () => {
  const observed = modelUsageFromClaudeCodeTranscript(nativeEvent(), correlation);
  observed.provider_facts.api_key = "must-never-cross";
  assert.throws(() => validateModelUsageObservation(observed), /not an admitted bounded native fact/);
});

test("real Codex exec JSONL evidence normalizes only the facts its terminal event supplies", () => {
  const observed = modelUsageFromCodexExecEvents(codexEvents(), {
    actuation_ref: "actuation:codex-provider-proof",
    invocation_ref: "invocation:codex:provider-proof-1",
    native_trace_ref: "trace:codex:provider-proof-2026-09-09",
    observed_at: "2026-09-09T13:00:38.593Z",
    agent_session_ref: "agent-session:codex-provider-proof",
  });
  assert.deepEqual(observed.tokens, { standing: "normalized-from-native", input: 20213, output: 9 });
  assert.deepEqual(observed.cache, { standing: "normalized-from-native", read_input: 12288, creation_input: 0 });
  assert.deepEqual(observed.usage_classes, [{ class: "reasoning_output", quantity: 0, unit: "tokens", standing: "normalized-from-native" }]);
  assert.equal(observed.provider.standing, "not-reported");
  assert.equal(observed.model.standing, "not-reported");
  assert.equal(observed.timing.latency.standing, "not-reported");
  assert.equal(observed.cost.standing, "not-reported");
  assert.equal(observed.outcome.state, "completed");
  assert.equal(observed.correlation.native_session_ref, "codex:thread:01a08641-70e1-7552-9adc-c969a82504ae");
});

test("Codex exec evidence refuses ambiguous or incomplete invocation boundaries", () => {
  const options = {
    actuation_ref: "actuation:codex-provider-proof",
    invocation_ref: "invocation:codex:provider-proof-1",
    native_trace_ref: "trace:codex:provider-proof-2026-09-09",
    observed_at: "2026-09-09T13:00:38.593Z",
  };
  assert.throws(() => modelUsageFromCodexExecEvents(codexEvents().slice(0, -1), options), /exactly one/);
  assert.throws(() => modelUsageFromCodexExecEvents([...codexEvents(), codexEvents().at(-1)], options), /exactly one/);
});

export { correlation, nativeEvent };
