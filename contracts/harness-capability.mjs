export const HARNESS_CAPABILITY_VERSION = "actuation.harness-capability/v1";

export const KNOWN_EVENTS = new Set([
  "session-start",
  "user-prompt-submit",
  "pre-tool-use",
  "post-tool-use",
  "stop",
  "session-end",
  "pre-compact",
  "notification",
  "custom",
]);

const INJECTION_CHANNELS = new Set([
  "stdout-additional-context",
  "stdout-plain-text",
  "exit-code-payload",
  "none",
]);

const BLOCKING_KINDS = new Set(["deny-and-block", "advisory-only", "none"]);
const WAKE_KINDS = new Set(["immediate-wake", "next-event", "none"]);
const CONFIG_FORMATS = new Set(["json", "jsonc", "toml"]);

function record(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string ref`);
  }
}

function stringArray(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty strings`);
  }
}

function oneOf(set, value, name) {
  if (!set.has(value)) {
    throw new TypeError(`${name} must be one of ${[...set].join(", ")}`);
  }
}

function validateSeam(seam, name) {
  record(seam, name);
  ref(seam.config_path, `${name}.config_path`);
  oneOf(CONFIG_FORMATS, seam.format, `${name}.format`);
  ref(seam.entry_shape, `${name}.entry_shape`);
  ref(seam.ownership_marker, `${name}.ownership_marker`);
  // Uninstall that cannot preserve foreign entries is not admissible: a
  // projection must be reversible without touching native configuration
  // it does not own. The schema pins the const; the validator repeats it
  // so hand-rolled objects fail with a readable message.
  if (seam.preserves_foreign_entries !== true) {
    throw new TypeError(`${name}.preserves_foreign_entries must be true`);
  }
}

/**
 * One declared harness capability. Everything here is a DECLARED fact about
 * what the harness is — native events, injection channel, blocking semantics,
 * wake capability, install/uninstall seams. Presence on a machine is
 * detection's job; how dispatch projections install into the harness is the
 * consumer's job. This contract is the single authority both read.
 */
export function validateHarnessCapability(input) {
  const capability = record(input, "HarnessCapability");
  if (capability.schema !== HARNESS_CAPABILITY_VERSION) {
    throw new TypeError(`HarnessCapability.schema must equal ${HARNESS_CAPABILITY_VERSION}`);
  }
  if (capability.document !== "capability") {
    throw new TypeError('HarnessCapability.document must equal "capability"');
  }
  ref(capability.harness_slug, "HarnessCapability.harness_slug");
  ref(capability.summary, "HarnessCapability.summary", { optional: true });

  const seen = new Set();
  if (!Array.isArray(capability.native_events)) {
    throw new TypeError("HarnessCapability.native_events must be an array");
  }
  for (const event of capability.native_events) {
    const entry = record(event, "HarnessCapability.native_events[]");
    oneOf(KNOWN_EVENTS, entry.event, "native event");
    if (entry.event !== "custom") {
      if (seen.has(entry.event)) {
        throw new TypeError(`HarnessCapability.native_events declares ${entry.event} twice`);
      }
      seen.add(entry.event);
    }
    ref(entry.native_name, "native_event.native_name");
    ref(entry.transport, "native_event.transport");
    if (typeof entry.can_block !== "boolean") {
      throw new TypeError("native_event.can_block must be a boolean");
    }
    oneOf(INJECTION_CHANNELS, entry.context_channel, "native_event.context_channel");
    ref(entry.notes, "native_event.notes", { optional: true });
  }

  const channel = record(capability.injection_channel, "HarnessCapability.injection_channel");
  oneOf(INJECTION_CHANNELS, channel.kind, "injection_channel.kind");
  ref(channel.mechanism, "injection_channel.mechanism");
  ref(channel.notes, "injection_channel.notes", { optional: true });

  const blocking = record(capability.blocking_semantics, "HarnessCapability.blocking_semantics");
  oneOf(BLOCKING_KINDS, blocking.kind, "blocking_semantics.kind");
  ref(blocking.notes, "blocking_semantics.notes", { optional: true });

  const wake = record(capability.wake_capability, "HarnessCapability.wake_capability");
  oneOf(WAKE_KINDS, wake.kind, "wake_capability.kind");
  ref(wake.notes, "wake_capability.notes", { optional: true });
  // Wake honesty: an immediate-wake declaration must point at the listener
  // that makes it true, never be asserted from optimism.
  if (wake.kind === "immediate-wake" && !wake.notes) {
    throw new TypeError("wake_capability.kind immediate-wake requires notes naming the listener that makes it true");
  }

  validateSeam(capability.install_seam, "HarnessCapability.install_seam");
  validateSeam(capability.uninstall_seam, "HarnessCapability.uninstall_seam");

  const provenance = record(capability.provenance, "HarnessCapability.provenance");
  ref(provenance.authored_by, "provenance.authored_by");
  stringArray(provenance.source_refs, "provenance.source_refs");
  if (!Array.isArray(provenance.source_refs) || provenance.source_refs.length === 0) {
    throw new TypeError("provenance.source_refs must cite at least one source");
  }

  return capability;
}

export function harnessCapability(input) {
  return structuredClone(validateHarnessCapability(input));
}
