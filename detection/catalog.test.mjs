import assert from "node:assert/strict";
import test from "node:test";

import { CATALOG_REVISION, harnessDescriptors } from "./catalog.mjs";

test("every catalog descriptor validates and slugs are unique", () => {
  const descriptors = harnessDescriptors();
  assert.ok(descriptors.length >= 11, "catalog carries the surveyed field");
  const slugs = descriptors.map((descriptor) => descriptor.slug);
  assert.equal(new Set(slugs).size, slugs.length, "no duplicate slugs");
  for (const descriptor of descriptors) {
    assert.equal(descriptor.schema, "actuation.harness-detection/v1");
    assert.equal(descriptor.document, "descriptor");
    assert.ok(descriptor.provenance.catalog_revision <= CATALOG_REVISION);
  }
});
