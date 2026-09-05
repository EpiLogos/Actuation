// Runtime self-identification over the catalog. Env markers are identity
// evidence read from this process's environment; presence stays with
// detection. One match resolves, more than one is disclosed ambiguity
// (nested harnesses are real), zero is an honest null.
import { harnessSelf } from "../contracts/harness-detection.mjs";
import { realEffects } from "./probes.mjs";
import { runDetection } from "./detect.mjs";

export const SELF_DETECTOR_IMPLEMENTATION = "actuation harness self";

export function resolveSelf({
  descriptors,
  effects = realEffects(),
  env = process.env,
  now = new Date(),
}) {
  const matched = [];
  for (const descriptor of descriptors) {
    const names = descriptor.probe?.env?.any_of ?? [];
    if (names.length === 0) continue;
    const identity = effects.envProbe(names, env);
    if (!identity.ok) continue;
    const markers = Object.keys(identity.matched);
    if (markers.length === 0) continue;
    matched.push({
      slug: descriptor.slug,
      harness_ref: `harness/${descriptor.slug}`,
      markers,
    });
  }
  const detection = runDetection({ descriptors, effects, now });
  const resolved = matched.length === 1
    ? { slug: matched[0].slug, harness_ref: matched[0].harness_ref, markers: matched[0].markers }
    : null;
  return harnessSelf({
    schema: "actuation.harness-detection/v1",
    document: "self",
    self_ref: `self:${now.toISOString()}`,
    observed_at: now.toISOString(),
    catalog_revision: detection.catalog_revision,
    matched,
    resolved,
    ambiguity: matched.length > 1,
    detection_ref: detection.detection_ref,
    detection: {
      states: Object.fromEntries(detection.harnesses.map((entry) => [entry.slug, entry.state])),
    },
  });
}
