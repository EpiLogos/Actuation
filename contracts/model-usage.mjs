export const MODEL_USAGE_VERSION = "actuation.model-usage/v1";

const STANDINGS = new Set([
  "provider-reported", "observed", "normalized-from-native", "derived",
  "estimated", "unavailable", "not-reported",
]);
const OUTCOMES = new Set(["completed", "partial", "failed", "cancelled", "interrupted", "unknown"]);
const PROVIDER_FACT_KEYS = new Set(["service_tier", "speed", "inference_geo"]);

function object(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${name} must be an object`);
  return value;
}

function text(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") throw new TypeError(`${name} must be a non-empty string`);
}

function refs(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string" || entry.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty refs`);
  }
}

function count(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (!Number.isSafeInteger(value) || value < 0) throw new TypeError(`${name} must be a non-negative safe integer`);
}

function standing(value, name) {
  if (!STANDINGS.has(value)) throw new TypeError(`${name} must name a supported evidence standing`);
}

function timestamp(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "" || Number.isNaN(Date.parse(value))) {
    throw new TypeError(`${name} must be an ISO-compatible timestamp string`);
  }
}

function validateIdentity(value, name) {
  const identity = object(value, name);
  standing(identity.standing, `${name}.standing`);
  for (const key of ["ref", "name", "revision", "variant"]) text(identity[key], `${name}.${key}`, { optional: true });
  if (["unavailable", "not-reported"].includes(identity.standing)) {
    if (["ref", "name", "revision", "variant"].some((key) => identity[key] != null)) {
      throw new TypeError(`${name} with ${identity.standing} standing must not claim identity fields`);
    }
  } else if (identity.ref == null && identity.name == null) {
    throw new TypeError(`${name} requires ref or name when identity is available`);
  }
}

function validateMeasured(value, name) {
  const measured = object(value, name);
  standing(measured.standing, `${name}.standing`);
  if (["unavailable", "not-reported"].includes(measured.standing)) {
    if (measured.milliseconds != null) throw new TypeError(`${name} with ${measured.standing} standing must not carry milliseconds`);
  } else if (typeof measured.milliseconds !== "number" || !Number.isFinite(measured.milliseconds) || measured.milliseconds < 0) {
    throw new TypeError(`${name}.milliseconds must be a non-negative finite number when reported`);
  }
}

