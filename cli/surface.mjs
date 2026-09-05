import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { ACTIVITY_VERSION } from "../contracts/activity.mjs";
import { ACTUATION_STREAM_VERSION } from "../contracts/actuation-stream.mjs";
import { HARNESS_DETECTION_VERSION } from "../contracts/harness-detection.mjs";
import { MODEL_BEARING_CONTRACT_VERSION } from "../contracts/model-bearing.mjs";
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
    "model.read",
    "harness.detect",
    "verify",
  ]),
  native_contracts: Object.freeze({
    agency: AGENCY_CONTRACT_VERSION,
    realised: REALISED_ACTUATION_VERSION,
    stream: ACTUATION_STREAM_VERSION,
    activity: ACTIVITY_VERSION,
    model_bearing: MODEL_BEARING_CONTRACT_VERSION,
    harness_detection: HARNESS_DETECTION_VERSION,
  }),
});
