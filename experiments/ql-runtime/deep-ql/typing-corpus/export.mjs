import { corpus, corpusStats, agreementMetrics } from './corpus.js';

const mode = process.argv.includes('--ndjson') ? 'ndjson' : 'json';
const payload = {
  schema: 'ql-typing-benchmark/0.2',
  review_policy: '../../TYPING-CORPUS-REVIEW-CLARIFICATION-08-14-2026.md',
  stats: corpusStats(),
  metrics: agreementMetrics(),
  records: corpus
};

if (mode === 'ndjson') {
  for (const record of corpus) console.log(JSON.stringify(record));
} else {
  console.log(JSON.stringify(payload, null, 2));
}