export function validateModelUsageObservation(input) {
  const observation = object(input, "ModelUsageObservation");
  if (observation.schema !== MODEL_USAGE_VERSION) throw new TypeError(`ModelUsageObservation.schema must equal ${MODEL_USAGE_VERSION}`);
  for (const key of ["usage_ref", "actuation_ref", "invocation_ref"]) text(observation[key], `ModelUsageObservation.${key}`);

  const correlation = object(observation.correlation, "ModelUsageObservation.correlation");
  for (const key of ["activity_ref", "agent_ref", "agency_ref", "agent_session_ref", "harness_ref", "body_ref", "native_session_ref"]) {
    text(correlation[key], `ModelUsageObservation.correlation.${key}`, { optional: true });
  }
  refs(correlation.external_refs, "ModelUsageObservation.correlation.external_refs", { optional: true });

  validateIdentity(observation.provider, "ModelUsageObservation.provider");
  validateIdentity(observation.model, "ModelUsageObservation.model");

  const tokens = object(observation.tokens, "ModelUsageObservation.tokens");
  standing(tokens.standing, "ModelUsageObservation.tokens.standing");
  for (const key of ["input", "output"]) count(tokens[key], `ModelUsageObservation.tokens.${key}`, { optional: true });
  if (["unavailable", "not-reported"].includes(tokens.standing) && (tokens.input != null || tokens.output != null)) {
    throw new TypeError(`ModelUsageObservation.tokens with ${tokens.standing} standing must not carry counts`);
  }

  const cache = object(observation.cache, "ModelUsageObservation.cache");
  standing(cache.standing, "ModelUsageObservation.cache.standing");
  for (const key of ["read_input", "creation_input", "output"]) count(cache[key], `ModelUsageObservation.cache.${key}`, { optional: true });
  if (["unavailable", "not-reported"].includes(cache.standing) && ["read_input", "creation_input", "output"].some((key) => cache[key] != null)) {
    throw new TypeError(`ModelUsageObservation.cache with ${cache.standing} standing must not carry counts`);
  }

  if (observation.usage_classes != null) {
    if (!Array.isArray(observation.usage_classes)) throw new TypeError("ModelUsageObservation.usage_classes must be an array");
    for (const [index, entry] of observation.usage_classes.entries()) {
      object(entry, `ModelUsageObservation.usage_classes[${index}]`);
      text(entry.class, `ModelUsageObservation.usage_classes[${index}].class`);
      count(entry.quantity, `ModelUsageObservation.usage_classes[${index}].quantity`);
      text(entry.unit, `ModelUsageObservation.usage_classes[${index}].unit`);
      standing(entry.standing, `ModelUsageObservation.usage_classes[${index}].standing`);
    }
  }

  const timing = object(observation.timing, "ModelUsageObservation.timing");
  timestamp(timing.started_at, "ModelUsageObservation.timing.started_at", { optional: true });
  timestamp(timing.completed_at, "ModelUsageObservation.timing.completed_at", { optional: true });
  validateMeasured(timing.latency, "ModelUsageObservation.timing.latency");
  if (timing.started_at != null && timing.completed_at != null && Date.parse(timing.completed_at) < Date.parse(timing.started_at)) {
    throw new TypeError("ModelUsageObservation.timing.completed_at must not precede started_at");
  }

  const cost = object(observation.cost, "ModelUsageObservation.cost");
  standing(cost.standing, "ModelUsageObservation.cost.standing");
  if (["unavailable", "not-reported"].includes(cost.standing)) {
    if (cost.amount != null || cost.currency != null || cost.pricing_basis != null) {
      throw new TypeError(`ModelUsageObservation.cost with ${cost.standing} standing must not carry monetary fields`);
    }
  } else {
    if (typeof cost.amount !== "number" || !Number.isFinite(cost.amount) || cost.amount < 0) {
      throw new TypeError("ModelUsageObservation.cost.amount must be a non-negative finite number when supplied");
    }
    text(cost.currency, "ModelUsageObservation.cost.currency");
    if (cost.standing === "derived") {
      const basis = object(cost.pricing_basis, "ModelUsageObservation.cost.pricing_basis");
      text(basis.source_ref, "ModelUsageObservation.cost.pricing_basis.source_ref");
      text(basis.revision, "ModelUsageObservation.cost.pricing_basis.revision");
      timestamp(basis.effective_at, "ModelUsageObservation.cost.pricing_basis.effective_at");
    } else if (cost.pricing_basis != null) {
      throw new TypeError("ModelUsageObservation.cost.pricing_basis is only valid for derived cost");
    }
  }

  const outcome = object(observation.outcome, "ModelUsageObservation.outcome");
  if (!OUTCOMES.has(outcome.state)) throw new TypeError("ModelUsageObservation.outcome.state is unsupported");
  standing(outcome.standing, "ModelUsageObservation.outcome.standing");
  text(outcome.reason, "ModelUsageObservation.outcome.reason", { optional: true });

  const provenance = object(observation.provenance, "ModelUsageObservation.provenance");
  for (const key of ["reporter_ref", "native_event_ref", "native_schema"]) text(provenance[key], `ModelUsageObservation.provenance.${key}`);
  text(provenance.native_request_ref, "ModelUsageObservation.provenance.native_request_ref", { optional: true });
  timestamp(provenance.observed_at, "ModelUsageObservation.provenance.observed_at");
  refs(provenance.raw_evidence_refs, "ModelUsageObservation.provenance.raw_evidence_refs");
  if (provenance.raw_evidence_refs.length === 0) throw new TypeError("ModelUsageObservation.provenance.raw_evidence_refs must retain at least one reconstructable native evidence ref");

  if (observation.provider_facts != null) {
    object(observation.provider_facts, "ModelUsageObservation.provider_facts");
    for (const [key, value] of Object.entries(observation.provider_facts)) {
      if (!PROVIDER_FACT_KEYS.has(key)) throw new TypeError(`ModelUsageObservation.provider_facts.${key} is not an admitted bounded native fact`);
      if (!["string", "number", "boolean"].includes(typeof value) && value !== null) {
        throw new TypeError(`ModelUsageObservation.provider_facts.${key} must be a scalar native fact`);
      }
    }
  }
  return observation;
}

