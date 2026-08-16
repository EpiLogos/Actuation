#!/usr/bin/env node
import { runFoundationGate } from './index.js';

const evidence = await runFoundationGate();
process.stdout.write(`${JSON.stringify(evidence, null, 2)}\n`);
if (!evidence.passed) process.exitCode = 1;
