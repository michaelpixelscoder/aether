"""Read-only source topology/UV comparison, independent of glTF reindexing."""
import bpy,json,hashlib,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def inspect(path):
    bpy.ops.wm.open_mainfile(filepath=str(path))
    surfaces=[]
    for o in bpy.context.scene.objects:
        if o.type!='MESH' or not o.data.shape_keys:continue
        m=o.data
        topology=repr([(tuple(p.vertices),p.material_index) for p in m.polygons]).encode()
        uv=b''.join(struct.pack('<ff',*v.uv) for v in m.uv_layers[0].data)
        surfaces.append({'vertices':len(m.vertices),'polygons':len(m.polygons),'material':m.materials[0].name,'topology_sha256':hashlib.sha256(topology).hexdigest(),'loop_uv_sha256':hashlib.sha256(uv).hexdigest(),'keys':[k.name for k in m.shape_keys.key_blocks]})
    return sorted(surfaces,key=lambda s:(s['material'],s['vertices']))
a=inspect(ROOT/'tools/art/sail-source/archive/r44-canvas.blend');b=inspect(ROOT/'tools/art/source/sail.blend')
assert a==b,(a,b)
report={'source_topology_and_loop_uv_exact':True,'surfaces':b,'explanation':'glTF exporter may reindex vertices or choose another quad diagonal after deformation; Blender polygon topology and every original loop UV are byte-exact.'}
(ROOT/'tools/art/sail-source/evidence/r52/current-source-uv-topology.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report),flush=True)
