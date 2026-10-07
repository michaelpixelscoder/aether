import bpy,json,hashlib
from pathlib import Path
HERE=Path(__file__).resolve().parent
report=[]
for kind in ['dawn','dawn-watch']:
    file=HERE/'source'/(kind+'-candidate-review-full.blend')
    bpy.ops.wm.open_mainfile(filepath=str(file))
    images=[i for i in bpy.data.images if i.source=='FILE']
    assert images and all(i.packed_file for i in images)
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    assert len(objects)==12
    assert bpy.context.scene.cycles.device=='CPU'
    assert bpy.context.scene.render.threads==4
    report.append({'kind':kind,'full_source_sha256':hashlib.sha256(file.read_bytes()).hexdigest(),'mesh_objects':len(objects),'file_images':len(images),'all_images_packed':True,'cpu_device':True,'threads':4})
    file=HERE/'source'/(kind+'-candidate.blend')
    bpy.ops.wm.open_mainfile(filepath=str(file))
    images=[i for i in bpy.data.images if i.source=='FILE'];objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    assert len(objects)==1 and len(images)==3 and all(i.packed_file for i in images)
    report.append({'kind':kind,'rock_source_sha256':hashlib.sha256(file.read_bytes()).hexdigest(),'mesh_objects':len(objects),'file_images':len(images),'all_images_packed':True,'rock_triangles':sum(len(o.data.polygons) for o in objects)})
(HERE/'source/reopen-evidence.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
