// Capture may read only the exact original implementation. Future parity reads
// the frozen files, never generates new expectations from the implementation.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
export const BASE_REVISION = '1c862c6bf58478adf6842a090214906dd2337001';
export function lockOriginalSources(root) {
  const tree = execFileSync('git', ['ls-tree', '-r', BASE_REVISION], { cwd: root, encoding: 'utf8' });
  const sources = {};
  for (const line of tree.trim().split('\n')) {
    const [metadata, path] = line.split('\t');
    const [, type, sha] = metadata.split(' ');
    if (type !== 'blob' || !/^(bin\/|cli\/|contracts\/|detection\/|experiments\/)/.test(path)) continue;
    const bytes = readFileSync(new URL(path, `file://${root.replace(/\/$/, '')}/`));
    const actual = createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
    assert.equal(actual, sha, `original source changed before capture: ${path}`);
    sources[path] = sha;
  }
  assert.ok(Object.keys(sources).length > 100, 'missing original source tree');
  return sources;
}
