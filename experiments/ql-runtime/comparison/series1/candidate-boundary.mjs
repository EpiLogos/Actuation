const HUMAN_ONLY_KEY = /^(reviewReference|review_reference|human_reference|includeForHumanReview|expectedAnswer|expected_answer)$/i;

export function assertCandidateBoundary(value, path = '$') {
  if (Array.isArray(value)) {
    value.forEach((entry, index) => assertCandidateBoundary(entry, `${path}[${index}]`));
    return value;
  }
  if (!value || typeof value !== 'object') return value;
  for (const [key, child] of Object.entries(value)) {
    if (HUMAN_ONLY_KEY.test(key)) {
      const error = new Error(`Human-review-only field '${key}' crossed the Series 1 candidate boundary at ${path}.`);
      error.code = 'SERIES1_REVIEW_REFERENCE_LEAK';
      throw error;
    }
    assertCandidateBoundary(child, `${path}.${key}`);
  }
  return value;
}

export function buildCandidateRequest({ task, capabilities, maxSteps, provenance }) {
  return assertCandidateBoundary({
    taskId: task.id,
    input: task.prompt,
    successConditions: task.successConditions,
    capabilities,
    maxSteps,
    provenance
  });
}
