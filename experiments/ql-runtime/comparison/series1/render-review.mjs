import fs from 'node:fs/promises';
import { renderMaskedReview, renderMaskMapping, renderUnmaskedReview, reviewEvidenceAssessment } from './review.mjs';

const file = process.argv[2];
const modeIndex = process.argv.indexOf('--mode');
const mode = modeIndex >= 0 ? process.argv[modeIndex + 1] : 'unmasked';

if (!file) {
  console.error('Usage: node render-review.mjs <series1-run.json> [--mode masked|unmasked|mapping|assessment]');
  process.exitCode = 1;
} else {
  try {
    const manifest = JSON.parse(await fs.readFile(file, 'utf8'));
    if (mode === 'masked') process.stdout.write(renderMaskedReview(manifest));
    else if (mode === 'unmasked') process.stdout.write(renderUnmaskedReview(manifest));
    else if (mode === 'mapping') process.stdout.write(renderMaskMapping(manifest));
    else if (mode === 'assessment') process.stdout.write(`${JSON.stringify(reviewEvidenceAssessment(manifest), null, 2)}\n`);
    else throw new Error(`Unknown review mode '${mode}'.`);
  } catch (error) {
    console.error(error.stack ?? error.message ?? String(error));
    process.exitCode = 1;
  }
}
