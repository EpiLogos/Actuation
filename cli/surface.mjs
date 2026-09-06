// Identity constants for the Actuation CLI. The command list is NOT
// transcribed here: it is derived from the command table in commands.mjs,
// which is the single source of truth for routes, usage and handlers.
// surface ↔ table parity is asserted by cli/actuation.test.mjs.
import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { ACTIVITY_VERSION } from "../contracts/activity.mjs";
import { ACTUATION_STREAM_VERSION } from "../contracts/actuation-stream.mjs";
import { HARNESS_DETECTION_VERSION } from "../contracts/harness-detection.mjs";
import { HARNESS_CAPABILITY_VERSION } from "../contracts/harness-capability.mjs";
import { ACTUATION_INSTANTIATION_VERSION, LEGACY_MODEL_BEARING_SCHEMA } from "../contracts/instantiation.mjs";
import { REALISED_ACTUATION_VERSION } from "../contracts/realised-actuation.mjs";

export const ACTUATION_CLI_VERSION = "0.2.0";
export const ACTUATION_CLI_CONTRACT = "actuation.cli/v1";

export const ACTUATION_CLI_SURFACE = Object.freeze({
  contract: ACTUATION_CLI_CONTRACT,
  product: "actuation",
  executable: "actuation",
  version: ACTUATION_CLI_VERSION,
  native_contracts: Object.freeze({
    agency: AGENCY_CONTRACT_VERSION,
    realised: REALISED_ACTUATION_VERSION,
    stream: ACTUATION_STREAM_VERSION,
    activity: ACTIVITY_VERSION,
    instantiation: ACTUATION_INSTANTIATION_VERSION,
    model_bearing_legacy: LEGACY_MODEL_BEARING_SCHEMA,
    harness_detection: HARNESS_DETECTION_VERSION,
    harness_capability: HARNESS_CAPABILITY_VERSION,
  }),
});
