import { openConjugateCircuit, openChildCircuit } from './operators.js';
export * from './operators.js';
export * from './operator-session.js';

export function createDeepQLRuntimeClass(BaseRuntime) {
  if (typeof BaseRuntime !== 'function') throw new TypeError('BaseRuntime constructor required.');
  return class QLDeepRuntime extends BaseRuntime {
    id = 'ql-deep';
    version = '0.1.0-deep-candidate';
    openConjugate(args) { return openConjugateCircuit(args); }
    openChild(args) { return openChildCircuit(args); }
  };
}
