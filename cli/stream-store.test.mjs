import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

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
