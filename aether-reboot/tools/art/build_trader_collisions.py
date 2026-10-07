"""Export conservative merchant colliders from the authored source primitives.

Run: blender --background --python-exit-code 1 --python tools/art/build_trader_collisions.py
Does not rebuild or modify trader.glb, its source blends, or its manifest.
All output centres/sizes are game-local metres: Y up, forward -Z.
"""
import hashlib
import importlib.util
import itertools
import json
from pathlib import Path

from mathutils import Vector
from mathutils.geometry import tessellate_polygon

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'tools/art/build_trader.py'
OUTPUT = ROOT / 'assets/fauna/trader-collisions.json'
REPORT = ROOT / '.dream-loop/trader-collisions-validation.json'
EPSILON = 2e-6


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def authored_primitives(lod):
    """Execute the actual model functions, intercepting their geometric output."""
    spec = importlib.util.spec_from_file_location('merchant_collision_source', SOURCE)
    model = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(model)
    model.LOD = lod
    result = {}
    for group in ['envelope', 'hull', 'cabin', 'cargo', 'engines']:
        primitives = []
        declarations = []
        original_box = model.box

        def collect(mat, points, faces, bevel=0, smooth=False):
            primitives.append({'points': [tuple(p) for p in points],
                               'faces': faces, 'bevel': bevel, 'material': mat})

        def collect_box(position, size, mat, bevel=.018):
            declarations.append({'center': list(position), 'size': list(size),
                                 'bevel': 0 if lod else min(bevel, min(size) * .15),
                                 'primitive': len(primitives), 'material': mat})
            original_box(position, size, mat, bevel)

        model.geom = collect
        model.box = collect_box
        getattr(model, group)()
        result[group] = {'primitives': primitives, 'boxes': declarations}
        model.box = original_box
    return model, result


def planes(primitives):
    """Conservative inner half-spaces of source faces, including twisted quads.

    Every quad is triangulated by Blender's tessellator, independently from the
    box construction. Its inward core is the intersection of those half-spaces.
    """
    points = [Vector(p) for primitive in primitives for p in primitive['points']]
    interior = sum(points, Vector()) / len(points)
    result = []
    for primitive in primitives:
        vertices = [Vector(p) for p in primitive['points']]
        for face in primitive['faces']:
            polygon = [vertices[i] for i in face]
            triangles = tessellate_polygon([polygon])
            for triangle in triangles:
                a, b, c = [polygon[v] if isinstance(v, int) else v for v in triangle]
                normal = (b - a).cross(c - a).normalized()
                assert normal.length > .99
                if normal.dot(interior - a) > 0:
                    normal = -normal
                result.append((normal, normal.dot(a)))
    return result


def corners(box):
    return [Vector(tuple(box['center'][i] + sign[i] * box['size'][i] / 2
                         for i in range(3)))
            for sign in itertools.product([-1, 1], repeat=3)]


def clearance(box, boundaries):
    return min(offset - normal.dot(point)
               for point in corners(box) for normal, offset in boundaries)


def interpolate(profile, z):
    for (z0, value0), (z1, value1) in zip(profile, profile[1:]):
        if z0 <= z <= z1:
            t = (z - z0) / (z1 - z0)
            return value0 * (1 - t) + value1 * t
    raise AssertionError(z)


