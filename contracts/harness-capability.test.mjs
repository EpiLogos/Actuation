import assert from "node:assert/strict";
import test from "node:test";

import {
  HARNESS_CAPABILITY_VERSION,
  KNOWN_EVENTS,
  harnessCapability,
  validateHarnessCapability,
} from "./harness-capability.mjs";
import schema from "./harness-capability-v1.schema.json" with { type: "json" };

function capability(extra = {}) {
  return {
    schema: HARNESS_CAPABILITY_VERSION,
    document: "capability",
    harness_slug: "test-harness",
    summary: "a fixture capability",
    native_events: [
      {
        event: "pre-tool-use",
        native_name: "PreToolUse",
        transport: "settings-json-hooks-map",
        can_block: true,
        context_channel: "stdout-additional-context",
      },
    ],
    injection_channel: {
      kind: "stdout-additional-context",
      mechanism: "hook outcome additionalContext",
    },
    blocking_semantics: { kind: "deny-and-block" },
    wake_capability: { kind: "none" },
    install_seam: {
      config_path: "~/.test-harness/settings.json",
      format: "json",
      entry_shape: "hooks map command entries",
      ownership_marker: "command resolves to the dispatch executable",
      preserves_foreign_entries: true,
    },
    uninstall_seam: {
      config_path: "~/.test-harness/settings.json",
      format: "json",
      entry_shape: "marker-matched entries removed",
      ownership_marker: "command resolves to the dispatch executable",
      preserves_foreign_entries: true,
    },
    provenance: {
      authored_by: "test",
      source_refs: ["survey:test"],
    },
    ...extra,
  };
}

test("a well-formed capability validates and round-trips", () => {
  const validated = validateHarnessCapability(capability());
  assert.equal(validated.harness_slug, "test-harness");
  const constructed = harnessCapability(capability());
  assert.deepEqual(constructed, capability());
});

test("schema document stays aligned with the validator version", () => {
  assert.equal(schema.$defs.capabilityEntry.properties.schema.const, HARNESS_CAPABILITY_VERSION);
  assert.equal(schema.$defs.capabilityEntry.properties.document.const, "capability");
});

test("wrong schema version or document kind is refused", () => {
  assert.throws(() => validateHarnessCapability(capability({ schema: "actuation.harness-capability/v2" })), /schema must equal/);
  assert.throws(() => validateHarnessCapability(capability({ document: "descriptor" })), /document must equal "capability"/);
});

test("unknown event kinds and duplicate standard events are refused", () => {
  const unknown = capability();
  unknown.native_events[0].event = "mid-tool-use";
  assert.throws(() => validateHarnessCapability(unknown), /must be one of/);

  const duplicate = capability({
    native_events: [
      { event: "stop", native_name: "Stop", transport: "t", can_block: false, context_channel: "none" },
      { event: "stop", native_name: "Stop", transport: "t", can_block: false, context_channel: "none" },
    ],
  });
  assert.throws(() => validateHarnessCapability(duplicate), /declares stop twice/);

  const customTwice = capability({
    native_events: [
      { event: "custom", native_name: "A", transport: "t", can_block: false, context_channel: "none" },
      { event: "custom", native_name: "B", transport: "t", can_block: false, context_channel: "none" },
    ],
  });
  validateHarnessCapability(customTwice); // custom events are not deduplicated
});

test("injection channels, blocking and wake kinds are closed enums", () => {
  const channel = capability();
  channel.injection_channel.kind = "stderr";
  assert.throws(() => validateHarnessCapability(channel), /injection_channel.kind/);

  const blocking = capability();
  blocking.blocking_semantics.kind = "sometimes";
  assert.throws(() => validateHarnessCapability(blocking), /blocking_semantics.kind/);

  const wake = capability();
  wake.wake_capability.kind = "telepathy";
  assert.throws(() => validateHarnessCapability(wake), /wake_capability.kind/);
});

test("seams must preserve foreign entries: irreversibility is inadmissible", () => {
  const install = capability();
  install.install_seam.preserves_foreign_entries = false;
  assert.throws(() => validateHarnessCapability(install), /install_seam.preserves_foreign_entries/);

  const uninstall = capability();
  uninstall.uninstall_seam.preserves_foreign_entries = "yes";
  assert.throws(() => validateHarnessCapability(uninstall), /uninstall_seam.preserves_foreign_entries/);

  for (const key of ["config_path", "format", "entry_shape", "ownership_marker"]) {
    const seam = capability();
    delete seam.install_seam[key];
    assert.throws(() => validateHarnessCapability(seam), new RegExp(`install_seam.${key}`));
  }
});

