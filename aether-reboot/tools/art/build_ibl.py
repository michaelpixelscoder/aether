"""Deterministic, offline Lambert/GGX filtering of the authored linear sky.

Research: KhronosGroup/glTF-IBL-Sampler and Bevy 0.19.1 environment_map.wgsl.
This is lighting convolution/format conversion, not a retouched sky render.
The source MUST omit the camera-visible solar disc while retaining its actual
illumination of the volumes, to avoid duplicating the shadow-casting key.
"""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import struct
import time

import numpy as np


def unit(vector):
    return vector / np.linalg.norm(vector, axis=-1, keepdims=True).clip(1e-12)


def cube_directions(size):
    """WebGPU cube faces +X,-X,+Y,-Y,+Z,-Z, then Bevy's world Z flip."""
    v, u = np.meshgrid((np.arange(size) + .5) / size * 2 - 1,
                       (np.arange(size) + .5) / size * 2 - 1, indexing='ij')
    one = np.ones_like(u)
    faces = [(one, -v, -u), (-one, -v, u), (u, one, v),
             (u, -one, -v), (u, -v, one), (-u, -v, -one)]
    result = unit(np.stack([np.stack(face, -1) for face in faces])).astype(np.float32)
    result[..., 2] *= -1
    return result


def panorama_sample(pixels, direction):
    """Exact world->UV convention of art.rs Rx(-pi/2), sky.wgsl U=.75-U."""
    height, width = pixels.shape[:2]
    u = (.75 - np.arctan2(-direction[..., 2], direction[..., 0]) / (2 * np.pi)) % 1
    v = np.arccos(direction[..., 1].clip(-1, 1)) / np.pi
    x, y = u * width - .5, (v * height - .5).clip(0, height - 1)
    ix, iy = np.floor(x).astype(np.int32), np.floor(y).astype(np.int32)
    fx, fy = (x - ix)[..., None], (y - iy)[..., None]
    a = pixels[iy, ix % width] * (1 - fx) + pixels[iy, (ix + 1) % width] * fx
    iy2 = (iy + 1).clip(0, height - 1)
    b = pixels[iy2, ix % width] * (1 - fx) + pixels[iy2, (ix + 1) % width] * fx
    return a * (1 - fy) + b * fy


def hammersley(count):
    bits = np.arange(count, dtype=np.uint32)
    bits = (bits << 16) | (bits >> 16)
    for shift, mask in [(1, 0x55555555), (2, 0x33333333), (4, 0x0F0F0F0F), (8, 0x00FF00FF)]:
        bits = ((bits & mask) << shift) | ((bits >> shift) & mask)
    return (np.arange(count) + .5) / count, bits.astype(np.float64) / 2**32


def convolve(pixels, normals, count, roughness=None):
    if roughness == 0:
        return panorama_sample(pixels, normals).astype(np.float32)
    x, y = hammersley(count)
    phi = 2 * np.pi * y
    if roughness is None:
        cos_theta = np.sqrt(1 - x)  # cosine-weighted Lambert samples
    else:
        alpha = roughness ** 2
        cos_theta = np.sqrt((1 - x) / (1 + (alpha * alpha - 1) * x))
    sin_theta = np.sqrt(1 - cos_theta * cos_theta)
    sample = np.stack((np.cos(phi) * sin_theta, np.sin(phi) * sin_theta, cos_theta), -1).astype(np.float32)
    flat = normals.reshape(-1, 3)
    result = np.empty_like(flat)
    for first in range(0, len(flat), 256):
        n = flat[first:first + 256]
        up = np.zeros_like(n)
        up[:, 1] = 1
        up[np.abs(n[:, 1]) > .98] = (1, 0, 0)
        tangent = unit(np.cross(up, n))
        bitangent = np.cross(n, tangent)
        h = (tangent[:, None] * sample[None, :, 0, None]
             + bitangent[:, None] * sample[None, :, 1, None]
             + n[:, None] * sample[None, :, 2, None])
        if roughness is None:
            result[first:first + len(n)] = panorama_sample(pixels, h).mean(1)
        else:
            # Split sum prefilter with V=N; the BRDF LUT remains Bevy's own.
            l = 2 * np.sum(n[:, None] * h, axis=-1, keepdims=True) * h - n[:, None]
            weight = np.sum(n[:, None] * l, -1).clip(0)[..., None]
            result[first:first + len(n)] = (panorama_sample(pixels, l) * weight).sum(1) / weight.sum(1).clip(1e-12)
    return result.reshape(normals.shape)