function nativeCount(value, name) {
  if (!Number.isSafeInteger(value) || value < 0) throw new TypeError(`${name} must be a non-negative safe integer`);
  return value;
}

/** Normalize one real Claude Code transcript assistant record without copying message content. */
export function modelUsageFromClaudeCodeTranscript(nativeInput, options = {}) {
  const native = object(nativeInput, "ClaudeCodeTranscriptEvent");
  const message = object(native.message, "ClaudeCodeTranscriptEvent.message");
  if (native.type !== "assistant") throw new TypeError("ClaudeCodeTranscriptEvent.type must equal assistant");
  if (native.isApiErrorMessage === true || message.model === "<synthetic>") throw new TypeError("synthetic Claude Code error rows are not provider usage evidence");
  for (const [value, name] of [[message.id, "message.id"], [message.model, "message.model"], [native.sessionId, "sessionId"], [options.actuation_ref, "actuation_ref"], [options.native_trace_ref, "native_trace_ref"]]) text(value, `ClaudeCodeTranscriptEvent.${name}`);
  timestamp(native.timestamp, "ClaudeCodeTranscriptEvent.timestamp");

  const nativeUsage = message.usage == null ? null : object(message.usage, "ClaudeCodeTranscriptEvent.message.usage");
  const correlation = { harness_ref: "harness:claude-code", native_session_ref: `claude-code:session:${native.sessionId}` };
  for (const key of ["activity_ref", "agent_ref", "agency_ref", "agent_session_ref", "body_ref"]) if (options[key] != null) correlation[key] = options[key];
  if (options.external_refs != null) correlation.external_refs = options.external_refs;

  const hasTokens = nativeUsage != null && (nativeUsage.input_tokens != null || nativeUsage.output_tokens != null);
  const tokens = hasTokens ? {
    standing: "normalized-from-native",
    ...(nativeUsage.input_tokens != null ? { input: nativeCount(nativeUsage.input_tokens, "message.usage.input_tokens") } : {}),
    ...(nativeUsage.output_tokens != null ? { output: nativeCount(nativeUsage.output_tokens, "message.usage.output_tokens") } : {}),
  } : { standing: "not-reported" };
  const hasCache = nativeUsage != null && (nativeUsage.cache_read_input_tokens != null || nativeUsage.cache_creation_input_tokens != null);
  const cache = hasCache ? {
    standing: "normalized-from-native",
    ...(nativeUsage.cache_read_input_tokens != null ? { read_input: nativeCount(nativeUsage.cache_read_input_tokens, "message.usage.cache_read_input_tokens") } : {}),
    ...(nativeUsage.cache_creation_input_tokens != null ? { creation_input: nativeCount(nativeUsage.cache_creation_input_tokens, "message.usage.cache_creation_input_tokens") } : {}),
  } : { standing: "not-reported" };

  const usageClasses = [];
  if (nativeUsage?.server_tool_use != null) {
    object(nativeUsage.server_tool_use, "ClaudeCodeTranscriptEvent.message.usage.server_tool_use");
    for (const [name, quantity] of Object.entries(nativeUsage.server_tool_use)) {
      usageClasses.push({ class: `server_tool_use.${name}`, quantity: nativeCount(quantity, `message.usage.server_tool_use.${name}`), unit: "requests", standing: "normalized-from-native" });
    }
  }
  const providerFacts = {};
  for (const key of ["service_tier", "speed", "inference_geo"]) if (nativeUsage?.[key] != null) providerFacts[key] = nativeUsage[key];

  return validateModelUsageObservation({
    schema: MODEL_USAGE_VERSION,
    usage_ref: `model-usage:claude-code:${message.id}`,
    actuation_ref: options.actuation_ref,
    invocation_ref: options.invocation_ref ?? (native.requestId ? `invocation:claude-code:${native.requestId}` : `invocation:claude-code:${message.id}`),
    correlation,
    provider: { standing: "not-reported" },
    model: { standing: "normalized-from-native", name: message.model, ...(nativeUsage?.service_tier ? { variant: nativeUsage.service_tier } : {}) },
    tokens,
    cache,
    ...(usageClasses.length ? { usage_classes: usageClasses } : {}),
    timing: { completed_at: native.timestamp, latency: { standing: "not-reported" } },
    cost: { standing: "not-reported" },
    outcome: {
      state: message.stop_reason === "max_tokens" ? "partial" : message.stop_reason == null ? "unknown" : "completed",
      standing: "normalized-from-native",
      ...(message.stop_reason ? { reason: message.stop_reason } : {}),
    },
    provenance: {
      reporter_ref: "harness:claude-code",
      native_event_ref: `claude-code:message:${message.id}`,
      ...(native.requestId ? { native_request_ref: `claude-code:request:${native.requestId}` } : {}),
      native_schema: "claude-code.transcript/assistant-message",
      observed_at: native.timestamp,
      raw_evidence_refs: [options.native_trace_ref],
    },
    ...(Object.keys(providerFacts).length ? { provider_facts: providerFacts } : {}),
  });
}

