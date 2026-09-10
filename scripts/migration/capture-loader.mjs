// Only named public functions in the explicitly inspected JSON-facing modules
// are wrapped. Internal calls, class mechanics, source files and test assertions
// are untouched. The recorder's own tests prove output/error transparency.
import { fileURLToPath } from 'node:url';
import { relative } from 'node:path';
const root = fileURLToPath(new URL('../../', import.meta.url));
const recorder = new URL('./capture-record.mjs', import.meta.url).href;
const extra = new Set([
  'experiments/epistemic-cultivation/records.mjs',
  'experiments/ql-runtime/prime/conditions.mjs',
  'experiments/ql-runtime/prime/evidence.mjs',
  'experiments/ql-runtime/prime/return-contract.mjs',
  'experiments/ql-runtime/prime/source-lock.mjs',
]);
export async function load(url, context, nextLoad) {
  const loaded = await nextLoad(url, context);
  if (!url.startsWith('file:')) return loaded;
  const path = relative(root, fileURLToPath(url)).replaceAll('\\', '/');
  if (!(/^contracts\/[^/]+\.mjs$/.test(path) && !path.endsWith('.test.mjs')) && !extra.has(path)) return loaded;
  // State-bearing I/O is exercised by the separate committed store scenarios.
  if (path === 'contracts/actuation-stream-store.mjs') return loaded;
  const source = Buffer.from(loaded.source).toString('utf8');
  const names = [];
  const transformed = source.replace(/^export function (\w+)\(/gm, (_, name) => {
    names.push(name);
    return `function ${name}(`;
  });
  const wrapper = names.map(name => `const __oracle_${name} = (...args) => __capture(${JSON.stringify(`${path}#${name}`)}, ${name}, args);\nexport { __oracle_${name} as ${name} };`).join('\n');
  return { ...loaded, source: `${transformed}\nimport { captureCall as __capture } from ${JSON.stringify(recorder)};\n${wrapper}\n` };
}
