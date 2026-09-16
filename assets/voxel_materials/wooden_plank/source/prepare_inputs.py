#!/usr/bin/env python3
"""Build the shared wooden-plank geometry/material priors using Pillow only."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter


ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path(__file__).resolve().parent / "albedo_macro_master_v1.png"
OUT = Path(__file__).resolve().parent / "inputs"
OUTPUT_SIZE = 1280
# The AI master is treated as one continuous 5×5 panel. Slice each complete
# horizontal board independently only to establish the exact pixel grid; never
# synthesize atlas cells separately, so grain remains continuous across every
# planned 1+3+1 / 4+1 / 2+2+1 UV split.
PLANK_BOUNDARIES_AT_1254 = (0, 244, 486, 730, 973, 1214)
CELL = OUTPUT_SIZE // 5


def pixel_perfect_frame(image: Image.Image) -> Image.Image:
    """Fit five complete source planks to an exact 5 × 256 px vertical grid."""
    w, h = image.size
    scale = h / 1254.0
    boundaries = [round(value * scale) for value in PLANK_BOUNDARIES_AT_1254]
    bands = []
    for top, bottom in zip(boundaries, boundaries[1:]):
        band = image.crop((0, top, w, bottom)).resize(
            (OUTPUT_SIZE, OUTPUT_SIZE // 5), Image.Resampling.LANCZOS
        )
        bands.append(band)
    framed = Image.new("RGB", (OUTPUT_SIZE, OUTPUT_SIZE))
    for index, band in enumerate(bands):
        framed.paste(band, (0, index * (OUTPUT_SIZE // 5)))
    return framed


def reconcile_periodic_edges(image: Image.Image, width: int = 18) -> Image.Image:
    """Cross-fade paired boundaries symmetrically; first/last pixels become equal."""
    source_mode = image.mode
    values = np.asarray(image, dtype=np.float32).copy()
    if values.ndim == 2:
        values = values[..., None]
    for offset in range(width):
        weight = 1.0 - offset / width
        left = values[:, offset, :].copy()
        right = values[:, -1 - offset, :].copy()
        mean = (left + right) * 0.5
        values[:, offset, :] = left * (1 - weight) + mean * weight
        values[:, -1 - offset, :] = right * (1 - weight) + mean * weight
    for offset in range(width):
        weight = 1.0 - offset / width
        top = values[offset, :, :].copy()
        bottom = values[-1 - offset, :, :].copy()
        mean = (top + bottom) * 0.5
        values[offset, :, :] = top * (1 - weight) + mean * weight
        values[-1 - offset, :, :] = bottom * (1 - weight) + mean * weight
    values = np.clip(np.rint(values), 0, 255).astype(np.uint8)
    if source_mode == "L":
        values = values[..., 0]
    return Image.fromarray(values, mode=source_mode)


def build_macro_atlas(master: Image.Image) -> Image.Image:
    """Pack complete plank templates for the greedy-quad shape vocabulary.

    The packed regions are selected as one texture per greedy quad at runtime.
    They are deliberately not joined to make one larger visual plank: a 1×3
    template is a complete three-voxel plank, not three cropped 1×1 tiles.
    """
    atlas = Image.new("RGB", (OUTPUT_SIZE, OUTPUT_SIZE))

    def plank(row: int, width_cells: int) -> Image.Image:
        source = master.crop((0, row * CELL, OUTPUT_SIZE, (row + 1) * CELL))
        return source.resize((width_cells * CELL, CELL), Image.Resampling.LANCZOS)

    # 1×1, 1×3, 1×1
    atlas.paste(plank(0, 1), (0, 0))
    atlas.paste(plank(1, 3), (CELL, 0))
    atlas.paste(plank(2, 1), (4 * CELL, 0))
    # 1×4, 1×1
    atlas.paste(plank(3, 4), (0, CELL))
    atlas.paste(plank(4, 1), (4 * CELL, CELL))
    # 1×2, 1×2, 1×1
    atlas.paste(plank(0, 2), (0, 2 * CELL))
    atlas.paste(plank(1, 2), (2 * CELL, 2 * CELL))
    atlas.paste(plank(2, 1), (4 * CELL, 2 * CELL))
    # 2×5: two whole plank rows, retained at one texel-per-voxel density.
    atlas.paste(master.crop((0, 3 * CELL, OUTPUT_SIZE, 5 * CELL)), (0, 3 * CELL))
    return atlas


def build_maps(base: Image.Image) -> tuple[Image.Image, Image.Image, Image.Image]:
    w, h = base.size
    gray = base.convert("L")
    blur = gray.filter(ImageFilter.GaussianBlur(radius=max(3, w / 90)))
    gray_values = np.asarray(gray, dtype=np.float32)
    blur_values = np.asarray(blur, dtype=np.float32)
    period = h / 5.0
    yy, xx = np.mgrid[0:h, 0:w]
    seam_mod = np.mod(yy, period)
    seam_distance = np.minimum(seam_mod, period - seam_mod)
    edge_distance = np.minimum(xx, w - 1 - xx)
    seam_profile = np.where(
        seam_distance < h * 0.005,
        0.16,
        np.minimum(0.64, 0.30 + seam_distance / (h * 0.055)),
    )
    edge_profile = np.where(
        edge_distance < w * 0.005,
        0.16,
        np.minimum(0.64, 0.30 + edge_distance / (w * 0.040)),
    )
    structure = np.minimum(seam_profile, edge_profile)
    grain = (gray_values - blur_values) / 255.0

    row = np.mod(yy, period) / period
    y_distance = np.abs(row - 0.5) * period
    x_distance = np.minimum(np.abs(xx - w * 0.052), np.abs(xx - w * 0.948))
    radius = w * 0.016
    nail = np.clip((radius - np.hypot(x_distance, y_distance)) / (radius * 0.28), 0.0, 1.0)

    # Bevy's parallax depth convention is black = closest/top and white =
    # deepest/bottom. The geometric prior is naturally authored as elevation,
    # so invert it before publishing the shared height map.
    height_values = 1.0 - np.maximum(structure + grain * 0.12, 0.82 * nail)
    bevel = structure < 0.56
    wood_rough = 0.69 - grain * 0.10 - bevel * 0.10
    roughness_values = wood_rough * (1 - nail) + 0.27 * nail
    metallic_values = 0.94 * nail

    def grayscale(values: np.ndarray) -> Image.Image:
        return Image.fromarray(np.clip(np.rint(values * 255), 0, 255).astype(np.uint8), mode="L")

    return grayscale(height_values), grayscale(roughness_values), grayscale(metallic_values)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    source_bytes = SOURCE.read_bytes()
    master = pixel_perfect_frame(Image.open(SOURCE).convert("RGB"))
    master.save(OUT / "macro_master.png", optimize=True)
    base = reconcile_periodic_edges(build_macro_atlas(master))
    height, roughness, metallic = build_maps(base)
    maps = {
        "base_color.png": reconcile_periodic_edges(base),
        "height.png": reconcile_periodic_edges(height),
        "roughness.png": reconcile_periodic_edges(roughness),
        "metallic.png": reconcile_periodic_edges(metallic),
    }
    for name, image in maps.items():
        image.save(OUT / name, optimize=True)
    metadata = {
        "source": str(SOURCE.relative_to(ROOT)),
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "size": list(base.size),
        "framing": {
            "output_size": OUTPUT_SIZE,
            "plank_height_px": OUTPUT_SIZE // 5,
            "source_boundaries_at_1254": list(PLANK_BOUNDARIES_AT_1254),
            "discarded_source_rows": [PLANK_BOUNDARIES_AT_1254[-1], 1254],
        },
        "continuous_master": "macro_master.png",
        "base_color_role": "packed greedy-quad template atlas",
        "periodic_blend_width_px": 18,
        "plank_count": 5,
        "macro_voxels": [5, 5],
        "atlas_regions": [
            {"origin": [0, 0], "size": [1, 1]},
            {"origin": [1, 0], "size": [3, 1]},
            {"origin": [4, 0], "size": [1, 1]},
            {"origin": [0, 1], "size": [4, 1]},
            {"origin": [4, 1], "size": [1, 1]},
            {"origin": [0, 2], "size": [2, 1]},
            {"origin": [2, 2], "size": [2, 1]},
            {"origin": [4, 2], "size": [1, 1]},
            {"origin": [0, 3], "size": [5, 2]},
        ],
        "nail_columns_normalized": [0.052, 0.948],
    }
    (OUT / "inputs.json").write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    main()
