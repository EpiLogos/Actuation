// The Actuation harness catalog: every harness this product can detect.
// Adding a harness is one descriptor module in ./harnesses/ plus one import
// here and a CATALOG_REVISION bump — a small, mechanical, LLM-automatable
// step around upstream releases.
import { harnessDescriptor } from "../contracts/harness-detection.mjs";
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

export const CATALOG_REVISION = 1;

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
].map(harnessDescriptor);

export function harnessDescriptors() {
  return structuredClone(DESCRIPTORS);
}

export function harnessDescriptorBySlug(slug) {
  return structuredClone(DESCRIPTORS.find((descriptor) => descriptor.slug === slug));
}
