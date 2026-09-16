#!/usr/bin/env python3
"""Validate periodicity and cross-map coherence, then make a review sheet."""

from __future__ import annotations

import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageStat


HERE = Path(__file__).resolve().parent
TEXTURES = HERE / "textures"
PREVIEWS = HERE / "previews"
VALIDATION = HERE / "validation"


def edge_mae(image: Image.Image) -> dict[str, float]:
    rgb = image.convert("RGB")
    w, h = rgb.size
    p = rgb.load()
    horizontal = sum(abs(a - b) for y in range(h) for a, b in zip(p[0, y], p[w - 1, y])) / (h * 3)
    vertical = sum(abs(a - b) for x in range(w) for a, b in zip(p[x, 0], p[x, h - 1])) / (w * 3)
    return {"left_right_mae": horizontal, "top_bottom_mae": vertical}


def normal_length_error(image: Image.Image) -> float:
    p = image.convert("RGB").load()
    w, h = image.size
    errors = []
    for y in range(0, h, 4):
        for x in range(0, w, 4):
            nx, ny, nz = ((channel / 255.0) * 2 - 1 for channel in p[x, y])
            errors.append(abs(math.sqrt(nx * nx + ny * ny + nz * nz) - 1.0))
    return sum(errors) / len(errors)


def mean_crop(image: Image.Image, box: tuple[int, int, int, int], channel: int = 0) -> float:
    crop = image.convert("RGB").crop(box).split()[channel]
    return ImageStat.Stat(crop).mean[0]


def row_mean(image: Image.Image, y: int) -> float:
    w, _ = image.size
    return mean_crop(image, (round(w * 0.12), y, round(w * 0.88), y + 1))


def make_contact_sheet(images: dict[str, Image.Image]) -> None:
    PREVIEWS.mkdir(parents=True, exist_ok=True)
    thumb = 330
    label = 28
    sheet = Image.new("RGB", (thumb * 3, (thumb + label) * 2), (24, 24, 27))
    draw = ImageDraw.Draw(sheet)
    for index, (name, source) in enumerate(images.items()):
        x = (index % 3) * thumb
        y = (index // 3) * (thumb + label)
        image = source.convert("RGB").resize((thumb, thumb), Image.Resampling.LANCZOS)
        sheet.paste(image, (x, y + label))
        draw.text((x + 8, y + 7), name, fill=(235, 235, 240))
    sheet.save(PREVIEWS / "contact_sheet.png", optimize=True)


def main() -> None:
    required = ["base_color.png", "height.png", "normal.png", "ao.png", "orm.png", "tiled_preview.png"]
    images = {name.removesuffix(".png"): Image.open(TEXTURES / name) for name in required}
    sizes = {name: list(image.size) for name, image in images.items() if name != "tiled_preview"}
    size_ok = len({tuple(size) for size in sizes.values()}) == 1
    edges = {name: edge_mae(image) for name, image in images.items() if name != "tiled_preview"}
    periodic_ok = all(max(metric.values()) <= 1.0 for metric in edges.values())

    w, h = images["orm"].size
    radius = max(4, round(w * 0.010))
    nail_x, nail_y = round(w * 0.052), round(h * 0.10)
    nail_box = (nail_x - radius, nail_y - radius, nail_x + radius, nail_y + radius)
    wood_box = (round(w * 0.35), round(h * 0.08), round(w * 0.65), round(h * 0.16))
    nail_metal = mean_crop(images["orm"], nail_box, 2)
    wood_metal = mean_crop(images["orm"], wood_box, 2)
    material_ids_ok = nail_metal >= 180 and wood_metal <= 8

    seam_height = mean_crop(images["height"], (round(w * 0.25), 0, round(w * 0.75), max(2, round(h * 0.004))))
    body_height = mean_crop(images["height"], wood_box)
    # Bevy depth maps use black for the raised surface and white for recesses.
    structure_ok = seam_height - body_height >= 70
    exact_grid = w == 1280 and h == 1280
    atlas_layout_ok = exact_grid and (HERE / "source" / "inputs" / "macro_master.png").is_file()
    seam_rows = [row_mean(images["height"], y) for y in (0, 256, 512, 768, 1024, 1279)]
    body_rows = [row_mean(images["height"], y) for y in (128, 384, 640, 896, 1152)]
    bottom_is_final_bevel = row_mean(images["base_color"], h - 1) < row_mean(images["base_color"], 1152) - 15
    framing_ok = exact_grid and min(seam_rows) > 170 and max(body_rows) < 125 and bottom_is_final_bevel
    length_error = normal_length_error(images["normal"])
    normals_ok = length_error <= 0.015

    checks = {
        "matching_dimensions": size_ok,
        "periodic_edges": periodic_ok,
        "packed_macro_atlas_layout": atlas_layout_ok,
        "wood_and_brass_material_ids": material_ids_ok,
        "recessed_seams": structure_ok,
        "pixel_perfect_five_plank_frame": framing_ok,
        "unit_length_normals": normals_ok,
    }
    report = {
        "passed": all(checks.values()),
        "checks": checks,
        "measurements": {
            "sizes": sizes,
            "edge_mae_8bit": edges,
            "nail_metallic_mean_8bit": nail_metal,
            "wood_metallic_mean_8bit": wood_metal,
            "seam_height_mean_8bit": seam_height,
            "board_body_height_mean_8bit": body_height,
            "seam_row_means_8bit": seam_rows,
            "board_center_row_means_8bit": body_rows,
            "bottom_is_final_bevel": bottom_is_final_bevel,
            "normal_mean_length_error": length_error,
        },
    }
    VALIDATION.mkdir(parents=True, exist_ok=True)
    (VALIDATION / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    summary = "PASS" if report["passed"] else "FAIL"
    lines = [f"# Wooden plank material validation: {summary}", ""]
    lines.extend(f"- [{'x' if passed else ' '}] {name.replace('_', ' ')}" for name, passed in checks.items())
    lines += ["", "See `report.json` for measurements.", ""]
    (VALIDATION / "report.md").write_text("\n".join(lines))

    roughness = images["orm"].convert("RGB").split()[1].convert("RGB")
    metallic = images["orm"].convert("RGB").split()[2].convert("RGB")
    make_contact_sheet({
        "Base color": images["base_color"],
        "Height": images["height"],
        "OpenGL normal": images["normal"],
        "Ambient occlusion": images["ao"],
        "Roughness": roughness,
        "Metallic (nails only)": metallic,
    })
    print(json.dumps(report, indent=2))
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
