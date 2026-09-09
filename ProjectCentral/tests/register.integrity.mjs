import assert from "node:assert/strict";
import { access, readFile, readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import test from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

async function readJson(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}

test("the checked-in ProjectCentral register has its full ownership floor", async () => {
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
  assert.match(completion.result, /Structural initialization only/);
});

test("the Actuation NOW horizon represents every reconciled live owner carrier", async () => {
  const agentsPath = path.join(root, "ProjectCentral/now/agents");
  const records = await Promise.all(
    (await readdir(agentsPath))
      .filter((name) => name.endsWith(".json"))
      .map((name) => readJson(path.join("ProjectCentral/now/agents", name))),
  );
  const live = records
    .filter((record) => ["active", "waiting", "carried"].includes(record.status))
    .sort((left, right) => left.id.localeCompare(right.id));

  assert.deepEqual(
    live.map(({ id, status }) => ({ id, status })),
    [
      { id: "actuation-wayfinder-programme-2026-09-09", status: "active" },
      { id: "branch-hygiene-report-remains-scheduler-owned-2026-09-09", status: "waiting" },
      { id: "prime-recursive-relational-agency-experiment-2026-09-09", status: "active" },
      { id: "public-determination-and-agency-actualisation-operation-2026-09-09", status: "active" },
      { id: "reinspect-codex-stop-event-capability-evidence-2026-09-09", status: "active" },
      { id: "supply-actuation-intent-and-grant-integration-handoff-2026-09-09", status: "active" },
    ],
  );

  for (const record of live) {
    assert.equal(record.schema, "central.project-now.handoff/v1");
    assert.equal(record.provenance, "agent-authored-bounded-return");
    assert.ok(record.actor);
    assert.ok(record.subject);
    assert.match(record.result, /Next condition:|waiting on the next scheduled inspection/);
    assert.ok(record.evidence_refs?.length > 0);
  }
});
