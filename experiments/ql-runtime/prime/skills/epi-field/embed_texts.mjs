// epi-field embed — bimba_embed READ-ONLY (no store_for; graph untouched).
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync } from 'node:fs';
const BIMBA = '/Users/admin/Documents/epi/bimba-portable';
const require2 = createRequire(BIMBA + '/package.json');
const { StdioClientTransport } = require2('@modelcontextprotocol/client/stdio');
const { Client } = require2('@modelcontextprotocol/client');
const scope = JSON.parse(readFileSync(process.argv[2], 'utf8')); // {items:[{key,text}]}
const outPath = process.argv[3];
const transport = new StdioClientTransport({
  command: 'bash', args: ['-c', 'set -a && source .env && set +a && exec node dist/main.js'],
  cwd: BIMBA, stderr: 'pipe', env: { ...process.env, BIMBA_MCP_PERMISSIONS: 'bimba:read' },
});
const client = new Client({ name: 'epi-field-embed', version: '1.0.0' }, { versionNegotiation: { mode: 'auto' } });
await client.connect(transport);
const out = {}; let failed = 0;
for (let i = 0; i < scope.items.length; i += 25) {
  const chunk = scope.items.slice(i, i + 25);
  try {
    const res = await client.callTool({ name: 'bimba_embed',
      arguments: { texts: chunk.map((x) => x.text), task_type: 'SEMANTIC_SIMILARITY', dimensions: 3072 } });
    if (res.isError) { failed += chunk.length; continue; }
    const rs = JSON.parse(res.content[0].text)?.data?.results;
    if (!Array.isArray(rs) || rs.length !== chunk.length) { failed += chunk.length; continue; }
    chunk.forEach((x, j) => { out[x.key] = rs[j]?.vector ?? null; });
  } catch { failed += chunk.length; }
}
writeFileSync(outPath, JSON.stringify(out));
console.log(JSON.stringify({ vectors: Object.values(out).filter(Boolean).length, failed }));
process.exit(0);