def write_cube(path, levels):
    """Linear RGBA16Float, six faces, real GGX roughness mips in KTX2."""
    payloads = []
    for index, rgb in enumerate(levels):
        size = max(1, levels[0].shape[1] >> index)
        assert rgb.shape == (6, size, size, 3)
        assert np.isfinite(rgb).all() and rgb.min() >= 0 and rgb.max() < 65504
        rgba = np.ones((*rgb.shape[:-1], 4), dtype='<f2')
        rgba[..., :3] = rgb
        payloads.append(rgba.tobytes())
    block = struct.pack('<IHH4B4B8B', 0, 2, 88, 1, 1, 1, 0,
                        0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0)
    for c, channel in enumerate((0, 1, 2, 15)):
        block += struct.pack('<HBB4BII', c * 16, 15, 0xC0 | channel,
                             0, 0, 0, 0, 0xBF800000, 0x3F800000)
    dfd = struct.pack('<I', 4 + len(block)) + block
    kvd = b''
    for key, value in [('KTXorientation', 'rd'), ('KTXwriter', 'Aether offline Lambert GGX / build_ibl.py')]:
        pair = key.encode() + b'\0' + value.encode() + b'\0'
        kvd += struct.pack('<I', len(pair)) + pair + b'\0' * (-len(pair) % 4)
    dfd_offset = 80 + 24 * len(levels)
    kvd_offset = dfd_offset + len(dfd)
    offset = (kvd_offset + len(kvd) + 7) // 8 * 8
    indices = [None] * len(levels)
    for index in reversed(range(len(levels))):
        indices[index] = (offset, len(payloads[index]), len(payloads[index]))
        offset += len(payloads[index])
    header = b'\xABKTX 20\xBB\r\n\x1A\n' + struct.pack('<13I2Q',
        97, 2, levels[0].shape[1], levels[0].shape[2], 0, 0, 6, len(levels), 0,
        dfd_offset, len(dfd), kvd_offset, len(kvd), 0, 0)
    with path.open('wb') as output:
        output.write(header)
        for index in indices:
            output.write(struct.pack('<3Q', *index))
        output.write(dfd)
        output.write(kvd)
        output.write(b'\0' * (indices[-1][0] - output.tell()))
        for payload in reversed(payloads):
            output.write(payload)


def self_test():
    # Analytic world-direction panorama catches all six rotations and Z signs.
    h, w = 256, 512
    v, u = np.meshgrid((np.arange(h) + .5) / h, (np.arange(w) + .5) / w, indexing='ij')
    theta, phi = np.pi * v, 2 * np.pi * (.75 - u)
    field = np.stack((np.sin(theta) * np.cos(phi), np.cos(theta), -np.sin(theta) * np.sin(phi)), -1)
    pixels = ((field + 1) * .5).astype(np.float32)
    directions = cube_directions(8)
    np.testing.assert_allclose(panorama_sample(pixels, directions), (directions + 1) * .5, atol=.00005)
    constant = np.ones_like(pixels) * (2, 4, 8)
    for roughness in [None, 0, .25, 1]:
        result = convolve(constant, directions, 256, roughness)
        np.testing.assert_allclose(result, np.broadcast_to((2, 4, 8), result.shape), atol=.00001)
    # Integral of a linear directional field under cosine sampling is 2N/3.
    result = convolve(pixels, directions, 4096)
    error = np.max(np.abs(result - (.5 + directions / 3)))
    assert error < .0004, error
    print(json.dumps({'self_test': 'PASS', 'directional_lambert_max_error': float(error)}), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--samples', type=int, default=1024)
    parser.add_argument('--size', type=int, default=256)
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    assert args.size >= 32 and args.size <= 512 and args.size & (args.size - 1) == 0
    assert 256 <= args.samples <= 4096
    root = args.root.resolve()
    source = root / 'tools/art/sky-source/sky-world-ibl-linear.hdr'
    decoder_path = root / 'tools/art/sky-source/sky_io.py'
    sky_manifest_path = root / 'tools/art/sky-source/manifest.json'
    output = root / 'assets/textures/ibl'
    sky = json.loads(sky_manifest_path.read_text(encoding='utf-8'))
    author = sky['ibl_authoring']
    assert author['camera_solar_disk_visible'] is False, 'IBL must omit the direct camera disk'
    assert author['solar_illumination_active'] is True, 'Clouds must retain the source illumination'
    assert author['view_transform_applied'] is False and author['resampled'] is False
    assert author['raw_exposure_stops'] == 0
    source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    assert source_hash == author['sha256'], 'IBL source changed since the original render'
    assert source.stat().st_size == author['bytes']
    gain = 2 ** float(author['runtime_reference_exposure_stops'])
    assert math.isfinite(gain) and gain > 0
    started = time.perf_counter()
    spec = importlib.util.spec_from_file_location('sky_io', decoder_path)
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)
    pixels = decoder.read_linear_hdr(source) * gain
    assert list(pixels.shape[:2][::-1]) == author['dimensions']
    output.mkdir(parents=True, exist_ok=True)
    diffuse = convolve(pixels, cube_directions(32), args.samples)
    write_cube(output / 'sky-diffuse.ktx2', [diffuse])
    mip_count = int(math.log2(args.size)) + 1
    levels = []
    for level in range(mip_count):
        roughness = level / (mip_count - 1)
        values = convolve(pixels, cube_directions(max(1, args.size >> level)), args.samples, roughness)
        levels.append(values)
        print(f'GGX mip {level}/{mip_count - 1}, roughness={roughness:.3f}', flush=True)
    write_cube(output / 'sky-specular.ktx2', levels)
    outputs = []
    for path in sorted(output.glob('sky-*.ktx2')):
        outputs.append({'file': path.name, 'bytes': path.stat().st_size,
                        'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    report = {'schema': 1, 'source': source.relative_to(root).as_posix(), 'source_sha256': source_hash,
              'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'generator': 'tools/art/build_ibl.py',
              'decoder': decoder_path.relative_to(root).as_posix(),
              'decoder_sha256': hashlib.sha256(decoder_path.read_bytes()).hexdigest(),
              'sky_manifest_sha256': hashlib.sha256(sky_manifest_path.read_bytes()).hexdigest(),
              'source_dimensions': list(pixels.shape), 'source_gain': gain,
              'samples': args.samples, 'diffuse_size': 32, 'specular_size': args.size,
              'specular_roughness': [i / (mip_count - 1) for i in range(mip_count)],
              'radiance_max': float(pixels.max()), 'elapsed_seconds': time.perf_counter() - started,
              'outputs': outputs, 'kind': 'derived lighting convolution; not an original sky render'}
    (output / 'ibl-manifest.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
