export const ACTUATION_INSTANTIATION_VERSION = "actuation.instantiation/v1";
// Legacy window: the receipt contract was renamed from actuation.model-bearing/v1.
export const LEGACY_MODEL_BEARING_SCHEMA = "actuation.model-bearing/v1";
import { validateHarnessDetection } from "./harness-detection.mjs";

const PLACEMENTS = new Set(["local", "remote", "distributed", "opaque"]);
const INTERIOR_DEPTHS = new Set([
  "opaque",
  "behavioral",
  "outputs",
  "state-read",
  "state-write",
  "causal-intervention",
  "learning",
]);

function record(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string ref`);
  }
}

function stringArray(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty strings`);
  }
}

function facts(value, name) {
  if (value == null) return;
  record(value, name);
  for (const [key, item] of Object.entries(value)) {
    if (typeof key !== "string" || key.trim() === "") {
      throw new TypeError(`${name} keys must be non-empty strings`);
    }
    if (!["string", "number", "boolean"].includes(typeof item) && item !== null) {
      throw new TypeError(`${name}.${key} must be a scalar provider fact`);
    }
  }
}

export function validateModelRelation(input) {
  const relation = record(input, "ModelRelation");
  if (relation.schema !== ACTUATION_INSTANTIATION_VERSION) {
    throw new TypeError(`ModelRelation.schema must equal ${ACTUATION_INSTANTIATION_VERSION}`);
  }
  ref(relation.model_ref, "ModelRelation.model_ref");
  ref(relation.variant_ref, "ModelRelation.variant_ref", { optional: true });

  if (relation.engine != null) {
    const engine = record(relation.engine, "ModelRelation.engine");
    ref(engine.implementation_ref, "ModelRelation.engine.implementation_ref", { optional: true });
    ref(engine.provider_ref, "ModelRelation.engine.provider_ref", { optional: true });
    facts(engine.facts, "ModelRelation.engine.facts");
  }

  if (relation.material != null) {
    const material = record(relation.material, "ModelRelation.material");
    ref(material.binding_ref, "ModelRelation.material.binding_ref", { optional: true });
    if (material.placement != null && !PLACEMENTS.has(material.placement)) {
      throw new TypeError("ModelRelation.material.placement must be local, remote, distributed or opaque");
    }
    facts(material.facts, "ModelRelation.material.facts");
  }

  const surface = record(relation.inference_surface, "ModelRelation.inference_surface");
  ref(surface.contract_ref, "ModelRelation.inference_surface.contract_ref");
  ref(surface.binding_ref, "ModelRelation.inference_surface.binding_ref", { optional: true });
  facts(surface.facts, "ModelRelation.inference_surface.facts");
  return relation;
}

export function validateModelAccessProfile(input) {
  const access = record(input, "ModelAccessProfile");
  if (access.schema !== ACTUATION_INSTANTIATION_VERSION) {
    throw new TypeError(`ModelAccessProfile.schema must equal ${ACTUATION_INSTANTIATION_VERSION}`);
  }
  const inference = record(access.inference, "ModelAccessProfile.inference");
  stringArray(inference.allowed, "ModelAccessProfile.inference.allowed");
  stringArray(inference.denied, "ModelAccessProfile.inference.denied", { optional: true });
  const control = record(access.control, "ModelAccessProfile.control");
  stringArray(control.allowed, "ModelAccessProfile.control.allowed");
  stringArray(control.denied, "ModelAccessProfile.control.denied", { optional: true });
  const interior = record(access.interior, "ModelAccessProfile.interior");
  if (!INTERIOR_DEPTHS.has(interior.depth)) {
    throw new TypeError(`ModelAccessProfile.interior.depth is not a supported research depth`);
  }
  stringArray(interior.allowed, "ModelAccessProfile.interior.allowed", { optional: true });
  stringArray(interior.denied, "ModelAccessProfile.interior.denied", { optional: true });
  return access;
}

