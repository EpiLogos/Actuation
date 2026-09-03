import test from "node:test";
import assert from "node:assert/strict";
import { executeCommand } from "./actuation.mjs";
import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { ACTUATION_STREAM_VERSION } from "../contracts/actuation-stream.mjs";

const agency = {
  binding: {
    schema: AGENCY_CONTRACT_VERSION,
    binding_ref: "world-binding:test",
    agent_ref: "agent:test",
    agency_ref: "agency:test",
    world_ref: "world:test",
    scope_ref: "scope:test",
  },
  root_scope: {
    schema: AGENCY_CONTRACT_VERSION,
    scope_ref: "scope:test",
    enclosing_world_ref: "world:test",
  },
};

const stream = {
  schema: ACTUATION_STREAM_VERSION,
  stream_ref: "stream:test",
  actuation_ref: "actuation:test",
  agency_ref: "agency:test",
  agent_session_ref: "agent-session:test",
  lifecycle: {
    state: "open",
    started_at: "2026-09-03T10:00:00Z",
  },
  events: [],
  cursor: {
    next_sequence: 1,
    last_sequence: 0,
  },
};

test("version and capabilities expose one maintained native surface", () => {
  assert.equal(executeCommand(["--version"]).stdout, "actuation 0.1.0");
  const result = executeCommand(["capabilities", "--json"]);
  const value = JSON.parse(result.stdout);
  assert.equal(value.contract, "actuation.cli/v1");
  assert.equal(value.executable, "actuation");
  assert.ok(value.commands.includes("agency.read"));
  assert.equal(value.native_contracts.agency, AGENCY_CONTRACT_VERSION);
});

test("agency command enters the native Agency read model", () => {
  const result = executeCommand(["agency", "-", "--json"], {
    stdin: JSON.stringify(agency),
  });
  const value = JSON.parse(result.stdout);
  assert.equal(value.agency_ref, "agency:test");
  assert.equal(value.agent_ref, "agent:test");
  assert.equal(value.root_for_scope, true);
});

test("stream command enters the native ActuationStream read model", () => {
  const result = executeCommand(["stream", "-", "--json"], {
    stdin: JSON.stringify(stream),
  });
  const value = JSON.parse(result.stdout);
  assert.equal(value.stream_ref, "stream:test");
  assert.equal(value.lifecycle.state, "open");
  assert.deepEqual(value.events, []);
});

test("invalid command fails rather than fabricating a fallback", () => {
  assert.throws(() => executeCommand(["unknown"]), /unknown command/);
});
