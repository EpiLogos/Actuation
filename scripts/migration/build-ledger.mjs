import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { BASE_REVISION, lockOriginalSources } from './source-lock.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const output = process.argv[2];
if (!output) throw new Error('usage: build-ledger.mjs <new-ledger.json>');
const source_blobs = lockOriginalSources(root);
const tuple = (classification, phase, target, reason, disposition = 'replace-after-parity') => ({ classification, phase, target, reason, disposition });
function decide(path) {
  if (path.startsWith('experiments/epistemic-cultivation/')) {
    if (path.endsWith('.json') || path.endsWith('.md')) return tuple('B', 'R6', 'crates/actuation-research/epistemic', 'Epistemic method/package source stays attributable; eight record kinds remain first-class research data, not claims of interior access.', 'retain-or-harmonise-source');
    return tuple('B', 'R6', 'crates/actuation-research/epistemic', 'Generic access, evidence, record validation and immutable persistence do not require JavaScript.');
  }
  if (path.startsWith('experiments/ql-runtime/')) {
    if (path.endsWith('dsh-ui-client.tsx')) return tuple('C', 'R6', path, 'The DSH-native inspection component is a React/TypeScript host UI integration, not an Actuation semantic runtime.', 'retain-tested-specimen');
    if (path.endsWith('/pydantic_bridge.py')) return tuple('C', 'R6', path, 'PydanticAI is a Python-native framework; keep only its bounded host RPC bridge in Python.', 'retain-tested-specimen');
    if (path.includes('/prime/skills/ql-relational/')) return tuple('C', 'R6', path, 'Prime inherits Python-backed Skills into its children; this faculty calls the separately source-locked QL owner and records provenance, without defining QL semantics.', 'retain-tested-specimen');
    if (path.endsWith('/series1/dsh.mjs')) return tuple('C', 'R6', 'experiments/specimens/dsh', 'The DSH JavaScript SDK/plugin/preset/native observer is target-language integration. Generic evidence, fingerprint and policy functions must be extracted into Rust; retaining the SDK bridge does not retain those generic responsibilities.', 'split-native-bridge-from-generic-research');
    if (path.endsWith('/series1/providers.mjs')) return tuple('B', 'R6', 'crates/actuation-research/providers + experiments/specimens/pi', 'Native HTTP/Pydantic dispatch and envelope normalization are generic research plumbing. Only PiAIProvider imports the Pi JS provider SDK and warrants a small C bridge; no generic model-selection authority is introduced.', 'split-Pi-SDK-bridge-and-migrate-generic');
    if (path.includes('/foundation/runtime-contract/') || path.includes('/foundation/classic-runtime/')) return tuple('A', 'R3', 'crates/actuation-runtime', 'LoopRuntime, RuntimeHost carrier dispatch, cancellation, registry and the bounded ordinary acting loop are generic Actuation mechanics already resident in the experiment.', path.endsWith('.md') ? 'retain-provenance-and-point-to-native' : 'replace-after-parity');
    if (path.includes('/foundation/ql-core-runtime/') || path.includes('/deep-ql/formal/')) return tuple('D', 'R6', 'QL-MEF public kernel/formal seam; frozen original revision for provenance', 'This is the historical local QL formal kernel/pairing implementation. QL-MEF now owns formal semantics. Preserve exact source history and its closure/pairing cases, while active Actuation research consumes QL-MEF instead of creating a second Rust QL kernel.', 'freeze-history-and-replace-active-owner-seam');
    if (/\.(md)$/.test(path) && !path.includes('/prime/') && !path.includes('/series1/')) return tuple('D', 'R6', path, 'Authored lineage, frozen foundation decisions and open research questions remain source/provenance; these are not discarded or counted as executed Rust functionality.', 'retain-attributable-research-source');
    if (path.endsWith('/foundation-freeze.json') || path.includes('/evidence/') || path.endsWith('/convergence/dry-runs.json')) return tuple('D', 'R6', path, 'An immutable prior experiment/foundation evidence record must retain its original scope and source identity; it is not re-labelled as current provider evidence.', 'retain-frozen-evidence');
    if (path.includes('/prime/')) return tuple('B', 'R6', 'crates/actuation-research/prime', 'Prime conditions, source locks, RPC driver, workspace/task verification, recursive family evidence and explicit P5 refinement form an active research programme. Generic runner/record plumbing becomes native Rust; the actual Python Skill is separately C.', /\.(json|md)$/.test(path) ? 'retain-data-and-harmonise-native-entry' : 'replace-after-parity');
    if (path.includes('/comparison/series1/')) return tuple('B', 'R6', 'crates/actuation-research/series1', 'Matched conditions, capability/task contracts, live controlled workspace, redaction, held constants, blind review and measurement stay executable research over native Actuation and the QL owner. Node task fixtures may remain benchmark data, not a Node product runtime.', /\.(json|md)$/.test(path) ? 'retain-data-and-harmonise-native-entry' : 'replace-after-parity');
    if (path.includes('/deep-ql/')) return tuple('B', 'R6', 'crates/actuation-research/ql', 'Deep operator/condition comparison, typing corpus, portable traces, conformance and review remain active. Native research orchestrates the QL owner rather than privately reimplementing its formal kernel.', /\.(json|md)$/.test(path) ? 'retain-data-and-harmonise-native-entry' : 'replace-after-parity');
    if (path.includes('/experiments/')) return tuple('B', 'R6', 'crates/actuation-research/hosts', 'These native/Pi/Pydantic baseline ports and policies are deterministic research apparatus, not actual target SDK use. Migrate them into typed native test hosts and preserve the evidence limit.', /\.(json|md)$/.test(path) ? 'retain-data-and-harmonise-native-entry' : 'replace-after-parity');
    if (path.includes('/foundation/')) return tuple('B', 'R6', 'crates/actuation-research/foundation', 'Foundation fixtures, scripted hosts, manifest/trace optics and matched execution are reusable research infrastructure; preserve historical formal expectations separately from current QL-owner conformance.', /\.(json|md)$/.test(path) ? 'retain-data-and-harmonise-native-entry' : 'replace-after-parity');
    throw new Error(`unclassified research file ${path}`);
  }
  if (path.startsWith('contracts/')) {
    let phase, target;
    if (/\/agency(?:[.-]|$)/.test(path) && !path.includes('actualisation')) [phase, target] = ['R2', 'actuation-core'];
    else if (/agency-actualisation|realised/.test(path)) [phase, target] = ['R3', 'actuation-runtime'];
    else if (/actuation-stream|activity|model-usage|request-correlation/.test(path)) [phase, target] = ['R4', 'actuation-stream'];
    else if (path.includes('epistemic')) [phase, target] = ['R6', 'actuation-research'];
    else [phase, target] = ['R5', 'actuation-adapters'];
    return tuple('A', phase, `crates/${target}`, 'The public contract keeps its schema/ref/null/extension and error-status semantics while native domain ownership is introduced. Existing tests are the oracle, not permission to collapse the underlying relations.', path.endsWith('.json') ? 'retain-public-schema' : 'replace-after-parity');
  }
  if (path.startsWith('detection/')) return tuple('A', 'R5', 'crates/actuation-adapters + catalog/', 'Detection records observed target conditions, native kind and unsupported evidence. Descriptor data becomes versioned data; effect execution is native. Resolution/selection remains AIKit.');
  if (path.startsWith('bin/') || path.startsWith('cli/')) return tuple('A', 'R7', 'crates/actuation-cli', 'The served product becomes a native executable over one descriptor table; CLI application/library semantics, JSON envelopes and exit statuses remain executable parity obligations.');
  if (path.startsWith('schemas/')) return tuple('A', 'R7', path, 'Portable public schema data stays source-addressable across the implementation language change.', 'retain-public-schema');
  if (path.startsWith('ProjectCentral/tests/')) return tuple('A', 'R7', 'crates/actuation-cli/tests', 'Repository integrity checks become native test tooling without moving Central authored-source ownership into Actuation.');
  if (path.startsWith('ProjectCentral/')) return tuple(null, 'R9', path, 'Preserve human/governance/wiki/NOW distinctions. Refresh generated account/matrix projections only through the serialized owner-approved path; do not infer human acceptance.', 'preserve-source-reconcile-projections');
  if (path.startsWith('docs/') || path.startsWith('skills/') || path === 'README.md') return tuple(null, 'R7–R9', path, 'Keep constitutional and research meaning; harmonise executable references and current standing, retaining historical evidence explicitly rather than erasing it.', 'preserve-and-harmonise');
  if (path.startsWith('.github/') || path.startsWith('scripts/')) return tuple(null, 'R7–R9', path, 'Build/verification tooling is not the product ontology. Replace Node-native gates with locked Rust gates while preserving shared owner maintenance tooling and branch-required check names.', 'harmonise-build-tooling');
  if (path === 'package.json') return tuple('A', 'R7', 'Cargo.toml + .oi/product.json', 'Node package ceases to be the ordinary product installation. Explicit target-language research specimens retain independent manifests if genuinely required.');
  if (path.startsWith('.oi/')) return tuple(null, 'R7', path, 'The product lifecycle descriptor must identify native targets, native verification and exact artifact provenance.', 'harmonise-native-install');
  if (path === '.gitignore') return tuple(null, 'R2', path, 'Ignore native build output without ignoring authored source.', 'add-native-build-ignore');
  throw new Error(`unclassified repository file ${path}`);
}
const rows = execFileSync('git', ['ls-tree', '-r', BASE_REVISION], { cwd: root, encoding: 'utf8' }).trim().split('\n').map(line => {
  const [metadata, path] = line.split('\t');
  const [mode, type, blob] = metadata.split(' ');
  const bytes = execFileSync('git', ['show', `${BASE_REVISION}:${path}`], { cwd: root, maxBuffer: 8 * 1024 * 1024 });
  const source = bytes.toString('utf8');
  const exported_symbols = /\.(mjs|js|tsx|py)$/.test(path) ? [...source.matchAll(/(?:export\s+(?:async\s+)?(?:function|class|const)|^def|^class|^async def)\s+([A-Za-z_][\w]*)/gm)].map(match => match[1]) : [];
  return { path, mode, type, source_blob: blob, bytes: bytes.length, exported_symbols, ...decide(path), status: 'planned-not-migrated' };
});
writeFileSync(output, JSON.stringify({ schema: 'actuation.migration-ledger/v1', source_revision: BASE_REVISION, source_blobs, phase: 'R0', decisions: 'provisional destination; per-file acceptance is updated only from merged native implementation and evidence', entries: rows }, null, 2) + '\n', { flag: 'wx' });
console.log(`Classified ${rows.length} exact baseline files (${rows.filter(r => r.path.startsWith('experiments/')).length} research files).`);
