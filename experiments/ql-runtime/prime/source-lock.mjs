const FULL_GIT_REVISION = /^[0-9a-f]{40}$/;

function requiredSha(value, name) {
  if (typeof value !== 'string' || !FULL_GIT_REVISION.test(value)) {
    throw new Error(`${name} must be a full lowercase Git revision.`);
  }
  return value;
}

export function validateSourceLock(lock) {
  if (!lock || lock.schema !== 'actuation.prime-source-lock/v1') {
    throw new Error('Prime source lock must use actuation.prime-source-lock/v1.');
  }
  requiredSha(lock.actuation?.base_revision, 'actuation.base_revision');
  requiredSha(lock.prime_agent?.release_revision, 'prime_agent.release_revision');
  requiredSha(lock.prime_agent?.observed_main_revision, 'prime_agent.observed_main_revision');
  requiredSha(lock.ql_mef?.accepted_main_revision, 'ql_mef.accepted_main_revision');
  requiredSha(lock.ql_mef?.harmonic_research?.revision, 'ql_mef.harmonic_research.revision');
  requiredSha(lock.ql_mef?.harmonic_research?.accepted_via_revision, 'ql_mef.harmonic_research.accepted_via_revision');
  if (!/^v\d+\.\d+\.\d+$/.test(lock.prime_agent?.release ?? '')) {
    throw new Error('prime_agent.release must be an exact semantic release tag.');
  }
  if (lock.ql_mef.harmonic_research.standing !== 'accepted-main-history-carrier') {
    throw new Error('The harmonic source standing must disclose its accepted-main history status.');
  }
  if (lock.ql_mef.harmonic_research.pull_request_state !== 'closed-unmerged') {
    throw new Error('The original harmonic pull-request state must remain explicit.');
  }
  return lock;
}

export function classifyQlRevision(lock, revision, { harmonicEnabled = false, sourceDirty = false } = {}) {
  validateSourceLock(lock);
  requiredSha(revision, 'observed QL-MEF revision');
  if (sourceDirty) return 'explicit-drift';
  if (revision === lock.ql_mef.accepted_main_revision) {
    return harmonicEnabled ? 'accepted-main-harmonic' : 'accepted-main';
  }
  if (revision === lock.ql_mef.harmonic_research.revision) {
    return 'historical-harmonic-head';
  }
  return 'explicit-drift';
}
