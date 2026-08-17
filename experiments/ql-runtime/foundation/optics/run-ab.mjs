#!/usr/bin/env node
import { runABDemo } from '../fixtures/index.js';
import { formatReplay } from './index.js';

const json = process.argv.includes('--json');
const record = await runABDemo();
if (json) {
  process.stdout.write(`${JSON.stringify(record, null, 2)}\n`);
} else {
  process.stdout.write(`CLASSIC\n${formatReplay(record.classic)}\n\nQL CORE\n${formatReplay(record.ql)}\n\nCOMPARISON\n${JSON.stringify(record.comparison, null, 2)}\n`);
}
