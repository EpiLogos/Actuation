// Test-only recorder. It never changes the result, thrown error, or product files.
import { appendFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
const destination = process.env.ACTUATION_ORACLE_CAPTURE_DIR;
if (destination) mkdirSync(destination, { recursive: true });
function jsonInput(value, seen = new Set()) {
  if (value === null || ['string', 'boolean'].includes(typeof value)) return true;
  if (typeof value === 'number') return Number.isFinite(value);
  if (typeof value !== 'object' || seen.has(value)) return false;
  if (!Array.isArray(value) && ![Object.prototype, null].includes(Object.getPrototypeOf(value))) return false;
  seen.add(value);
  const valid = Object.values(value).every(item => jsonInput(item, seen));
  seen.delete(value);
  return valid;
}
export function captureCall(operation, callable, args) {
  const before = destination && jsonInput(args) ? JSON.stringify(args) : null;
  function save(result) {
    if (before !== null) appendFileSync(join(destination, `${process.pid}.jsonl`), `${JSON.stringify({ operation, ...(process.env.ACTUATION_ORACLE_CASE_LABEL ? { scenario: process.env.ACTUATION_ORACLE_CASE_LABEL } : {}), args: JSON.parse(before), ...result })}\n`);
  }
  try {
    const value = callable(...args);
    if (value && typeof value.then === 'function') return value;
    if (value !== undefined && jsonInput(value)) {
      const encoded = JSON.stringify(value);
      if (encoded !== undefined) save({ expected: { ok: true, value: JSON.parse(encoded) } });
    }
    return value;
  } catch (error) {
    save({ expected: { ok: false, error: { name: error.name, message: error.message } } });
    throw error;
  }
}
