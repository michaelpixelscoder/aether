"""Measure trilinear cache error against the original analytic cloud union."""
import hashlib
import json
import struct
from pathlib import Path
import numpy as np
from build_cloud_cache import ROOT, CONFIG, union


def sample(data, uv):
    dims = np.array(data.shape[:3][::-1])
    pos = uv*dims-.5
    low = np.floor(pos).astype(np.int32)
    f = pos-low
    result = np.zeros((len(uv), 4), dtype=np.float32)
    for z in range(2):
        for y in range(2):
            for x in range(2):
                bit = np.array([x, y, z])
                ix = np.clip(low+bit, 0, dims-1)
                weight = np.prod(np.where(bit, f, 1-f), axis=-1)
                result += data[ix[:, 2], ix[:, 1], ix[:, 0]]/255*weight[:, None]
    return result


def main():
    manifest = json.loads((ROOT/'assets/atmosphere/cloud-cache-manifest.json').read_text())
    assert hashlib.sha256(CONFIG.read_bytes()).hexdigest() == manifest['config_sha256']
    assert hashlib.sha256((ROOT/manifest['generator']).read_bytes()).hexdigest() == manifest['generator_sha256']
    rng = np.random.default_rng(441003)
    results = []
    for bank, entry in zip(json.loads(CONFIG.read_text()), manifest['banks'], strict=True):
        raw = (ROOT/entry['file']).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == entry['sha256']
        assert raw[:12] == b'\xABKTX 20\xBB\r\n\x1A\n'
        vk, size, width, height, depth, layers, faces, levels, compression = struct.unpack_from('<9I', raw, 12)
        assert [vk, size, layers, faces, levels, compression] == [37, 1, 0, 1, 1, 0]
        assert [width, height, depth] == bank['resolution'] == entry['dimensions']
        offset, length, unpacked = struct.unpack_from('<3Q', raw, 80)
        assert length == unpacked == width*height*depth*4 == entry['gpu_bytes']
        assert len(raw) == offset+length
        data = np.frombuffer(raw, dtype=np.uint8, count=length, offset=offset).reshape((depth, height, width, 4))
        extent = np.array(bank['extent'], dtype=np.float32)
        padding = np.array(bank['padding'], dtype=np.float32)
        assert padding[0] >= 192 and padding[2] >= 192
        cache_size = extent+padding*2
        # Across the full visible volume and several hours, advection never
        # clamps to a repeated edge. Texture and world use identical XYZ axes.
        p = (rng.random((12000, 3), dtype=np.float32)-.5)*extent
        seconds = rng.random(12000, dtype=np.float32)*14400
        p[:, 0] += np.sin(seconds*.00375)*192
        p[:, 2] += np.sin(seconds*.0016145833)*192
        uv = p/cache_size+.5
        assert np.all((uv >= 0) & (uv <= 1))
        reference = union(p+np.array(bank['center'], dtype=np.float32), bank)
        cached = sample(data, uv)
        error = np.abs(reference[:, 0]-cached[:, 0])
        normal = cached[:, 1:]*2-1
        normal /= np.maximum(np.linalg.norm(normal, axis=-1, keepdims=True), 1e-20)
        visible = reference[:, 0] > .2
        degrees = np.degrees(np.arccos(np.clip(np.sum(normal[visible]*reference[visible, 1:], axis=-1), -1, 1)))
        assert np.isfinite(cached).all() and np.isfinite(degrees).all()
        # Macro density may differ at CSG creases. High-frequency erosion is
        # sampled live, so this bound concerns only the much larger cloud body.
        assert np.quantile(error, .99) < .035, (bank['key'], np.quantile(error, .99))
        assert np.mean(error) < .006, (bank['key'], np.mean(error))
        item = {'key':bank['key'], 'samples':len(p), 'density_error_mean':float(error.mean()),
                'density_error_p95_p99_max':np.quantile(error, [.95,.99,1]).tolist(),
                'normal_error_degrees_p50_p95':np.quantile(degrees, [.5,.95]).tolist(),
                'advection_in_bounds':True, 'sha256':entry['sha256']}
        results.append(item)
    report = {'passed':True, 'samples':sum(x['samples'] for x in results), 'gpu_bytes':manifest['gpu_bytes'], 'banks':results}
    (ROOT/'docs/evidence/cloud-cache.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
