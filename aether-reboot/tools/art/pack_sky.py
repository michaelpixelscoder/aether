"""Lossless repacking of the authored RGBE-derived sky into RGB9E5 KTX2.

No resize, tone curve, clipping, quantization tolerance or changed radiance.
Every component must survive exactly, otherwise the output is rejected.
Format references: Khronos EXT_texture_shared_exponent and KTX dfdutils.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import numpy as np

SIGNATURE = b'\xABKTX 20\xBB\r\n\x1A\n'


def pack(rgb):
    assert np.isfinite(rgb).all() and (rgb >= 0).all() and (rgb <= 65408).all()
    largest = rgb.max(axis=-1)
    exponent = np.maximum(-16, np.floor(np.log2(np.maximum(largest, 2.0 ** -24)))).astype(np.int32) + 16
    step = np.exp2(exponent - 24)
    exponent += (np.floor(largest / step + 0.5) == 512).astype(np.int32)
    step = np.exp2(exponent - 24)
    mantissa = np.floor(rgb / step[..., None] + 0.5).astype(np.uint32)
    assert (mantissa <= 511).all() and (exponent <= 31).all()
    return (mantissa[..., 0] | (mantissa[..., 1] << 9) | (mantissa[..., 2] << 18)
            | (exponent.astype(np.uint32) << 27)).astype('<u4')


def unpack(words):
    step = np.exp2((words >> 27).astype(np.int32) - 24)
    return np.stack([(words >> shift) & 511 for shift in (0, 9, 18)], axis=-1) * step[..., None]


def dfd():
    # Six samples: each 9-bit mantissa and its shared 5-bit exponent.
    block = struct.pack('<IHH4B4B8B', 0, 2, 120, 1, 1, 1, 0,
                        0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0)
    for channel in range(3):
        block += struct.pack('<HBB4BII', channel * 9, 8, channel, 0, 0, 0, 0, 0, 8448)
        block += struct.pack('<HBB4BII', 27, 4, channel | 0x20, 0, 0, 0, 0, 15, 31)
    return struct.pack('<I', 124) + block


def digest(file):
    with file.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def convert(source, output, report):
    source, output = source.resolve(), output.resolve()
    assert source != output, 'Keep the independently validated original as input'
    with source.open('rb') as file:
        header = file.read(104)
    assert header[:12] == SIGNATURE
    fields = struct.unpack_from('<13I2Q', header, 12)
    assert fields[:2] == (97, 2) and fields[4:9] == (0, 0, 1, 1, 0)
    width, height = fields[2:4]
    assert (width, height) in ((4096, 2048), (8192, 4096))
    offset, length, unpacked_length = struct.unpack_from('<3Q', header, 80)
    assert length == unpacked_length == width * height * 8
    assert offset + length == source.stat().st_size
    pixels = np.memmap(source, dtype='<f2', mode='r', offset=offset, shape=(height, width, 4))
    metadata = b''
    for key, value in [('KTXorientation', 'rd'), ('KTXwriter', 'Aether lossless RGB9E5 / pack_sky.py')]:
        pair = key.encode() + b'\0' + value.encode() + b'\0'
        metadata += struct.pack('<I', len(pair)) + pair + b'\0' * (-len(pair) % 4)
    descriptor = dfd()
    payload_offset = (104 + len(descriptor) + len(metadata) + 7) // 8 * 8
    payload_length = width * height * 4
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(output.suffix + '.pending')
    with temporary.open('wb') as file:
        file.write(SIGNATURE + struct.pack('<13I2Q', 123, 4, width, height, 0, 0, 1, 1, 0,
                                          104, len(descriptor), 104 + len(descriptor), len(metadata), 0, 0))
        file.write(struct.pack('<3Q', payload_offset, payload_length, payload_length))
        file.write(descriptor)
        file.write(metadata)
        file.write(b'\0' * (payload_offset - file.tell()))
        for y in range(0, height, 32):
            rgba = pixels[y:y + 32].astype(np.float32)
            assert (rgba[..., 3] == 1).all(), 'Only opaque skies can omit alpha'
            rgb = rgba[..., :3]
            words = pack(rgb)
            assert np.array_equal(unpack(words), rgb), 'RGB9E5 would lose authored values; reject'
            file.write(words.tobytes())
    temporary.replace(output)
    proof = {'schema': 1, 'source_sha256': digest(source), 'output_sha256': digest(output),
             'generator_sha256': digest(Path(__file__)), 'dimensions': [width, height],
             'source_format': 'RGBA16Float', 'runtime_format': 'Rgb9e5Ufloat', 'vk_format': 123,
             'source_gpu_bytes': length, 'gpu_bytes': payload_length, 'bytes': output.stat().st_size,
             'components_checked': width * height * 3, 'max_absolute_error': 0,
             'opaque_alpha_exact': True, 'resampled': False, 'lossless': True}
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(proof, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(proof))


def self_test():
    vectors = np.array([[0, 0, 0], [1, .5, .25], [65408, 32768, 0],
                        [2 ** -24, 0, 0], [.01220703125, .025390625, .037109375]], dtype=np.float32)
    words = pack(vectors)
    assert words[0] == 0
    assert words[1] == (16 << 27) | (64 << 18) | (128 << 9) | 256
    assert np.array_equal(unpack(words), vectors)
    for invalid in [np.nan, np.inf, -1, 65504]:
        try:
            pack(np.array([[invalid, 0, 0]], dtype=np.float32))
        except AssertionError:
            continue
        raise AssertionError('Invalid HDR input accepted')
    assert not np.array_equal(unpack(pack(np.array([[1, .001, .0001]]))), [[1, .001, .0001]])
    assert len(dfd()) == 124
    print('RGB9E5 known word, zero, range, tiny values, rejection and loss detection PASS')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--source', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.self_test:
        self_test()
    else:
        assert args.source and args.output and args.report
        convert(args.source, args.output, args.report)
