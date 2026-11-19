// Run on the publisher; only completed files reach the reader's mounted volume.
import {readFile,writeFile} from 'node:fs/promises';
import {basename,join} from 'node:path';
import {pack} from './saved-report.mjs';

const [jsonPath,mdPath,outDir] = process.argv.slice(2);
if (!outDir) throw new Error('Usage: node pack-report.mjs audit.json audit.md output-directory');
const [json,md] = await Promise.all([readFile(jsonPath,'utf8'),readFile(mdPath,'utf8')]);
const result = pack(json,md,basename(jsonPath));
await writeFile(join(outDir,result.name),result.bytes,{flag:'wx',mode:0o600});
console.log(result.name);
