import { lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

import { validateEpistemicRecord } from './records.mjs';

function fileFor(storeDir, recordRef) {
  return join(resolve(storeDir), `${encodeURIComponent(recordRef)}.json`);
}

function render(record) {
  return `${JSON.stringify(record, null, 2)}\n`;
}

export function persistEpistemicRecord(storeDir, value) {
  const record = validateEpistemicRecord(value);
  mkdirSync(storeDir, { recursive: true });
  const pathname = fileFor(storeDir, record.record_ref);
  const content = render(record);
  try {
    writeFileSync(pathname, content, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
    return { record, deduplicated: false, pathname };
  } catch (error) {
    if (error?.code !== 'EEXIST') throw error;
    const existing = readEpistemicRecord(storeDir, record.record_ref);
    if (render(existing) !== content) throw new TypeError(`conflicting epistemic record_ref ${record.record_ref}`);
    return { record: existing, deduplicated: true, pathname };
  }
}

export function readEpistemicRecord(storeDir, recordRef) {
  const pathname = fileFor(storeDir, recordRef);
  const stat = lstatSync(pathname);
  if (!stat.isFile() || stat.isSymbolicLink()) throw new TypeError(`stored epistemic record is not a regular file: ${recordRef}`);
  const parsed = JSON.parse(readFileSync(pathname, 'utf8'));
  const record = validateEpistemicRecord(parsed);
  if (record.record_ref !== recordRef) throw new TypeError(`stored record_ref does not match requested ${recordRef}`);
  return record;
}

export function listEpistemicRecords(storeDir) {
  return readdirSync(resolve(storeDir))
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((name) => readEpistemicRecord(storeDir, decodeURIComponent(name.slice(0, -'.json'.length))));
}