def main():
    model, hd = authored_primitives(False)
    _, lod = authored_primitives(True)
    boxes = []
    witnesses = []

    def append(group, minimum, maximum, checks):
        minimum = [float(v) for v in minimum]
        maximum = [float(v) for v in maximum]
        box = {'center': [round((a + b) / 2, 7) for a, b in zip(minimum, maximum)],
               'size': [round(b - a, 7) for a, b in zip(minimum, maximum)]}
        assert all(v > 0 for v in box['size'])
        clearances = [clearance(box, boundaries) for boundaries in checks]
        assert min(clearances) >= -EPSILON, (group, box, clearances)
        witnesses.append({'box': len(boxes), 'group': group,
                          'source_face_clearance_hd_lod_m': [round(c, 7) for c in clearances]})
        boxes.append(box)

    # The first N source polygons form the closed envelope, before any stitches,
    # ropes or ornaments. Requiring both 24- and 12-sided hulls fits both LODs.
    envelope_planes = [planes(data['envelope']['primitives'][:(len(model.PROFILE) - 1) * sides])
                       for data, sides in [(hd, 24), (lod, 12)]]
    stations = [-6.85, -6.4, -5.7, -4.7, -3.2, -1.5,
                1.5, 3.2, 4.7, 5.7, 6.4, 6.85]
    for z0, z1 in zip(stations, stations[1:]):
        radius = min(interpolate(model.PROFILE, z0), interpolate(model.PROFILE, z1))
        # A broad middle bar plus narrower upper/lower bars: no circumscribed
        # ellipse bounds, and the narrowed tips never block the surrounding air.
        for x0, x1, y0, y1 in [(-.86, .86, -.47, .47),
                                (-.47, .47, .47, .85),
                                (-.47, .47, -.85, -.47)]:
            append('envelope',
                   [x0 * model.RX * radius, model.CY + y0 * model.RY * radius, z0],
                   [x1 * model.RX * radius, model.CY + y1 * model.RY * radius, z1],
                   envelope_planes)

    # Each plank course is already a solid source wedge. Keep every vertical
    # seam open; never fill the open deck or the suspension gap above it.
    intervals = len(model.HULL) - 1
    hull_lod_planes = [planes([p]) for p in lod['hull']['primitives'][:3 * intervals]]
    for index, primitive in enumerate(hd['hull']['primitives'][:6 * intervals]):
        course, section = divmod(index, intervals)
        x = min(abs(p[0]) for p in primitive['points']) - .004
        if x < .025:
            continue  # Extremely thin front-tip strips need no collision.
        y0 = min(p[1] for p in primitive['points']) + .002
        y1 = max(p[1] for p in primitive['points']) - .002
        z0 = min(p[2] for p in primitive['points']) + .002
        z1 = max(p[2] for p in primitive['points']) - .002
        checks = [planes([primitive]), hull_lod_planes[(course // 2) * intervals + section]]
        # Lofted side quads may be slightly twisted. Fit the actual triangular
        # planes, not just their projected width at the four source corners.
        original_x = x
        for attempt in range(12):
            x = original_x
            for boundaries in checks:
                for normal, offset in boundaries:
                    if abs(normal.x) > .00001:
                        for y, z in itertools.product([y0, y1], [z0, z1]):
                            x = min(x, (offset - normal.y * y - normal.z * z - .001) / abs(normal.x))
            if x > .025:
                break
            trim = (z1 - z0) * .08
            z0 += trim
            z1 -= trim
        assert x > 0, (index, course, section, x, y0, y1, z0, z1)
        append('gondola_planks', [-x, y0, z0], [x, y1, z1],
               checks)

    def solid_box(group, declaration):
        # Shrink every axis beyond its bevel width. The complete box is then
        # inside the beveled source, including its clipped edges and corners.
        inset = declaration['bevel'] + .002
        minimum = [c - s / 2 + inset for c, s in zip(declaration['center'], declaration['size'])]
        maximum = [c + s / 2 - inset for c, s in zip(declaration['center'], declaration['size'])]
        assert all(b > a for a, b in zip(minimum, maximum))
        counterpart = next(item for item in lod[group]['boxes']
                           if item['center'] == declaration['center'] and item['size'] == declaration['size'])
        checks = [planes([hd[group]['primitives'][declaration['primitive']]]),
                  planes([lod[group]['primitives'][counterpart['primitive']]])]
        append(group, minimum, maximum, checks)
        witnesses[-1]['bevel_inset_m'] = inset

    for declaration in hd['hull']['boxes']:
        if declaration['center'] in [[0, -1.85, -.05], [0, .33, -3.60]]:
            solid_box('hull', declaration)
    for declaration in hd['cabin']['boxes']:
        if declaration['material'] == model.WOOD:
            # Window muntins are small decoration; retain walls, cornice/roof.
            if declaration['size'][2] > 1.5:
                solid_box('cabin', declaration)
    for declaration in hd['cargo']['boxes']:
        if min(declaration['size']) > .5:
            solid_box('cargo', declaration)

    # Closed barrel/pod source rods receive small central inscribed boxes.
    # Thin ropes, railings, lanterns, fins, propeller blades and their empty
    # circular guards deliberately do not create any broad collision field.
    for group, selections in [
        ('cargo', [((.46, .54, .30), (.34, .42, .34)),
                   ((.35, .54, .92), (.34, .42, .34))]),
        ('engines', [((-2.01, .4, 2.15), (.30, .30, .58)),
                     ((2.01, .4, 2.15), (.30, .30, .58))]),
    ]:
        for center, size in selections:
            probe = {'center': center, 'size': size}
            checks = []
            for data in [hd, lod]:
                found = None
                for primitive in data[group]['primitives']:
                    expected_material = model.NAVY if group == 'cargo' else model.WOOD
                    if primitive['material'] != expected_material or len(primitive['faces'][-1]) < 6:
                        continue
                    bounds = planes([primitive])
                    if clearance(probe, bounds) > .0001:
                        found = bounds
                        break
                assert found is not None, (group, probe)
                checks.append(found)
            append(group, [c - s / 2 for c, s in zip(center, size)],
                   [c + s / 2 for c, s in zip(center, size)], checks)

    def blocked(point):
        return any(all(abs(point[i] - box['center'][i]) <= box['size'][i] / 2
                       for i in range(3)) for box in boxes)

    # Player-relevant negative and positive examples: prevent the original
    # large bounding boxes from silently returning in a future rebuild.
    free_probes = [(0, 2.8, 0), (1.5, 1.6, 0), (2.5, 5.64, -6),
                   (0, 7.6, -5.8), (2.75, .4, 2.9), (0, .8, -2)]
    solid_probes = [(0, 5.64, 0), (2, 5.64, 0), (0, 6.9, 0),
                    (0, -.8, 0), (0, .65, 2.6), (-.5, .5, -.75)]
    assert all(not blocked(p) for p in free_probes), 'Collision fills visible empty space'
    assert all(blocked(p) for p in solid_probes), 'Missing principal solid'
    assert all(p.length < 12 for box in boxes for p in corners(box))

    counts = {group: sum(w['group'] == group for w in witnesses)
              for group in sorted({w['group'] for w in witnesses})}
    metadata = {
        'axes': 'Y up; forward -Z; hull-centre origin; local metres',
        'size_convention': 'Full XYZ dimensions, not half extents; identity local rotation',
        'generator': 'tools/art/build_trader_collisions.py',
        'generator_sha256': digest(Path(__file__)),
        'model_generator': 'tools/art/build_trader.py',
        'model_generator_sha256': digest(SOURCE),
        'asset_sha256': {name: digest(ROOT / 'assets/fauna' / name)
                         for name in ['trader.glb', 'trader-lod.glb']},
        'box_count': len(boxes), 'group_counts': counts,
        'rationale': [
            'Three inscribed rectangular bars per longitudinal envelope section; all corners satisfy both HD and LOD source-face half-spaces.',
            'Gondola boxes fit individual solid plank wedges; six authored vertical courses and their 12 mm seams remain distinct.',
            'Cabin, stepped roof, bow step, keel and cargo boxes stay inside their source bevels; small central boxes fit closed barrels and engine pods.',
            'Empty suspension space and open deck remain empty. Thin rigging, railings, fins, chimney, lanterns and rotating blades/guards are excluded.',
            'Deliberately conservative approximation: some visible curved surface has no collision; no principal box extends into surrounding air.',
        ],
        'verification': '.dream-loop/trader-collisions-validation.json',
    }
    OUTPUT.write_text(json.dumps({'trader': boxes, 'metadata': metadata}, indent=2) + '\n', encoding='utf-8')
    report = {'passed': True, 'box_count': len(boxes), 'group_counts': counts,
              'source_sha256': digest(SOURCE), 'collision_sha256': digest(OUTPUT),
              'corner_count': len(boxes) * 8, 'hd_and_lod_source_halfspace_checks': True,
              'beveled_box_insets_checked': True, 'within_physics_radius_12m': True,
              'free_space_probes': free_probes, 'solid_probes': solid_probes,
              'witnesses': witnesses}
    REPORT.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('TRADER_COLLISIONS', json.dumps({k: report[k] for k in
          ['passed', 'box_count', 'group_counts', 'corner_count', 'collision_sha256']}), flush=True)


if __name__ == '__main__':
    main()
