"""Reproducible modular naval fittings. Blender 5.2; no downloaded assets.

Run from the repository: blender --background --python tools/art/build_ship.py
All authored points use metres, game Y-up. glTF output keeps these coordinates.
The only image inputs are the shared cedar colour/normal maps in assets/textures.
"""
import bpy
import bmesh
import hashlib
import json
import math
import struct
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'assets/ship'
SOURCE = ROOT / 'tools/art/ship-source'
WORK = ROOT / '.dream-loop/ship-export'
for directory in [OUT, SOURCE, WORK]:
    directory.mkdir(parents=True, exist_ok=True)
BATCH = {}
MATERIALS = {}


def material(name, color, metallic=0, roughness=.7, texture=False, emission=0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Metallic'].default_value = metallic
    bsdf.inputs['Roughness'].default_value = roughness
    if texture:
        tex = m.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image = bpy.data.images.load(str(ROOT / 'assets/textures/cedar.png'), check_existing=True)
        m.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
        normal_tex = m.node_tree.nodes.new('ShaderNodeTexImage')
        normal_tex.image = bpy.data.images.load(str(ROOT / 'assets/textures/cedar-normal.png'), check_existing=True)
        normal_tex.image.colorspace_settings.name = 'Non-Color'
        normal = m.node_tree.nodes.new('ShaderNodeNormalMap')
        normal.inputs['Strength'].default_value = .28
        m.node_tree.links.new(normal_tex.outputs['Color'], normal.inputs['Color'])
        m.node_tree.links.new(normal.outputs['Normal'], bsdf.inputs['Normal'])
    if emission:
        bsdf.inputs['Emission Color'].default_value = (*color, 1)
        bsdf.inputs['Emission Strength'].default_value = emission
    MATERIALS[name] = m
    return name


CEDAR = material('Ship | oiled cedar and hemp', (.35, .16, .055), roughness=.64, texture=True)
BRASS = material('Ship | aged brass', (.53, .29, .075), metallic=.72, roughness=.34)
IRON = material('Ship | blued wrought iron', (.045, .068, .105), metallic=.72, roughness=.4)
AMBER = material('Ship | warm lantern glass', (1, .47, .10), metallic=.05, roughness=.22, emission=2.0)


def vec(p):
    x, y, z = p
    return Vector((x, -z, y))


def geom(mat, points, faces, bevel=0, smooth=False):
    vertices, polygons, flags = BATCH.setdefault((mat, bevel), ([], [], []))
    offset = len(vertices)
    vertices.extend(vec(p) for p in points)
    polygons.extend(tuple(offset + i for i in face) for face in faces)
    flags.extend([smooth] * len(faces))


def box(p, size, mat, bevel=.002):
    x, y, z = p
    a, b, c = [v / 2 for v in size]
    pts = [(x + sx*a, y + sy*b, z + sz*c) for sx, sy, sz in
           [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
    geom(mat, pts, [(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)], bevel)


def tube(points, radius, mat, sides=6, closed=False):
    """Continuous swept rod, sharing rings rather than overlapping cylinders."""
    pts = [Vector(p) for p in points]
    vertices = []
    for i, p in enumerate(pts):
        if closed:
            tangent = (pts[(i+1)%len(pts)] - pts[(i-1)%len(pts)]).normalized()
        else:
            tangent = (pts[min(i+1,len(pts)-1)] - pts[max(i-1,0)]).normalized()
        axis = Vector((0,0,1))
        if abs(tangent.dot(axis)) > .95:
            axis = Vector((0,1,0))
        cross = tangent.cross(axis).normalized()
        binormal = tangent.cross(cross).normalized()
        for j in range(sides):
            a = j * math.tau / sides
            vertices.append(tuple(p + radius*(math.cos(a)*cross + math.sin(a)*binormal)))
    faces = []
    for i in range(len(pts) if closed else len(pts)-1):
        k = (i+1) % len(pts)
        faces.extend((i*sides+j, i*sides+(j+1)%sides, k*sides+(j+1)%sides, k*sides+j) for j in range(sides))
    if not closed:
        faces.extend([tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+j for j in range(sides))])
    geom(mat, vertices, faces, smooth=True)


def beam(a, b, radius, mat, sides=6):
    tube([a,b], radius, mat, sides)


def lathe(p, profile, mat, sides=8):
    """Hand-authored turned profile: (height, radius), around game Y."""
    x, y, z = p
    vertices = [(x+r*math.cos(i*math.tau/sides+math.pi/8), y+h, z+r*math.sin(i*math.tau/sides+math.pi/8))
                for h,r in profile for i in range(sides)]
    faces = [tuple(reversed(range(sides)))]
    faces += [(j*sides+i,j*sides+(i+1)%sides,(j+1)*sides+(i+1)%sides,(j+1)*sides+i)
              for j in range(len(profile)-1) for i in range(sides)]
    faces.append(tuple((len(profile)-1)*sides+i for i in range(sides)))
    geom(mat,vertices,faces)


def extrusion_x(profile, half_width, mat):
    """Moulding profile is a closed list of game (Y,Z) coordinates."""
    vertices = [(x,y,z) for x in [-half_width,half_width] for y,z in profile]
    n=len(profile)
    faces=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    faces += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    geom(mat,vertices,faces)


def rivet(p, radius=.006, depth=.004, mat=BRASS):
    x,y,z=p
    pts=[(x+radius*math.cos(i*math.tau/8),y+radius*math.sin(i*math.tau/8),z) for i in range(8)]
    pts.append((x,y,z+depth))
    geom(mat,pts,[(i,(i+1)%8,8) for i in range(8)])


def ring(p, radius, thickness, mat=BRASS, steps=14, axis='z'):
    x,y,z=p
    pts=[(x+radius*math.cos(i*math.tau/steps),y+radius*math.sin(i*math.tau/steps),z)
         if axis=='z' else (x+radius*math.cos(i*math.tau/steps),y,z+radius*math.sin(i*math.tau/steps))
         for i in range(steps)]
    tube(pts,thickness,mat,sides=5,closed=True)


def rail():
    # Two carved stanchions, each with a flared shoe and copper collar.
    for x in [-.222,.222]:
        lathe((x,0,0),[(0,.023),(.035,.023),(.052,.015),(.445,.014),(.472,.020),(.515,.020),(.530,.026),(.550,.022)],CEDAR,8)
        lathe((x,0,0),[(.012,.024),(.027,.024),(.036,.020)],BRASS,8)
        lathe((x,0,0),[(.440,.017),(.455,.019),(.469,.019)],BRASS,8)
        rivet((x,.022,.022),.006,.004)
        rivet((x,.450,.018),.006,.004)
    # Shaped cap and knee rail retain a precise half-metre repeat.
    extrusion_x([(.55,-.024),(.584,-.032),(.596,-.020),(.600,.018),(.586,.031),(.550,.024)],.25,CEDAR)
    extrusion_x([(.065,-.012),(.073,-.020),(.089,-.020),(.100,-.010),(.100,.010),(.089,.020),(.073,.020),(.065,.012)],.222,IRON)
    # Open wrought scrolls curve into the central diamond, leaving air between.
    for sign in [-1,1]:
        pts=[(sign*x,y,.001) for x,y in [(.205,.095),(.171,.110),(.120,.176),(.088,.245),(.111,.300),(.151,.318),(.170,.297),(.160,.273),(.140,.271)]]
        tube(pts,.009,IRON,6)
    tube([(-.065,.272,0),(0,.372,0),(.065,.272,0),(0,.165,0)],.008,BRASS,5,True)
    # A sagging hemp lifeline and helical seizing read clearly at grazing light.
    for strand in range(3):
        pts=[]
        for i in range(37):
            u=i/36
            a=u*math.tau*5+strand*math.tau/3
            pts.append((-.222+u*.444,.472-.050*math.sin(u*math.pi)+.005*math.cos(a),.016+.005*math.sin(a)))
        tube(pts,.0055,CEDAR,3)
    for x in [-.222,.222]:
        tube([(x+.020*math.cos(i*math.tau/8),.475+i*.002,.020*math.sin(i*math.tau/8)) for i in range(17)],.004,CEDAR,4)
    beam((0,.095,0),(0,.165,0),.007,BRASS,6)
    beam((0,.372,0),(0,.419,.016),.006,IRON,6)
    for sign in [-1,1]:beam((sign*.065,.272,0),(sign*.105,.286,0),.005,IRON,5)


def trim():
    # All relief stays within 25 mm of the attachment plane.
    extrusion_x([(-.125,0),(.125,0),(.125,.008),(.110,.014),(.096,.013),(.088,.007),(-.088,.007),(-.096,.013),(-.113,.015),(-.125,.008)],.25,CEDAR)
    for sign in [-1,1]:
        ys=[sign*v for v in [.093,.099,.113,.121]]
        profile=[(ys[0],.009),(ys[1],.022),(ys[2],.024),(ys[3],.014),(ys[3],.008),(ys[0],.005)]
        if sign<0: profile.reverse()
        extrusion_x(profile,.25,BRASS)
    extrusion_x([(-.047,.007),(-.037,.016),(.037,.016),(.047,.007)],.25,IRON)
    # The raised central strap tapers into two small cast arrow heads.
    for x in [-.212,-.106,0,.106,.212]:
        rivet((x,0,.017),.010,.008)
    for sign in [-1,1]:
        pts=[(sign*.026,.015,.017),(sign*.039,.032,.017),(sign*.075,.025,.017),(sign*.106,.017,.017)]
        tube(pts,.0035,BRASS,4)
        tube([(x,-y,z) for x,y,z in pts],.0035,BRASS,4)
    for x in [-.232,.232]:
        box((x,0,.012),(.018,.180,.010),BRASS,.001)
        for y in [-.071,.071]:rivet((x,y,.018),.005,.005,IRON)


def cabin():
    # A shallow carved window surround; the actual voxel glass remains visible.
    for x in [-.213,.213]:
        box((x,-.012,.021),(.061,.392,.042),CEDAR,.004)
        # Shutter louvres are real undercut geometry on the side stiles.
        for y in [-.134,-.067,0,.067,.134]:
            extrusion=[(y-.018,.041),(y+.012,.049),(y+.022,.039),(y-.018,.030)]
            pts=[(x+sx,y1,z1) for sx in [-.024,.024] for y1,z1 in extrusion]
            n=len(extrusion)
            geom(CEDAR,pts,[tuple(reversed(range(n))),tuple(n+i for i in range(n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)])
    extrusion_x([(-.25,0),(-.25,.049),(-.241,.065),(-.225,.065),(-.211,.047),(-.200,.039),(-.200,0)],.25,CEDAR)
    extrusion_x([(.190,0),(.190,.029),(.209,.043),(.217,.051),(.239,.051),(.25,.033),(.25,0)],.25,CEDAR)
    extrusion_x([(.223,.051),(.226,.056),(.233,.056),(.236,.051)],.25,BRASS)
    # Chamfered inner frame and mullions divide, without filling, the aperture.
    for x in [-.173,.173]:box((x,-.005,.033),(.014,.385,.017),BRASS,.002)
    for y in [-.193,.181]:box((0,y,.032),(.351,.014,.020),BRASS,.002)
    box((0,-.006,.026),(.015,.367,.020),IRON,.0015)
    box((0,-.005,.027),(.333,.013,.021),IRON,.0015)
    # Hinged corner tabs, visible pins, and a central cast diamond.
    for x in [-.212,.212]:
        for y in [-.171,.158]:
            box((x,y,.047),(.061,.023,.012),IRON,.001)
            rivet((x-.018,y,.054),.005,.004)
            rivet((x+.018,y,.054),.005,.004)
        beam((x,-.032,.054),(x,.032,.054),.008,BRASS,8)
    tube([(-.033,-.005,.045),(0,.030,.045),(.033,-.005,.045),(0,-.041,.045)],.0045,BRASS,5,True)


def lantern():
    # Hexagonal cage, flared roof, vent chimney, bail eye, and luminous panes.
    lathe((0,0,0),[(0,.060),(.010,.078),(.027,.078),(.045,.062),(.056,.066)],IRON,6)
    lathe((0,0,0),[(.016,.080),(.025,.080),(.031,.073)],BRASS,6)
    lathe((0,0,0),[(.063,.052),(.084,.062),(.242,.062),(.270,.049)],AMBER,6)
    lathe((0,0,0),[(.057,.068),(.068,.069),(.077,.064)],BRASS,6)
    lathe((0,0,0),[(.251,.068),(.267,.075),(.278,.077)],BRASS,6)
    for i in range(6):
        a=i*math.tau/6+math.pi/8
        x,z=math.cos(a),math.sin(a)
        tube([(x*.065,.059,z*.065),(x*.076,.092,z*.076),(x*.073,.230,z*.073),(x*.062,.274,z*.062)],.007,IRON,6)
    lathe((0,0,0),[(.276,.091),(.286,.094),(.302,.082),(.337,.037),(.342,.029)],IRON,6)
    lathe((0,0,0),[(.285,.095),(.292,.091),(.296,.085)],BRASS,6)
    for i in range(6):
        a=i*math.tau/6+math.pi/8
        beam((.027*math.cos(a),.337,.027*math.sin(a)),(.027*math.cos(a),.365,.027*math.sin(a)),.004,BRASS,5)
    lathe((0,0,0),[(.363,.033),(.371,.039),(.382,.022)],IRON,6)
    # Eye handle touches the exact requested height, 0.45 m.
    ring((0,.416,0),.029,.005,BRASS,16)
    rivet((0,.021,.077),.010,.006)
    # The front latch and its inset screw are distinct from the luminous glass.
    box((.055,.162,.045),(.021,.043,.012),BRASS,.002)
    rivet((.055,.162,.053),.004,.002,IRON)


def clear():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj,do_unlink=True)
    BATCH.clear()


def make_objects(name):
    objects=[]
    for (mat,bevel),(vertices,faces,flags) in BATCH.items():
        mesh=bpy.data.meshes.new(name+' | '+mat)
        mesh.from_pydata(vertices,[],faces)
        mesh.update()
        bm=bmesh.new();bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm,faces=bm.faces)
        bm.to_mesh(mesh);bm.free();mesh.update()
        uv=mesh.uv_layers.new(name='Cedar grain UV')
        for poly in mesh.polygons:
            poly.use_smooth=flags[poly.index]
            n=poly.normal
            dominant=max(range(3),key=lambda i:abs(n[i]))
            axes=[i for i in range(3) if i!=dominant]
            for index in poly.loop_indices:
                p=mesh.vertices[mesh.loops[index].vertex_index].co
                # A quarter tile avoids stretching large plank seams over small fittings.
                uv.data[index].uv=(p[axes[0]]*.8+.24,p[axes[1]]*.8+.30)
        obj=bpy.data.objects.new(name+' | '+mat,mesh)
        bpy.context.collection.objects.link(obj)
        mesh.materials.append(MATERIALS[mat])
        bpy.context.view_layer.objects.active=obj
        if bevel:
            mod=obj.modifiers.new('Machined edge relief','BEVEL')
            mod.width=bevel;mod.segments=1
            bpy.ops.object.modifier_apply(modifier=mod.name)
            mod=obj.modifiers.new('Stable broad face normals','WEIGHTED_NORMAL')
            mod.keep_sharp=True
            bpy.ops.object.modifier_apply(modifier=mod.name)
        objects.append(obj)
    merged=[]
    groups={mat:[obj for obj in objects if obj.data.materials[0]==MATERIALS[mat]] for mat in MATERIALS}
    for mat,group in groups.items():
        if not group:continue
        bpy.ops.object.select_all(action='DESELECT')
        for obj in group:obj.select_set(True)
        bpy.context.view_layer.objects.active=group[0]
        if len(group)>1:bpy.ops.object.join()
        group[0].name=name+' | '+mat
        mod=group[0].modifiers.new('Explicit triangles','TRIANGULATE')
        bpy.ops.object.modifier_apply(modifier=mod.name)
        group[0].data.validate(clean_customdata=False)
        merged.append(group[0])
    return merged


CONVENTIONS={
 'rail':{'origin':'foot centre on deck surface','dimensions_m':[.5,.6,.064],'attachment':'X horizontal; Y up; ±Z visible'},
 'trim':{'origin':'centre of rear attachment plane Z=0','dimensions_m':[.5,.25,.025],'attachment':'X horizontal; Y up; relief projects toward +Z'},
 'cabin-detail':{'origin':'centre of rear attachment plane Z=0','dimensions_m':[.5,.5,.065],'attachment':'X horizontal; Y up; exterior +Z; open window aperture'},
 'lantern':{'origin':'foot centre','dimensions_m':[.19,.45,.19],'attachment':'Y up; latch faces +Z'},
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def export_asset(name,objects):
    bpy.context.scene.unit_settings.system='METRIC'
    bpy.context.scene.unit_settings.scale_length=1
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:obj.select_set(True)
    bpy.context.view_layer.objects.active=objects[0]
    source=SOURCE/(name+'.blend')
    for image in bpy.data.images:
        if image.source=='FILE' and image.filepath:
            image.filepath=bpy.path.relpath(bpy.path.abspath(image.filepath),start=str(SOURCE))
    bpy.ops.wm.save_as_mainfile(filepath=str(source),compress=True)
    temp=WORK/name
    temp.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(temp/(name+'.gltf')),export_format='GLTF_SEPARATE',use_selection=True,
        export_yup=True,export_apply=False,export_animations=False,export_cameras=False,export_lights=False,
        export_tangents=True,export_image_format='AUTO',export_extras=True)
    doc=json.loads((temp/(name+'.gltf')).read_text())
    assert len(doc.get('buffers',[]))==1
    binary=(temp/doc['buffers'][0].pop('uri')).read_bytes()
    for image in doc.get('images',[]):
        image['uri']='../textures/'+Path(image['uri']).name
        assert (OUT/image['uri']).is_file(),image
    doc['asset']['extras']={'generator_sha256':sha(Path(__file__)),**CONVENTIONS[name]}
    encoded=json.dumps(doc,separators=(',',':')).encode()
    encoded+=b' '*((-len(encoded))%4)
    binary+=b'\0'*((-len(binary))%4)
    body=struct.pack('<II',len(encoded),0x4e4f534a)+encoded+struct.pack('<II',len(binary),0x004e4942)+binary
    glb=OUT/(name+'.glb')
    glb.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
    triangles=sum(len(obj.data.polygons) for obj in objects)
    points=[(v.co.x,v.co.z,-v.co.y) for obj in objects for v in obj.data.vertices]
    bounds=[[round(min(p[i] for p in points),6),round(max(p[i] for p in points),6)] for i in range(3)]
    assert triangles<2000,(name,triangles)
    assert len(objects)<=4,(name,len(objects))
    return {'file':glb.relative_to(ROOT).as_posix(),'sha256':sha(glb),'bytes':glb.stat().st_size,
            'source':source.relative_to(ROOT).as_posix(),'source_sha256':sha(source),'triangles':triangles,
            'meshes':len(objects),'materials':len(doc.get('materials',[])),'bounds_xyz_m':bounds,**CONVENTIONS[name]}


def light(name,power,color,position,size):
    data=bpy.data.lights.new(name,'AREA')
    data.energy=power;data.color=color;data.shape='DISK';data.size=size
    obj=bpy.data.objects.new(name,data);bpy.context.collection.objects.link(obj)
    obj.location=vec(position)
    obj.rotation_euler=(vec((0,.25,0))-obj.location).to_track_quat('-Z','Y').to_euler()


def presentation(assets):
    clear()
    placements={'rail':(-.93,0,0),'trim':(-.28,.28,0),'cabin-detail':(.39,.28,0),'lantern':(.96,.03,0)}
    for name,meshes in assets.items():
        for mesh in meshes:
            obj=bpy.data.objects.new(name,mesh)
            bpy.context.collection.objects.link(obj)
            obj.location=vec(placements[name])
    scene=bpy.context.scene
    scene.render.engine='CYCLES';scene.cycles.samples=48;scene.cycles.use_denoising=True
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences
        prefs.compute_device_type='CUDA';prefs.get_devices()
        has_gpu=False
        for device in prefs.devices:
            device.use=device.type=='CUDA';has_gpu=has_gpu or device.use
        if has_gpu:scene.cycles.device='GPU'
    except Exception:pass
    floor=material('Presentation | midnight blue',(.011,.023,.036),roughness=.77)
    BATCH.clear();box((0,-.037,0),(12,.06,10),floor,.01)
    make_objects('Presentation floor')
    # Brass accent lines beneath the four separate modules provide scale without hiding silhouettes.
    light('Warm large key',150,(1,.79,.57),(-1.6,2.5,2.1),2.4)
    light('Cool broad fill',85,(.46,.66,1),(1.5,1.6,1.5),1.9)
    light('Brass rim',190,(1,.71,.37),(.4,1.8,-1.6),1.7)
    scene.world.use_nodes=True
    scene.world.node_tree.nodes['Background'].inputs['Color'].default_value=(.10,.17,.28,1)
    scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value=.35
    camera_data=bpy.data.cameras.new('Kit review')
    camera=bpy.data.objects.new('Kit review',camera_data);bpy.context.collection.objects.link(camera)
    camera.location=vec((1.08,1.11,4.7))
    target=vec((0,.265,0))
    camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler()
    camera_data.type='ORTHO';camera_data.ortho_scale=2.75;scene.camera=camera
    scene.view_settings.view_transform='AgX'
    scene.render.resolution_x=1800;scene.render.resolution_y=860;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG'
    scene.render.filepath=str(ROOT/'.dream-loop/ship-kit.png')
    bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/'presentation.blend'),compress=True)
    bpy.ops.render.render(write_still=True)


def main():
    manifest={'generator':'tools/art/build_ship.py','generator_sha256':sha(Path(__file__)),
              'units':'metres','axes':'right-handed glTF, Y up, +Z exterior',
              'shared_images':[{ 'file':p,'sha256':sha(ROOT/p)} for p in ['assets/textures/cedar.png','assets/textures/cedar-normal.png']],
              'modules':{}}
    for name,build in [('rail',rail),('trim',trim),('cabin-detail',cabin),('lantern',lantern)]:
        clear();build();objects=make_objects(name)
        report=export_asset(name,objects)
        manifest['modules'][name]=report
        print('SHIP_MODULE',json.dumps(report),flush=True)
    (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    # Render the delivered GLBs after re-import, including the external maps.
    saved={}
    for name in manifest['modules']:
        clear()
        bpy.ops.import_scene.gltf(filepath=str(OUT/(name+'.glb')))
        saved[name]=[obj.data for obj in bpy.context.scene.objects if obj.type=='MESH']
        for mesh in saved[name]:mesh.use_fake_user=True
    presentation(saved)
    print('SHIP_KIT_COMPLETE',flush=True)


if __name__=='__main__':main()
