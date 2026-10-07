import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const manifest=JSON.parse(fs.readFileSync(path.join(root,'PROMOTION.json')));
assert.equal(manifest.recipe,'r55-directional-reflected-light');
let size=0;
for(const f of manifest.files) {
 assert(!path.isAbsolute(f.file)&&!f.file.split('/').includes('..'));
 const p=path.join(root,f.file),b=fs.readFileSync(p);
 assert.equal(b.length,f.bytes,f.file);
 assert.equal(crypto.createHash('sha256').update(b).digest('hex'),f.sha256,f.file);
 size+=b.length;
}
assert.equal(size,manifest.bytes);assert.equal(manifest.files.length,manifest.file_count);
console.log(`${manifest.file_count} portable R55 files verified, ${size} bytes; no live-stage dependency.`);
