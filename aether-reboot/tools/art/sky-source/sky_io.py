"""Exact original Radiance decoding and linear RGBA16F KTX2 writer."""
import struct

def read_linear_hdr(path):
    """Decode original Radiance scanlines without resizing or changing orientation."""
    import numpy as np
    with path.open('rb') as file:
        assert file.readline().strip() in (b'#?RADIANCE', b'#?RGBE')
        header = []
        while True:
            line = file.readline()
            assert line, 'Truncated HDR header'
            if not line.strip():
                break
            header.append(line.strip())
        assert b'FORMAT=32-bit_rle_rgbe' in header
        dimensions = file.readline().split()
        assert dimensions[0] == b'-Y' and dimensions[2] == b'+X', 'Expected top-left HDR orientation'
        height, width = int(dimensions[1]), int(dimensions[3])
        assert 8 <= width <= 16384 and 1 <= height <= 8192
        pixels = np.empty((height, width, 3), dtype=np.float32)
        for y in range(height):
            prefix = file.read(4)
            assert prefix == bytes((2, 2, width >> 8, width & 255)), 'Expected Radiance scanline RLE'
            row = np.empty((4, width), dtype=np.uint8)
            for c in range(4):
                x = 0
                while x < width:
                    count = file.read(1)
                    assert count and count[0] != 0, 'Truncated/invalid HDR run'
                    count = count[0]
                    if count > 128:
                        length = count - 128
                        value = file.read(1)
                        assert value and x + length <= width
                        row[c, x:x+length] = value[0]
                    else:
                        length = count
                        values = file.read(length)
                        assert len(values) == length and x + length <= width
                        row[c, x:x+length] = np.frombuffer(values, dtype=np.uint8)
                    x += length
            # Radiance RGBE uses an unsigned shared exponent, not an sRGB curve.
            scale = np.exp2(row[3].astype(np.float32) - 136.0)
            scale[row[3] == 0] = 0.0
            pixels[y] = row[:3].T.astype(np.float32) * scale[:, None]
        assert file.read(1) == b'', 'Unexpected trailing HDR bytes'
    assert np.isfinite(pixels).all() and (pixels >= 0).all()
    return pixels

def write_rgba16_ktx2(path, rgba):
    """Package native-resolution linear samples in standard uncompressed KTX2."""
    height, width, channels = rgba.shape
    assert channels == 4 and rgba.dtype.str == '<f2'
    # Khronos basic DFD: RGBSDA, BT.709 primaries, linear transfer, 8-byte texel.
    block = struct.pack('<IHH4B4B8B', 0, 2, 88, 1, 1, 1, 0,
                        0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0)
    for c, channel in enumerate((0, 1, 2, 15)):
        block += struct.pack('<HBB4BII', c*16, 15, 0xC0 | channel,
                             0, 0, 0, 0, 0xBF800000, 0x3F800000)
    dfd = struct.pack('<I', 4+len(block)) + block
    metadata = b''
    for key, value in [('KTXorientation', 'rd'), ('KTXwriter', 'Aether original Cycles render / build_sky.py')]:
        pair = key.encode('ascii') + b'\0' + value.encode('ascii') + b'\0'
        metadata += struct.pack('<I', len(pair)) + pair + b'\0' * (-len(pair) % 4)
    dfd_offset = 104
    kvd_offset = dfd_offset + len(dfd)
    payload_offset = (kvd_offset + len(metadata) + 7) // 8 * 8
    payload_length = width * height * 8
    header = b'\xABKTX 20\xBB\r\n\x1A\n' + struct.pack('<13I2Q',
        97, 2, width, height, 0, 0, 1, 1, 0,
        dfd_offset, len(dfd), kvd_offset, len(metadata), 0, 0)
    assert len(header) == 80 and len(dfd) == 92
    with path.open('wb') as file:
        file.write(header)
        file.write(struct.pack('<3Q', payload_offset, payload_length, payload_length))
        file.write(dfd)
        file.write(metadata)
        file.write(b'\0' * (payload_offset - file.tell()))
        file.write(rgba.tobytes(order='C'))
