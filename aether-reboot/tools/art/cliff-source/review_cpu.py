"""Original CPU4 diagnostic frames, shared cameras/lights, no postprocessing."""
import bpy,math,json,hashlib,time
from pathlib import Path
from mathutils import Vector
HERE=Path(__file__).resolve().parent;HERE.joinpath('review').mkdir(exist_ok=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
vec=lambda p:Vector((p[0],-p[2],p[1]))
report=[]
for kind in ['dawn','dawn-watch']:
    for variant in ['baseline','candidate']:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        file=(HERE/'baseline/assets/world' if variant=='baseline' else HERE.parents[2]/'assets/world')/(kind+'.glb');bpy.ops.import_scene.gltf(filepath=str(file))
        for image in bpy.data.images:
            if image.source=='FILE' and image.filepath:image.pack()
        scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=4;scene.cycles.use_denoising=False
        scene.render.threads_mode='FIXED';scene.render.threads=4
        scene.render.resolution_x=1672;scene.render.resolution_y=941;scene.render.resolution_percentage=100
        scene.view_settings.view_transform='AgX';scene.view_settings.look='AgX - Medium High Contrast';scene.view_settings.exposure=0
        world=bpy.data.worlds.new('Fixed cool diagnostic fill');world.use_nodes=True;scene.world=world;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.14,.23,.40,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
        light=bpy.data.lights.new('Fixed warm key','SUN');light.energy=3;light.color=(1,.74,.47);light.angle=math.radians(1.2)
        sun=bpy.data.objects.new('Fixed warm key',light);scene.collection.objects.link(sun);sun.rotation_euler=vec((-.79,-.342,-.51)).to_track_quat('-Z','Y').to_euler()
        data=bpy.data.cameras.new('Fixed camera');camera=bpy.data.objects.new('Fixed camera',data);scene.collection.objects.link(camera);scene.camera=camera;data.type='ORTHO'
        for view,location,target,scale in [('wide',(170,105,255),(0,-17,0),320),('close',(95,57,170),(5,-10,48),115)]:
            camera.location=vec(location);camera.rotation_euler=(vec(target)-camera.location).to_track_quat('-Z','Y').to_euler();data.ortho_scale=scale
            scene.render.image_settings.file_format='PNG';output=HERE/'review'/(kind+'-'+variant+'-'+view+'.png');scene.render.filepath=str(output)
            started=time.time();bpy.ops.render.render(write_still=True)
            report.append({'kind':kind,'variant':variant,'view':view,'geometry_sha256':sha(file),'camera_location_game_m':location,'camera_target_game_m':target,'orthographic_scale_m':scale,'sun_direction_game':[-.79,-.342,-.51],'sun_linear_color':[1,.74,.47],'cpu_threads':4,'cpu_samples':4,'gpu_used':False,'frame_original_resolution':[1672,941],'frame_sha256':sha(output),'seconds':time.time()-started})
        source=HERE/'source'/(kind+'-'+variant+'-review-full.blend');bpy.ops.wm.save_as_mainfile(filepath=str(source),compress=True)
(HERE/'review/cpu-render-evidence.json').write_text(json.dumps({'schema':1,'diagnostic_only_not_Bevy_quality_or_GPU_performance':True,'author_sha256':sha(Path(__file__)),'frames':report},indent=2)+'\n')
