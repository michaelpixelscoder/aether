"""Bake the existing ellipsoid union, not lighting or high-frequency erosion.

Run with Blender's bundled Python (numpy); no scene or GPU is needed. Runtime
keeps the original 128-step integration and live shadow/noise samples. The cache
stores density and its authored macro normal in a padded, linearly filtered 3D
texture. All six banks share this recipe and cloud-banks.json with the renderer.
"""
import hashlib
import json
import struct
from pathlib import Path
import numpy as np

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'assets/atmosphere'
CONFIG = OUT / 'cloud-banks.json'


def random_cell(x, z):
    with np.errstate(over='ignore'):
        h = x.astype(np.uint32)*np.uint32(1597334677) ^ z.astype(np.uint32)*np.uint32(3812015801)
        h = (h ^ (h >> 16))*np.uint32(2246822519)
        h ^= h >> 13
        values = [h, h*np.uint32(1664525)+np.uint32(1013904223), h*np.uint32(22695477)+np.uint32(1)]
    return np.stack([(v & 16777215).astype(np.float32)/np.float32(16777215) for v in values], axis=-1)


def union(p, bank):
    cells = np.floor(p[..., (0, 2)] / np.float32(480)).astype(np.int32)
    result = np.zeros(p.shape[:-1] + (4,), dtype=np.float32)
    result[..., 2] = 1
    seed = np.float32(bank['shape'][2])
    sx, sz = int(seed*np.float32(713)), int(seed*np.float32(173))
    for z in range(-1, 2):
        for x in range(-1, 2):
            ix, iz = cells[..., 0]+x, cells[..., 1]+z
            rnd = random_cell(ix+sx, iz+sz)
            middle = np.stack(((ix+.5)*480+(rnd[..., 0]-.5)*220,
                               bank['center'][1]-185+rnd[..., 2]*270,
                               (iz+.5)*480+(rnd[..., 1]-.5)*220), axis=-1).astype(np.float32)
            radius = np.stack((255+rnd[..., 0]*115, 165+rnd[..., 0]*180, 255+rnd[..., 1]*125), axis=-1)
            q = (p-middle)/radius
            allowed = np.sum(q*q, axis=-1) <= np.float32(1.70)

            def accumulate(q, radius):
                body = 1-np.sum(q*q, axis=-1)
                update = allowed & (body > result[..., 0])
                normal = q/radius
                normal /= np.maximum(np.linalg.norm(normal, axis=-1, keepdims=True), np.float32(1e-20))
                result[..., 0] = np.where(update, body, result[..., 0])
                result[..., 1:] = np.where(update[..., None], normal, result[..., 1:])

            accumulate(q, radius)
            for lobe in range(1, 4):
                bud = random_cell(ix+lobe*193, iz+lobe*313)
                direction = (bud-.5)*np.array([2, 1.2, 2], dtype=np.float32)+np.array([0, .2, 0], dtype=np.float32)
                direction /= np.maximum(np.linalg.norm(direction, axis=-1, keepdims=True), np.float32(1e-20))
                child_center = middle+direction*radius*.75
                child_radius = radius*(.35+bud[..., 2:3]*.20)
                accumulate((p-child_center)/child_radius, child_radius)
    return result


def ktx(path, data):
    depth, height, width, channels = data.shape
    assert channels == 4 and data.dtype == np.uint8
    block = struct.pack('<IHH4B4B8B', 0, 2, 88, 1, 1, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0)
    for c, channel in enumerate((0, 1, 2, 15)):
        block += struct.pack('<HBB4BII', c*8, 7, channel, 0, 0, 0, 0, 0, 255)
    dfd = struct.pack('<I', len(block)+4)+block
    offset = (104+len(dfd)+7)//8*8
    header = b'\xABKTX 20\xBB\r\n\x1A\n'+struct.pack('<13I2Q', 37, 1, width, height, depth, 0, 1, 1, 0, 104, len(dfd), 0, 0, 0, 0)
    path.write_bytes(header+struct.pack('<3Q', offset, data.nbytes, data.nbytes)+dfd+b'\0'*(offset-104-len(dfd))+data.tobytes())


def main():
    report = {'generator':'tools/art/build_cloud_cache.py', 'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'config_sha256':hashlib.sha256(CONFIG.read_bytes()).hexdigest(), 'format':'RGBA8Unorm 3D',
              'channels':['ellipsoid union', 'normal X encoded', 'normal Y encoded', 'normal Z encoded'], 'banks':[]}
    for bank in json.loads(CONFIG.read_text()):
        nx, ny, nz = bank['resolution']
        size = np.array(bank['extent'], dtype=np.float32)+2*np.array(bank['padding'], dtype=np.float32)
        center = np.array(bank['center'], dtype=np.float32)
        data = np.empty((nz, ny, nx, 4), dtype=np.uint8)
        axes = [(np.arange(n, dtype=np.float32)+.5)/n*size[a]+center[a]-size[a]*.5 for a, n in enumerate((nx, ny, nz))]
        for first in range(0, nz, 8):
            zz, yy, xx = np.meshgrid(axes[2][first:first+8], axes[1], axes[0], indexing='ij')
            field = union(np.stack((xx, yy, zz), axis=-1), bank)
            assert np.isfinite(field).all()
            field[..., 1:] = field[..., 1:]*.5+.5
            data[first:first+8] = np.round(np.clip(field, 0, 1)*255).astype(np.uint8)
        path = OUT / (bank['key']+'-shape.ktx2')
        ktx(path, data)
        item = {'key':bank['key'], 'file':path.relative_to(ROOT).as_posix(), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                'dimensions':[nx, ny, nz], 'world_texel_metres':(size/np.array([nx,ny,nz])).tolist(),
                'gpu_bytes':data.nbytes, 'positive_voxels':int(np.count_nonzero(data[..., 0])), 'finite':True}
        report['banks'].append(item)
        print(json.dumps(item), flush=True)
    report['gpu_bytes'] = sum(b['gpu_bytes'] for b in report['banks'])
    (OUT/'cloud-cache-manifest.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf8')


if __name__ == '__main__':
    main()