test("wake honesty: immediate-wake must name the listener that makes it true", () => {
  const wake = capability();
  wake.wake_capability = { kind: "immediate-wake" };
  assert.throws(() => validateHarnessCapability(wake), /immediate-wake requires notes/);

  wake.wake_capability = { kind: "immediate-wake", notes: "a resident listener owns the wake" };
  validateHarnessCapability(wake);
});

test("known event vocabulary matches the dispatcher-facing boundary names", () => {
  for (const event of ["session-start", "user-prompt-submit", "pre-tool-use", "post-tool-use", "stop", "session-end", "pre-compact", "notification"]) {
    assert.ok(KNOWN_EVENTS.has(event), `${event} must stay in the known set`);
  }
});

test("provenance is mandatory: descriptors never arrive anonymous", () => {
  const noProvenance = capability();
  delete noProvenance.provenance;
  assert.throws(() => validateHarnessCapability(noProvenance), /provenance/);

  const noSources = capability();
  noSources.provenance.source_refs = [];
  assert.throws(() => validateHarnessCapability(noSources), /source_refs/);
});

function modelDispatchProvider(extra = {}) {
  return {
    provider_ref: "provider:anthropic",
    selector: { kind: "config-key", name: "model" },
    credential: { required: true, hint: "Anthropic API key or an active subscription session" },
    ...extra,
  };
}

test("model_dispatch is optional: a capability without it still validates", () => {
  const bare = capability();
  assert.equal("model_dispatch" in bare, false, "the fixture itself carries no model_dispatch");
  const validated = validateHarnessCapability(bare);
  assert.equal(validated.model_dispatch, undefined);
});

test("a well-formed model_dispatch block validates", () => {
  const withDispatch = capability({
    model_dispatch: {
      kind: "native-provider-binding",
      providers: [modelDispatchProvider()],
      notes: "single native provider",
    },
  });
  const validated = validateHarnessCapability(withDispatch);
  assert.equal(validated.model_dispatch.kind, "native-provider-binding");
  assert.equal(validated.model_dispatch.providers[0].provider_ref, "provider:anthropic");
  assert.equal(validated.model_dispatch.providers[0].selector.name, "model");
});

test("model_dispatch kind none must carry no providers", () => {
  const contradictory = capability({
    model_dispatch: { kind: "none", providers: [modelDispatchProvider()] },
  });
  assert.throws(() => validateHarnessCapability(contradictory), /model_dispatch.providers must be absent when kind is none/);

  const honestAbsence = capability({ model_dispatch: { kind: "none", notes: "not established" } });
  validateHarnessCapability(honestAbsence);
});

test("model_dispatch native-provider-binding requires at least one provider", () => {
  const empty = capability({ model_dispatch: { kind: "native-provider-binding", providers: [] } });
  assert.throws(() => validateHarnessCapability(empty), /providers must be a non-empty array/);

  const missing = capability({ model_dispatch: { kind: "native-provider-binding" } });
  assert.throws(() => validateHarnessCapability(missing), /providers must be a non-empty array/);
});

test("a required credential with no hint is refused, and a hint on a credential-free path is refused", () => {
  const noHint = capability({
    model_dispatch: {
      kind: "native-provider-binding",
      providers: [modelDispatchProvider({ credential: { required: true } })],
    },
  });
  assert.throws(() => validateHarnessCapability(noHint), /credential.hint/);

  const strayHint = capability({
    model_dispatch: {
      kind: "native-provider-binding",
      providers: [modelDispatchProvider({ credential: { required: false, hint: "not actually needed" } })],
    },
  });
  assert.throws(() => validateHarnessCapability(strayHint), /credential.hint must be absent/);

  const noCredentialNeeded = capability({
    model_dispatch: {
      kind: "native-provider-binding",
      providers: [modelDispatchProvider({ credential: { required: false } })],
    },
  });
  validateHarnessCapability(noCredentialNeeded);
});

test("an unknown model_dispatch kind or selector kind is refused", () => {
  const badKind = capability({ model_dispatch: { kind: "telepathic-binding" } });
  assert.throws(() => validateHarnessCapability(badKind), /model_dispatch.kind/);

  const badSelector = capability({
    model_dispatch: {
      kind: "native-provider-binding",
      providers: [modelDispatchProvider({ selector: { kind: "carrier-pigeon", name: "model" } })],
    },
  });
  assert.throws(() => validateHarnessCapability(badSelector), /selector.kind/);
});

test("schema document admits the optional model_dispatch block on the closed capability shape", () => {
  assert.ok(schema.$defs.capabilityEntry.properties.model_dispatch, "the closed schema must declare model_dispatch or valid documents fail");
  assert.equal(schema.$defs.capabilityEntry.required.includes("model_dispatch"), false, "model_dispatch stays optional");
});