/** Normalize the terminal usage event emitted by a real `codex exec --json` invocation. */
export function modelUsageFromCodexExecEvents(nativeInput, options = {}) {
  const events = Array.isArray(nativeInput) ? nativeInput : object(nativeInput, "CodexExecEvents").events;
  if (!Array.isArray(events) || events.length === 0) throw new TypeError("CodexExecEvents must contain native events");
  const starts = events.filter((event) => event?.type === "thread.started");
  const completions = events.filter((event) => event?.type === "turn.completed");
  if (starts.length !== 1 || completions.length !== 1) {
    throw new TypeError("CodexExecEvents requires exactly one thread.started and one turn.completed event");
  }
  const thread = object(starts[0], "CodexExecEvents.thread.started");
  const completed = object(completions[0], "CodexExecEvents.turn.completed");
  const usage = object(completed.usage, "CodexExecEvents.turn.completed.usage");
  for (const [value, name] of [
    [thread.thread_id, "thread_id"],
    [options.actuation_ref, "actuation_ref"],
    [options.invocation_ref, "invocation_ref"],
    [options.native_trace_ref, "native_trace_ref"],
    [options.observed_at, "observed_at"],
  ]) text(value, `CodexExecEvents.${name}`);
  timestamp(options.observed_at, "CodexExecEvents.observed_at");

  const correlation = {
    harness_ref: "harness:codex",
    native_session_ref: `codex:thread:${thread.thread_id}`,
  };
  for (const key of ["activity_ref", "agent_ref", "agency_ref", "agent_session_ref", "body_ref"]) {
    if (options[key] != null) correlation[key] = options[key];
  }
  if (options.external_refs != null) correlation.external_refs = options.external_refs;

  const input = nativeCount(usage.input_tokens, "turn.completed.usage.input_tokens");
  const output = nativeCount(usage.output_tokens, "turn.completed.usage.output_tokens");
  const cacheRead = nativeCount(usage.cached_input_tokens, "turn.completed.usage.cached_input_tokens");
  const cacheWrite = nativeCount(usage.cache_write_input_tokens, "turn.completed.usage.cache_write_input_tokens");
  const reasoning = nativeCount(usage.reasoning_output_tokens, "turn.completed.usage.reasoning_output_tokens");

  return validateModelUsageObservation({
    schema: MODEL_USAGE_VERSION,
    usage_ref: `model-usage:codex:${options.invocation_ref}`,
    actuation_ref: options.actuation_ref,
    invocation_ref: options.invocation_ref,
    correlation,
    provider: { standing: "not-reported" },
    model: { standing: "not-reported" },
    tokens: { standing: "normalized-from-native", input, output },
    cache: { standing: "normalized-from-native", read_input: cacheRead, creation_input: cacheWrite },
    usage_classes: [{ class: "reasoning_output", quantity: reasoning, unit: "tokens", standing: "normalized-from-native" }],
    timing: { latency: { standing: "not-reported" } },
    cost: { standing: "not-reported" },
    outcome: { state: "completed", standing: "normalized-from-native" },
    provenance: {
      reporter_ref: "harness:codex",
      native_event_ref: `codex:event:${options.invocation_ref}:turn.completed`,
      native_schema: "codex.exec-jsonl/turn.completed",
      observed_at: options.observed_at,
      raw_evidence_refs: [options.native_trace_ref],
    },
  });
}