export function validateActuationReceipt(input) {
  const receipt = record(input, "ActuationReceipt");
  if (receipt.schema !== ACTUATION_INSTANTIATION_VERSION) {
    throw new TypeError(`ActuationReceipt.schema must equal ${ACTUATION_INSTANTIATION_VERSION}`);
  }
  ref(receipt.actuation_ref, "ActuationReceipt.actuation_ref");
  ref(receipt.agency_ref, "ActuationReceipt.agency_ref");
  ref(receipt.world_binding_ref, "ActuationReceipt.world_binding_ref");
  ref(receipt.harness_ref, "ActuationReceipt.harness_ref", { optional: true });
  ref(receipt.harness_composition_ref, "ActuationReceipt.harness_composition_ref", { optional: true });
  ref(receipt.agent_session_ref, "ActuationReceipt.agent_session_ref", { optional: true });
  validateModelRelation(receipt.model_relation);
  validateModelAccessProfile(receipt.access_profile);
  stringArray(receipt.bounds_refs, "ActuationReceipt.bounds_refs", { optional: true });
  stringArray(receipt.evidence_refs, "ActuationReceipt.evidence_refs", { optional: true });
  ref(receipt.return_ref, "ActuationReceipt.return_ref", { optional: true });
  if (receipt.experiment != null) {
    const experiment = record(receipt.experiment, "ActuationReceipt.experiment");
    stringArray(experiment.held_constant_refs, "ActuationReceipt.experiment.held_constant_refs", { optional: true });
    if (experiment.variables != null) {
      if (!Array.isArray(experiment.variables)) throw new TypeError("ActuationReceipt.experiment.variables must be an array");
      for (const [index, variable] of experiment.variables.entries()) {
        record(variable, `ActuationReceipt.experiment.variables[${index}]`);
        if (typeof variable.name !== "string" || variable.name.trim() === "") throw new TypeError(`ActuationReceipt.experiment.variables[${index}].name must be non-empty`);
        ref(variable.value_ref, `ActuationReceipt.experiment.variables[${index}].value_ref`, { optional: true });
        facts(variable.facts, `ActuationReceipt.experiment.variables[${index}].facts`);
      }
    }
  }
  if (receipt.observed_at != null && Number.isNaN(Date.parse(receipt.observed_at))) {
    throw new TypeError("ActuationReceipt.observed_at must be an ISO-compatible timestamp");
  }
  return receipt;
}

export function instantiationReceipt(input) {
  // Legacy window: old model-bearing documents are read as instantiation
  // receipts. The schema string is normalised here, loudly, in one place.
  if (input && input.schema === LEGACY_MODEL_BEARING_SCHEMA) {
    const legacy = structuredClone(input);
    legacy.schema = ACTUATION_INSTANTIATION_VERSION;
    if (legacy.model_relation?.schema === LEGACY_MODEL_BEARING_SCHEMA) {
      legacy.model_relation.schema = ACTUATION_INSTANTIATION_VERSION;
    }
    if (legacy.access_profile?.schema === LEGACY_MODEL_BEARING_SCHEMA) {
      legacy.access_profile.schema = ACTUATION_INSTANTIATION_VERSION;
    }
    validateActuationReceipt(legacy);
    return legacy;
  }
  validateActuationReceipt(input);
  return structuredClone(input);
}


/**
 * Bind an instantiation receipt to harness evidence. Refuses a receipt
 * whose harness is not detected-with-receipts in the supplied live record;
 * a receipt with no harness_ref is unattributed and passes untouched.
 * Returns the receipt with detection_ref and harness_receipts stamped.
 */
export function attachDetectionEvidence(receipt, detectionRecord) {
  if (!receipt.harness_ref) {
    return { ...structuredClone(receipt), unattributed: true };
  }
  const slug = receipt.harness_ref.replace(/^harness\//, "");
  const detection = validateHarnessDetection(detectionRecord);
  const entry = detection.harnesses.find((candidate) => candidate.slug === slug);
  if (!entry || entry.state !== "detected" || !entry.receipts?.executable) {
    throw new TypeError(
      `instantiation refused: harness ${receipt.harness_ref} is not detected with receipts in ${detection.detection_ref}`,
    );
  }
  const bound = structuredClone(receipt);
  bound.detection_ref = detection.detection_ref;
  bound.harness_receipts = structuredClone(entry.receipts);
  return bound;
}
