import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import test from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

async function readJson(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}

test("the checked-in ProjectCentral register is complete and its NOW return is attributable", async () => {
  const manifest = await readJson("ProjectCentral/project.json");
  assert.deepEqual(manifest, {
    human_source: "ProjectCentral/user",
    project_id: "Actuation",
    schema: "central.project/v1",
    wiki: {
      profile: "okf-wiki/v1",
      source: "ProjectCentral/agents/wiki/wiki.json",
    },
  });

  const wiki = await readJson(manifest.wiki.source);
  const projectSpace = wiki.objects.find(
    (object) => object.object === "space" && object.ref === "central:wiki:project:Actuation",
  );
  assert.ok(projectSpace, "the Actuation WikiSpace must be present");
  assert.deepEqual(projectSpace.parent_space_refs, ["central:wiki:root"]);

  await Promise.all([
    access(path.join(root, "ProjectCentral/agents/governance/repo-content.md")),
    access(path.join(root, "ProjectCentral/agents/governance/repo-structure.md")),
  ]);

  const policy = await readJson("ProjectCentral/now/policy.json");
  assert.equal(policy.schema, "central.project-now.policy/v1");
  assert.deepEqual(policy.carry_statuses, ["active", "waiting", "carried"]);
  assert.deepEqual(policy.remove_statuses, ["resolved", "expired", "promoted"]);
  assert.equal(policy.human_scratch_cleanup, "human-owned-manual");
  assert.equal(policy.day_boundary, "caller-supplied-local-civil-date");

  const promotions = await readJson("ProjectCentral/now/promotions.json");
  assert.deepEqual(promotions, {
    schema: "central.project-now.promotions/v1",
    entries: [],
  });

  const completion = await readJson(
    "ProjectCentral/now/agents/complete-actuation-projectcentral-now-register-2026-09-09.json",
  );
  assert.equal(completion.schema, "central.project-now.handoff/v1");
  assert.equal(completion.provenance, "agent-authored-bounded-return");
  assert.equal(completion.status, "resolved");
  assert.ok(completion.actor);
  assert.ok(completion.source_refs.includes("ProjectCentral/project.json"));
  assert.ok(completion.evidence_refs.includes("https://github.com/EpiLogos/Actuation/issues/1"));
  assert.ok(completion.preserve_refs.includes("https://github.com/EpiLogos/Actuation/pull/41"));
});
