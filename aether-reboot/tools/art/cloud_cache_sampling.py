"""Trilinear RGBA8 cache sampler shared by the independent CPU audits."""
import numpy as np

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

