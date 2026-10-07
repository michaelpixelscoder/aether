"""CPU-only source reopening and packed image checks, no rendering."""
import bpy,json,hashlib
from pathlib import Path
HERE=Path(__file__).resolve().parent
records=[]
for kind in ['dawn','dawn-watch']:
    file=HERE/'source'/(kind+'-terraces.blend')
    bpy.ops.wm.open_mainfile(filepath=str(file))
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    original=[o for o in objects if o.get('r56_role')=='immutable reference']
    added=[o for o in objects if o.get('r56_role')=='additive landscape geometry']
    assert len(original)==12 and len(added)==3 and len(objects)==15
    images=[i for i in bpy.data.images if i.source=='FILE']
    assert images and all(i.packed_file is not None for i in images)
    records.append({'kind':kind,'sha256':hashlib.sha256(file.read_bytes()).hexdigest(),'original_objects':len(original),'addition_objects':len(added),'all_file_images_packed':True,'packed_images':len(images)})
(HERE/'evidence/source-reopen.json').write_text(json.dumps({'schema':1,'sources':records},indent=2)+'\n')
print('TERRACES_SOURCE_REOPEN',json.dumps(records),flush=True)
