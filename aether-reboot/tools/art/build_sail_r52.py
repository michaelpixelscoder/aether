"""R52 sail-only rest cut from the archived R44 Blender canvas source.

No voxel, polygon or UV is regenerated. Unchanged shared build_art.py remains
the source of the other art models. Run this, build_sail_rig.py, then assembly.
"""
import bpy,math,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'assets/art';OUT.mkdir(parents=True,exist_ok=True)
ARCHIVE=ROOT/'tools/art/sail-source/archive/r44-canvas.blend'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(ARCHIVE)=='467bb211fc896e20ecafa01ef206d321cbfb1eb3ad21c7291039c801ae518487'
bpy.ops.wm.open_mainfile(filepath=str(ARCHIVE))
for image in bpy.data.images:
    if image.source=='FILE' and image.filepath:
        image.filepath=str(ROOT/'assets/textures'/Path(image.filepath).name)
        assert Path(image.filepath).is_file(),image.filepath
        image.reload();image.pack()

def cut(x,y,z):
    if y>1.26:return x,y,z
    u=max(0,min(1,(x+1.25)/2.5));v=max(0,min(1,(y+1.15)/2.30))
    e=math.sin(math.pi*u)*math.sin(math.pi*v)
    return (x*(1-.13*math.sin(math.pi*v)),
            y+math.sin(math.pi*u)*(.13*(1-v)**4-.045*v**6),
            z-.21*e)

animated=0
for obj in list(bpy.context.scene.objects):
    if obj.type!='MESH' or not obj.data.shape_keys:continue
    original=[v.co.copy() for v in obj.data.shape_keys.key_blocks[0].data]
    obj.shape_key_clear()
    for vertex,p in zip(obj.data.vertices,original):
        x,y,z=cut(p.x,p.z,-p.y);vertex.co=(x,-z,y)
    obj.data.update();bpy.context.view_layer.objects.active=obj
    mod=obj.modifiers.new('R52 rest cut weighted normals','WEIGHTED_NORMAL');mod.keep_sharp=True
    bpy.ops.object.modifier_apply(modifier=mod.name)
    basis=[v.co.copy() for v in obj.data.vertices]
    obj.shape_key_add(name='Basis',from_mix=False)
    for key_name in ['WindPressure','RippleSin','RippleCos']:
        key=obj.shape_key_add(name=key_name,from_mix=False);key.slider_min=-1;key.slider_max=1
        for index,vertex in enumerate(key.data):
            vertex.co=basis[index];x,y=original[index].x,original[index].z
            u=max(0,min(1,(x+1.25)/2.5));v=max(0,min(1,(y+1.15)/2.30))
            e=math.sin(math.pi*u)*math.sin(math.pi*v);wave=3.7*x+1.9*y
            delta=(.18 if key_name=='WindPressure' else .065*(math.sin(wave) if key_name=='RippleSin' else math.cos(wave)))*e
            vertex.co.y-=delta # Real +Z response for positive apparent wind.
    animated+=1
assert animated==5
sources=ROOT/'tools/art/source';sources.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=str(sources/'sail.blend'),compress=True)
file=OUT/'sail.glb'
bpy.ops.export_scene.gltf(filepath=str(file),export_format='GLB',export_yup=True,export_apply=False,export_morph=True,export_morph_normal=True,export_image_format='AUTO',export_animations=False,export_cameras=False,export_lights=False)
data=file.read_bytes();length=struct.unpack_from('<I',data,12)[0];doc=json.loads(data[20:20+length])
doc['asset']['extras']={'aether_geometry_sha256':sha(ROOT/'tools/art/scene.json'),'aether_generator_sha256':sha(Path(__file__)),'r52_original_canvas_sha256':sha(ARCHIVE)}
factors={'Cedar | grain':(.70,.62,.52,1),'Indigo canvas':(.30,.44,.72,1),'Indigo canvas variation':(.33,.47,.75,1)}
for material in doc.get('materials',[]):
    if material['name'] in factors:material['pbrMetallicRoughness']['baseColorFactor']=factors[material['name']]
encoded=json.dumps(doc,separators=(',',':')).encode();encoded+=b' '*((-len(encoded))%4)
body=struct.pack('<II',len(encoded),0x4e4f534a)+encoded+data[20+length:]
file.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
print('R52_ARCHIVED_CANVAS',json.dumps({'glb_sha256':sha(file),'source_sha256':sha(sources/'sail.blend'),'animated_surfaces':animated}),flush=True)
