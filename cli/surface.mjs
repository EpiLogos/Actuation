import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { ACTIVITY_VERSION } from "../contracts/activity.mjs";
import { ACTUATION_STREAM_VERSION } from "../contracts/actuation-stream.mjs";
import { HARNESS_DETECTION_VERSION } from "../contracts/harness-detection.mjs";
import { ACTUATION_INSTANTIATION_VERSION, LEGACY_MODEL_BEARING_SCHEMA } from "../contracts/instantiation.mjs";
import { REALISED_ACTUATION_VERSION } from "../contracts/realised-actuation.mjs";

export const ACTUATION_CLI_VERSION = "0.1.0";
export const ACTUATION_CLI_CONTRACT = "actuation.cli/v1";

export const ACTUATION_CLI_SURFACE = Object.freeze({
  contract: ACTUATION_CLI_CONTRACT,
  product: "actuation",
  executable: "actuation",
  version: ACTUATION_CLI_VERSION,
  commands: Object.freeze([
    "capabilities",
    "contract.list",
    "agency.read",
    "realised.read",
    "stream.read",
    "activity.read",
    "instantiation.read",
    "instantiation.record",
    "harness.detect",
    "verify",
  ]),
  native_contracts: Object.freeze({
    agency: AGENCY_CONTRACT_VERSION,
    realised: REALISED_ACTUATION_VERSION,
    stream: ACTUATION_STREAM_VERSION,
    activity: ACTIVITY_VERSION,
    instantiation: ACTUATION_INSTANTIATION_VERSION,
    model_bearing_legacy: LEGACY_MODEL_BEARING_SCHEMA,
    harness_detection: HARNESS_DETECTION_VERSION,
  }),
});
