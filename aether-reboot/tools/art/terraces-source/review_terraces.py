"""Fixed comparison cameras; final GLBs reimported; Cycles CPU4 only."""
import bpy,json,sys
from pathlib import Path
from mathutils import Vector
HERE=Path(__file__).resolve().parent
ASSETS=HERE/'assets' if (HERE/'assets/world').exists() else HERE.parents[2]/'assets'
WIDE='--wide' in sys.argv
def vec(p):return Vector((p[0],-p[2],p[1]))
def render(kind,label):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    assets=HERE/'baseline/assets' if label=='before' else ASSETS
    bpy.ops.import_scene.gltf(filepath=str(assets/'world'/(kind+'.glb')))
    scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=20;scene.cycles.use_denoising=True
    scene.render.threads_mode='FIXED';scene.render.threads=4
    scene.world=bpy.data.worlds.new('Same review environment');scene.world.use_nodes=True
    bg=scene.world.node_tree.nodes['Background'];bg.inputs['Color'].default_value=(.19,.27,.40,1);bg.inputs['Strength'].default_value=.65
    light=bpy.data.lights.new('Fixed warm sun','SUN');light.energy=3;light.color=(1,.79,.56);light.angle=.04
    o=bpy.data.objects.new(light.name,light);scene.collection.objects.link(o);o.location=vec((-100,130,100));o.rotation_euler=(vec((0,0,0))-o.location).to_track_quat('-Z','Y').to_euler()
    data=bpy.data.cameras.new('Fixed terrain comparison');data.type='ORTHO';data.ortho_scale=(195 if kind=='dawn' else 178) if WIDE else (123 if kind=='dawn' else 113)
    camera=bpy.data.objects.new(data.name,data);scene.collection.objects.link(camera)
    target=(0,-5,0) if WIDE else ((0,4,16) if kind=='dawn' else (-9,10,18))
    position=(125,125,173) if kind=='dawn' else (100,140,184)
    camera.location=vec(position);camera.rotation_euler=(vec(target)-camera.location).to_track_quat('-Z','Y').to_euler();scene.camera=camera
    scene.view_settings.view_transform='AgX';scene.render.resolution_x=864 if WIDE else 1080;scene.render.resolution_y=720 if WIDE else 900;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(HERE/'evidence'/(kind+('-hero-' if WIDE else '-')+label+'.png'))
    bpy.ops.render.render(write_still=True);print('TERRACES_REVIEW',kind,label,flush=True)
args=[a for a in sys.argv[sys.argv.index('--')+1:] if not a.startswith('--')] if '--' in sys.argv else ['dawn','dawn-watch']
if not args:args=['dawn','dawn-watch']
for kind in args:
    for label in ['before','after']:render(kind,label)
