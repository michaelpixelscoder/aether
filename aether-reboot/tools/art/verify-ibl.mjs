// Validate offline lighting cubemaps. No GPU, npm dependency or image editing.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';

const argument = flag => {
  const i = process.argv.indexOf(flag);
  if (i < 0) return undefined;
  assert(process.argv[i + 1] && !process.argv[i + 1].startsWith('--'), `Missing ${flag}`);
  return process.argv[i + 1];
};
const root = path.resolve(argument('--root') ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..'));
const resolve = file => {
  const value = path.resolve(root, file);
  assert(value.startsWith(root + path.sep), 'Asset path escapes the project');
  return value;
};
const read = file => fs.readFileSync(resolve(file));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const signature = Buffer.from([171,75,84,88,32,50,48,187,13,10,26,10]);

function cube(bytes, size, count) {
  assert(bytes.length >= 80 + count * 24, 'Truncated KTX header/index');
  assert(bytes.subarray(0, 12).equals(signature), 'KTX2 signature');
  assert.deepEqual(Array.from({length:9}, (_, i) => bytes.readUInt32LE(12 + i * 4)),
    [97, 2, size, size, 0, 0, 6, count, 0], 'Linear RGBA16F cube header');
  assert.equal(bytes.readBigUInt64LE(64), 0n);
  assert.equal(bytes.readBigUInt64LE(72), 0n);
  const dfdOffset = bytes.readUInt32LE(48), dfdLength = bytes.readUInt32LE(52);
  const kvdOffset = bytes.readUInt32LE(56), kvdLength = bytes.readUInt32LE(60);
  assert.equal(dfdOffset, 80 + count * 24);
  assert.equal(dfdLength, 92);
  assert.equal(kvdOffset, dfdOffset + dfdLength);
  const dataStart = Math.ceil((kvdOffset + kvdLength) / 8) * 8;
  assert(dataStart <= bytes.length, 'Truncated KTX metadata');
  const dfd = bytes.subarray(dfdOffset, dfdOffset + dfdLength);
  assert.equal(dfd.readUInt32LE(0), 92);
  assert.deepEqual([...dfd.subarray(12, 28)], [1,1,1,0,0,0,0,0,8,0,0,0,0,0,0,0]);
  for (let c = 0; c < 4; c++) {
    assert.equal(dfd.readUInt16LE(28 + c * 16), c * 16);
    assert.equal(dfd[30 + c * 16], 15);
    assert.equal(dfd[31 + c * 16], 0xc0 | [0,1,2,15][c]);
  }
  const ranges = [];
  let gpuBytes = 0, max = 0;
  for (let level = 0; level < count; level++) {
    const sizeAtLevel = Math.max(1, size >> level);
    const offset = Number(bytes.readBigUInt64LE(80 + 24 * level));
    const length = Number(bytes.readBigUInt64LE(88 + 24 * level));
    const unpacked = Number(bytes.readBigUInt64LE(96 + 24 * level));
    assert(Number.isSafeInteger(offset) && Number.isSafeInteger(length));
    assert.equal(length, sizeAtLevel ** 2 * 6 * 8);
    assert.equal(unpacked, length);
    assert(offset % 8 === 0 && offset >= dataStart && offset + length <= bytes.length, 'KTX level bounds');
    for (let i = offset; i < offset + length; i += 8) {
      for (let c = 0; c < 3; c++) {
        const half = bytes.readUInt16LE(i + c * 2);
        const exp = half >> 10;
        assert(exp < 31, 'Non-finite or negative radiance');
        const value = exp ? (1 + (half & 1023) / 1024) * 2 ** (exp - 15) : (half & 1023) * 2 ** -24;
        max = Math.max(max, value);
      }
      assert.equal(bytes.readUInt16LE(i + 6), 0x3c00, 'Cube alpha must be one');
    }
    ranges.push({level, size:sizeAtLevel, offset, bytes:length});
    gpuBytes += length;
  }
  const ordered = [...ranges].sort((a, b) => a.offset - b.offset);
  assert.equal(ordered[0].offset, dataStart);
  for (let i = 1; i < ordered.length; i++) {
    assert.equal(ordered[i-1].offset + ordered[i-1].bytes, ordered[i].offset, 'Overlapping or missing KTX data');
  }
  assert.equal(ordered.at(-1).offset + ordered.at(-1).bytes, bytes.length, 'Trailing KTX data');
  assert(max > 0, 'Black environment');
  return {levels:ranges, gpu_bytes:gpuBytes, radiance_max:max};
}

const folder = 'assets/textures/ibl/';
const manifest = JSON.parse(read(folder + 'ibl-manifest.json'));
assert.equal(manifest.schema, 1);
for (const field of ['source', 'generator', 'decoder']) {
  assert.equal(hash(read(manifest[field])), manifest[field + '_sha256'], `${field} changed`);
}
const skyBytes = read('tools/art/sky-source/manifest.json');
assert.equal(hash(skyBytes), manifest.sky_manifest_sha256, 'Sky manifest changed: rebuild cubemaps');
const sky = JSON.parse(skyBytes), author = sky.ibl_authoring;
assert.equal(author.camera_solar_disk_visible, false);
assert.equal(author.solar_illumination_active, true);
assert.equal(author.view_transform_applied, false);
assert.equal(author.resampled, false);
assert.equal(author.raw_exposure_stops, 0);
assert.equal(author.sha256, manifest.source_sha256);
assert.deepEqual(manifest.source_dimensions, [author.dimensions[1], author.dimensions[0], 3]);
assert.equal(manifest.source_gain, 2 ** author.runtime_reference_exposure_stops);
assert.equal(manifest.diffuse_size, 32);
assert([32,64,128,256,512].includes(manifest.specular_size));
assert(Number.isInteger(manifest.samples) && manifest.samples >= 256 && manifest.samples <= 4096);
const mipCount = Math.log2(manifest.specular_size) + 1;
assert.deepEqual(manifest.specular_roughness, Array.from({length:mipCount}, (_, i) => i / (mipCount - 1)));
assert.deepEqual(manifest.outputs.map(o => o.file).sort(), ['sky-diffuse.ktx2', 'sky-specular.ktx2']);
const outputs = manifest.outputs.map(record => {
  const bytes = read(folder + record.file);
  assert.equal(bytes.length, record.bytes);
  assert.equal(hash(bytes), record.sha256, `Changed ${record.file}`);
  const diffuse = record.file === 'sky-diffuse.ktx2';
  return {...record, ...cube(bytes, diffuse ? 32 : manifest.specular_size, diffuse ? 1 : mipCount)};
});
let negatives = 0;
if (process.argv.includes('--self-test')) {
  const good = read(folder + 'sky-specular.ktx2');
  const reject = edit => {
    const changed = Buffer.from(good);
    assert.throws(() => cube(edit(changed) ?? changed, manifest.specular_size, mipCount));
    negatives++;
  };
  const firstPixel = Number(good.readBigUInt64LE(80));
  reject(b => b.subarray(0, b.length - 1));
  reject(b => { b.writeUInt32LE(1, 36); }); // Not a cube.
  reject(b => { b.writeUInt16LE(0x7c00, firstPixel); }); // Infinity.
  reject(b => { b.writeUInt16LE(0xbc00, firstPixel); }); // Negative radiance.
  reject(b => { b.writeUInt16LE(0, firstPixel + 6); }); // Wrong alpha.
  reject(b => { b.writeBigUInt64LE(b.readBigUInt64LE(104), 80); }); // Overlap.
  reject(b => Buffer.concat([b, Buffer.from([0])]));
}
const report = {schema:1, checked_at:new Date().toISOString(), manifest_sha256:hash(read(folder+'ibl-manifest.json')), outputs, negative_cases:negatives};
const destination = resolve(argument('--report') ?? 'docs/evidence/ibl-assets.json');
fs.mkdirSync(path.dirname(destination), {recursive:true});
fs.writeFileSync(destination, JSON.stringify(report, null, 2) + '\n');
console.log(`IBL: two cubemaps, ${outputs.reduce((n, o) => n + o.levels.length, 0)} mip levels, ${outputs.reduce((n, o) => n + o.gpu_bytes, 0)} GPU bytes; ${negatives} negative cases PASS.`);
