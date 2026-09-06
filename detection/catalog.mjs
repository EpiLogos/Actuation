// The Actuation harness catalog: every harness this product can detect, and
// the capability descriptors declaring what the dispatch-relevant harnesses
// are. Adding a harness is one descriptor module in ./harnesses/ plus one
// import here and a CATALOG_REVISION bump — a small, mechanical,
// LLM-automatable step around upstream releases. Adding a capability
// descriptor is one module in ./capabilities/ (slug-aligned to ./harnesses/),
// one import, and the same revision bump.
import { harnessDescriptor } from "../contracts/harness-detection.mjs";
import { harnessCapability } from "../contracts/harness-capability.mjs";
import claudeCode from "./harnesses/claude-code.mjs";
import codex from "./harnesses/codex.mjs";
import gemini from "./harnesses/gemini.mjs";
import geminiAntigravity from "./harnesses/gemini-antigravity.mjs";
import pi from "./harnesses/pi.mjs";
import hermes from "./harnesses/hermes.mjs";
import hermesAcp from "./harnesses/hermes-acp.mjs";
import grokBot from "./harnesses/grok-bot.mjs";
import ollama from "./harnesses/ollama.mjs";
import openclaw from "./harnesses/openclaw.mjs";
import kimi from "./harnesses/kimi.mjs";
import zcode from "./harnesses/zcode.mjs";
import claudeCodeCapability from "./capabilities/claude-code.mjs";
import codexCapability from "./capabilities/codex.mjs";
import zcodeCapability from "./capabilities/zcode.mjs";

export const CATALOG_REVISION = 3;

const DESCRIPTORS = [
  claudeCode,
  codex,
  gemini,
  geminiAntigravity,
  pi,
  hermes,
  hermesAcp,
  grokBot,
  ollama,
  openclaw,
  kimi,
  zcode,
].map(harnessDescriptor);

const CAPABILITIES = [claudeCodeCapability, codexCapability, zcodeCapability].map(harnessCapability);

export function harnessDescriptors() {
  return structuredClone(DESCRIPTORS);
}

export function harnessDescriptorBySlug(slug) {
  return structuredClone(DESCRIPTORS.find((descriptor) => descriptor.slug === slug));
}

export function capabilityDescriptors() {
  return structuredClone(CAPABILITIES);
}

export function capabilityDescriptorBySlug(slug) {
  return structuredClone(CAPABILITIES.find((descriptor) => descriptor.harness_slug === slug));
}
