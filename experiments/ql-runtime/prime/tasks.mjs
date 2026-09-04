import fs from 'node:fs/promises';
import path from 'node:path';

import { getTask as getSeries1Task } from '../comparison/series1/tasks.mjs';

async function write(root, relative, content) {
  const target = path.join(root, relative);
  await fs.mkdir(path.dirname(target), { recursive: true });
  await fs.writeFile(target, content, 'utf8');
}

async function exists(root, relative) {
  try {
    await fs.access(path.join(root, relative));
    return true;
  } catch {
    return false;
  }
}

function preserved(before, after, paths) {
  return Object.fromEntries(paths.map((relative) => [relative, before?.[relative] === after?.[relative]]));
}

function allTrue(object) {
  return Object.values(object).every(Boolean);
}

const COMPOSITION_SOURCES = Object.freeze([
  'ground/product-intent.md',
  'runtime/service-contract.md',
  'desktop/client.md',
  'wiki/identity.md',
  'evidence/acceptance.md',
  'archive/old-contract.md'
]);

export const PRIME_TASKS = Object.freeze([
  {
    id: 'PRIME-COMPOSITION-001',
    category: 'recursive-agency-reconstitution',
    prompt: [
      'Investigate why this Project is locally functional but not yet whole-application ready.',
      'This is a mechanical recursive-Agency acceptance task: use at least two differentiated Prime child Agencies for genuinely distinct evidence regions, preserve their returned differences before synthesis, then reconstitute the whole at the root.',
      'Use only the workspace sources. Distinguish current source from archived/superseded source. Do not edit source files.',
      'Create RETURN.md containing: current ground, the decisive cross-source relations, each child finding and what difference it made, the best present determination, unresolved pressure, and the next verification that would change the determination.'
    ].join(' '),
    successConditions: [
      'At least two real Prime child acting loci are used and returned difference is reconstituted by the root.',
      'RETURN.md distinguishes current source from archived material.',
      'RETURN.md connects runtime, desktop, Wiki identity and acceptance evidence rather than listing them independently.',
      'All source files remain byte-identical.'
    ],
    primeAcceptance: { minChildLoci: 2, requireNestedChild: false },
    async setup(root) {
      await write(root, 'ground/product-intent.md', '# Product intent\n\nThe release candidate is ready only when the same stable ResourceRef remains intelligible through CLI, desktop and Wiki navigation. Local service correctness alone is not whole-application readiness.\n');
      await write(root, 'runtime/service-contract.md', '# Current runtime contract\n\nThe accepted service request key is `resourceRef`. `legacyId` was removed from the current request contract after the identity migration. The service tests exercise `resourceRef` and pass.\n');
      await write(root, 'desktop/client.md', '# Desktop client\n\nThe desktop request builder still emits `{ legacyId: selected.id }` when opening the detail Surface. The UI catches a rejected request and presents an empty detail state.\n');
      await write(root, 'wiki/identity.md', '# Wiki identity relation\n\nWiki links and backlinks use the stable `ResourceRef`. The source relation is intentionally independent of one desktop selection id or transport payload field.\n');
      await write(root, 'evidence/acceptance.md', '# Acceptance observation\n\nCLI detail open succeeds for `ResourceRef=resource:atlas`. The same object selected in the desktop produces an empty detail panel. Server logs record `missing required field resourceRef`. No Wiki source or backlink corruption was observed.\n');
      await write(root, 'archive/old-contract.md', '# ARCHIVE — pre-migration request contract\n\nDesktop and service both use `legacyId` as the canonical request identity.\n\nThis file predates the ResourceRef migration and is retained for history only.\n');
    },
    async verify(root, { before, after }) {
      const returnExists = await exists(root, 'RETURN.md');
      const sourcePreservation = preserved(before, after, COMPOSITION_SOURCES);
      return {
        protocol: 'prime-composition-return-and-source-preservation',
        observations: { returnExists, sourcePreservation },
        objective_checks_pass: returnExists && allTrue(sourcePreservation)
      };
    }
  },
  {
    id: 'PRIME-RECURSIVE-001',
    category: 'nested-recursive-agency',
    prompt: [
      'Reconstruct the release relation in this workspace and produce RETURN.md.',
      'This is a deliberate depth-2 mechanical acceptance task. The root must create one child Agency to own the integration diagnosis. That child must, if the live Prime recursion cap admits depth 2, create its own child to verify the evidence/provenance question before returning upward.',
      'The nested child should answer only its bounded evidence question; the integration child must preserve that returned difference; the root must then reconstitute the final determination.',
      'Do not edit source files. Record the observed child lineage/handles in RETURN.md when Prime exposes them.'
    ].join(' '),
    successConditions: [
      'A child-of-child is attempted under the source-locked depth-2 Prime condition and actual observed lineage is retained.',
      'RETURN.md preserves the nested returned difference through child and root reconstitution.',
      'Current and archived claims are distinguished by provenance.',
      'All source files remain byte-identical.'
    ],
    primeAcceptance: { minChildLoci: 2, requireNestedChild: true },
    async setup(root) {
      await write(root, 'world/current.md', '# Current World\n\nThe current accepted Project relation is source → build → packaged Surface → human acceptance. A green source test is one constituent return, not the completion of the whole.\n');
      await write(root, 'build/build-receipt.md', '# Build receipt\n\nCommit `candidate-a` builds successfully and its unit suite is green. The receipt does not claim that the packaged desktop consumed the same artifact.\n');
      await write(root, 'surface/package-receipt.md', '# Package receipt\n\nThe installed desktop bundle reports embedded revision `candidate-prev`. The package job reused a pre-existing bundle because the clean step was skipped.\n');
      await write(root, 'evidence/human-return.md', '# Human return\n\nThe running desktop still shows the behaviour fixed in `candidate-a`. Rebuilding from a clean package directory removes the stale behaviour.\n');
      await write(root, 'archive/early-assumption.md', '# ARCHIVE\n\nA green unit suite is sufficient evidence that the installed desktop is current. This assumption predates package-revision receipts.\n');
    },
    async verify(root, { before, after }) {
      const sources = ['world/current.md', 'build/build-receipt.md', 'surface/package-receipt.md', 'evidence/human-return.md', 'archive/early-assumption.md'];
      const returnExists = await exists(root, 'RETURN.md');
      const sourcePreservation = preserved(before, after, sources);
      return {
        protocol: 'prime-depth2-return-and-source-preservation',
        observations: { returnExists, sourcePreservation },
        objective_checks_pass: returnExists && allTrue(sourcePreservation)
      };
    }
  }
]);

export function getPrimeTask(id) {
  const prime = PRIME_TASKS.find((entry) => entry.id === id);
  return prime ?? getSeries1Task(id);
}
