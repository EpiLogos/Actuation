import assert from "node:assert/strict";
import test from "node:test";

import {
  CATALOG_REVISION,
  capabilityDescriptorBySlug,
  capabilityDescriptors,
  harnessDescriptorBySlug,
  harnessDescriptors,
} from "./catalog.mjs";
import { HARNESS_CAPABILITY_VERSION } from "../contracts/harness-capability.mjs";

test("every capability descriptor is schema-valid and catalog-valid", () => {
  const slugs = new Set(harnessDescriptors().map((descriptor) => descriptor.slug));
  for (const capability of capabilityDescriptors()) {
    assert.equal(capability.schema, HARNESS_CAPABILITY_VERSION);
    assert.ok(slugs.has(capability.harness_slug), `${capability.harness_slug} must exist in the detection catalog`);
    assert.ok(Array.isArray(capability.native_events) && capability.native_events.length > 0);
  }
});

test("the dispatch-relevant harnesses carry capability descriptors", () => {
  for (const slug of ["claude-code", "codex", "zcode"]) {
    const capability = capabilityDescriptorBySlug(slug);
    assert.ok(capability, `${slug} must declare a capability descriptor`);
    assert.ok(harnessDescriptorBySlug(slug), `${slug} must remain detectable`);
  }
});

test("every declared event names its native trigger and honest channel", () => {
  for (const capability of capabilityDescriptors()) {
    for (const event of capability.native_events) {
      assert.ok(event.native_name.trim().length > 0, `${capability.harness_slug}.${event.event} needs a native name`);
      assert.ok(event.transport.trim().length > 0, `${capability.harness_slug}.${event.event} needs a transport`);
      assert.equal(
        ["stdout-additional-context", "stdout-plain-text", "exit-code-payload", "none"].includes(event.context_channel),
        true,
        `${capability.harness_slug}.${event.event} context channel must be honest`,
      );
    }
  }
});

test("a deny-and-block declaration is carried by at least one blockable event", () => {
  for (const capability of capabilityDescriptors()) {
    const blockable = capability.native_events.some((event) => event.can_block);
    if (capability.blocking_semantics.kind === "deny-and-block") {
      assert.ok(blockable, `${capability.harness_slug} claims deny-and-block with no blockable event`);
    }
    if (capability.blocking_semantics.kind === "none") {
      assert.equal(blockable, false, `${capability.harness_slug} claims no blocking yet marks events blockable`);
    }
  }
});

test("wake claims stay honest across the catalog", () => {
  for (const capability of capabilityDescriptors()) {
    if (capability.wake_capability.kind === "immediate-wake") {
      assert.ok(
        /listener/i.test(capability.wake_capability.notes ?? ""),
        `${capability.harness_slug} immediate-wake must name its listener`,
      );
    }
  }
  const codex = capabilityDescriptorBySlug("codex");
  assert.equal(codex.wake_capability.kind, "none", "codex must not pretend it can be woken");
  assert.equal(codex.injection_channel.kind, "none", "codex must not pretend an injection channel exists");
});

test("install and uninstall seams are declared and reversible", () => {
  for (const capability of capabilityDescriptors()) {
    for (const seam of [capability.install_seam, capability.uninstall_seam]) {
      assert.ok(seam.config_path.trim().length > 0, `${capability.harness_slug} seam needs a path`);
      assert.equal(seam.preserves_foreign_entries, true, `${capability.harness_slug} must preserve foreign entries`);
    }
  }
});

test("catalog revision moved with the capability addition", () => {
  assert.ok(CATALOG_REVISION >= 3, "capability descriptors require a catalog revision bump");
});
