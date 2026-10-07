"""GLB reimport review, fixed four CPU threads. No GPU API is selected."""
import bpy,math,json
from mathutils import Vector
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];HERE=ROOT/'tools/art/sail-source/evidence'
def vec(p):return Vector((p[0],-p[2],p[1]))
def clear():
    for o in list(bpy.data.objects):bpy.data.objects.remove(o,do_unlink=True)
def light(name,p,power,color,size):
    data=bpy.data.lights.new(name,'AREA');data.energy=power;data.color=color;data.shape='DISK';data.size=size
    o=bpy.data.objects.new(name,data);bpy.context.collection.objects.link(o);o.location=vec(p);o.rotation_euler=(vec((0,.3,0))-o.location).to_track_quat('-Z','Y').to_euler()
def camera(location,target,scale):
    data=bpy.data.cameras.new('Identical inspection camera');data.type='ORTHO';data.ortho_scale=scale
    o=bpy.data.objects.new(data.name,data);bpy.context.collection.objects.link(o);o.location=vec(location);o.rotation_euler=(vec(target)-o.location).to_track_quat('-Z','Y').to_euler();bpy.context.scene.camera=o
def render(file,asset,detail=False):
    clear();bpy.ops.import_scene.gltf(filepath=str(asset))
    for o in bpy.context.scene.objects:
        if o.type!='MESH':continue
        if o.data.shape_keys:
            for key,val in zip(o.data.shape_keys.key_blocks[1:],[.58,.27,.12]):key.value=val
        for mat in o.data.materials:
            if mat.name.startswith('Indigo canvas'):
                # Match the game's new dye factor. Diffuse transmission remains
                # engine-owned; this is geometry review, not Bevy lighting proof.
                bsdf=mat.node_tree.nodes.get('Principled BSDF')
                links=list(bsdf.inputs['Base Color'].links)
                if links:
                    source=links[0].from_socket;mat.node_tree.links.remove(links[0])
                    factor=(.33,.47,.75) if mat.name.startswith('Indigo canvas variation') else (.30,.44,.72)
                    mix=mat.node_tree.nodes.new('ShaderNodeMixRGB');mix.blend_type='MULTIPLY';mix.inputs[0].default_value=1;mix.inputs[2].default_value=(.8/factor[0],.86/factor[1],1/factor[2],1)
                    mat.node_tree.links.new(source,mix.inputs[1]);mat.node_tree.links.new(mix.outputs[0],bsdf.inputs['Base Color'])
    scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=12;scene.cycles.use_denoising=True
    scene.render.threads_mode='FIXED';scene.render.threads=4
    scene.world.use_nodes=True;bg=scene.world.node_tree.nodes['Background'];bg.inputs['Color'].default_value=(.19,.27,.38,1);bg.inputs['Strength'].default_value=.4
    light('Broad neutral key',(-3.2,4.5,5),450,(1,.89,.76),4)
    light('Cool fill',(3,1.5,4),220,(.65,.78,1),3)
    light('Rim',(-.4,3,-3),350,(1,.79,.5),2)
    if detail:camera((3,2.2,8),(0,1.31,.05),1.5)
    else:camera((4,2.4,10),(0,.12,0),4.0)
    scene.view_settings.view_transform='AgX';scene.render.resolution_x=800;scene.render.resolution_y=800;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(HERE/file)
    bpy.ops.render.render(write_still=True)
    if not detail and 'candidate' in file:bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'tools/art/sail-source/reimport-review.blend'),compress=True)
    print('CPU_REVIEW',file,flush=True)
render('canonical-cpu.png',ROOT/'.dream-loop/sail-export/base-sail.glb')
render('candidate-cpu.png',ROOT/'assets/art/sail.glb')
render('candidate-masthead-cpu.png',ROOT/'assets/art/sail.glb',True)
