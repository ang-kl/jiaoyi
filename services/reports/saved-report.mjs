import {createHash} from 'node:crypto';
import {readdir, readFile, lstat} from 'node:fs/promises';
import {join} from 'node:path';
import MarkdownIt from 'markdown-it';

const maxBytes = 16 * 1024 * 1024;
const archiveName = /^audit-(\d{8}T\d{6}Z)-[a-f0-9]{64}\.json$/;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

export function describe(bundle) {
  if (bundle.version !== 1 || typeof bundle.json !== 'string' || typeof bundle.markdown !== 'string' ||
      !/^[a-zA-Z0-9_-]+\.json$/.test(bundle.filename)) throw new Error('Invalid saved report');
  const data = JSON.parse(bundle.json);
  const title = /^# ([^\r\n]+)/.exec(bundle.markdown)?.[1];
  if (!title || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(Z|[+-]\d{2}:\d{2})$/.test(data.as_of_sgt))
    throw new Error('Saved report requires a title and a zoned cutoff');
  const iso = new Date(data.as_of_sgt).toISOString();
  return {title, as_of: data.as_of_sgt, stamp: iso.replace(/[-:]/g,'').replace('.000',''),
    filename:bundle.filename, raw:bundle.json, markdown:bundle.markdown.substring(bundle.markdown.indexOf('\n')+1)};
}

export function pack(json, markdown, filename) {
  const bundle = {version:1, filename, json, markdown};
  const {stamp} = describe(bundle);
  const bytes = Buffer.from(JSON.stringify(bundle));
  if (bytes.length > maxBytes) throw new Error('Saved report exceeds 16 MiB');
  return {bytes, name:'audit-'+stamp+'-'+sha256(bytes)+'.json'};
}

export async function loadSavedReport(dir, version = null) {
  if (version !== null && !archiveName.test(version)) throw new Error('Invalid report version');
  const files = (await readdir(dir)).filter(name => archiveName.test(name)).sort();
  const name = version ?? files.at(-1);
  if (!name) throw new Error('No published audit');
  if (!version && files.at(-2)?.slice(0,22) === name.slice(0,22)) throw new Error('Conflicting audits for the same cutoff');
  const path = join(dir,name);
  const stat = await lstat(path);
  if (!stat.isFile() || stat.size > maxBytes) throw new Error('Invalid audit file');
  const bytes = await readFile(path);
  const report = describe(JSON.parse(bytes));
  if (name !== 'audit-'+report.stamp+'-'+sha256(bytes)+'.json') throw new Error('Saved audit integrity mismatch');
  if (Date.parse(report.as_of) > Date.now()) throw new Error('Future report cutoff');
  return {...report, downloadUrl:'/report.json?version='+name};
}

// Only the matching, authenticated JSON download is linked. Historical references
// remain readable as text. Embedded HTML and remote images cannot load from the reader.
const md = new MarkdownIt({html:false}).disable('image');
md.renderer.rules.link_open = (tokens, i, _opts, env) =>
  tokens[i].attrGet('href') === env.filename ? '<a href="'+env.downloadUrl+'">' : '<span>';
md.renderer.rules.link_close = (tokens, i, _opts, env) => {
  let start = i - 1;
  while (start >= 0 && tokens[start].type !== 'link_open') start--;
  return tokens[start]?.attrGet('href') === env.filename ? '</a>' : '</span>';
};
export const renderAudit = report => md.render(report.markdown,report);
