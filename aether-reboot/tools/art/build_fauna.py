"""Authored cave fauna, metres, Y-up, forward -Z. Blender 5.2.

Run: blender --background --python tools/art/build_fauna.py
Only shared local textures are used. Limbs are rigid articulated hierarchies;
their unrotated local axes match the game axes. L denotes X-negative.
"""
import bpy, bmesh, math, random, json, struct, hashlib
from pathlib import Path
from mathutils import Vector, Matrix

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'assets/fauna'
SOURCE=ROOT/'tools/art/fauna-source'
WORK=SOURCE/'.export'
for p in [OUT,SOURCE,WORK]:p.mkdir(parents=True,exist_ok=True)
bpy.context.preferences.filepaths.save_version=0
MATERIALS={};FACTORS={};BATCH={};PIVOTS={};PART='Body'
RNG=random.Random(40720)


def vec(p):return Vector((p[0],-p[2],p[1]))


def material(name,color,rough=.8,metal=0,texture=None,emission=0):
    m=bpy.data.materials.new(name);m.use_nodes=True
    bsdf=m.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value=(*color,1)
    bsdf.inputs['Roughness'].default_value=rough
    bsdf.inputs['Metallic'].default_value=metal
    if texture:
        t=m.node_tree.nodes.new('ShaderNodeTexImage');t.name='Shared texture'
        t.image=bpy.data.images.load(str(ROOT/'assets/textures'/texture),check_existing=True)
        mix=m.node_tree.nodes.new('ShaderNodeMixRGB');mix.name='Authored mineral tint'
        mix.blend_type='MULTIPLY';mix.inputs[0].default_value=1;mix.inputs[2].default_value=(*color,1)
        m.node_tree.links.new(t.outputs['Color'],mix.inputs[1]);m.node_tree.links.new(mix.outputs[0],bsdf.inputs['Base Color'])
        FACTORS[name]=color
    if emission:
        bsdf.inputs['Emission Color'].default_value=(*color,1)
        bsdf.inputs['Emission Strength'].default_value=emission
    MATERIALS[name]=m
    return name


LIMESTONE=material('Fauna | cave limestone',(.30,.35,.29),.92,texture='limestone-world.png')
OBSIDIAN=material('Fauna | fractured obsidian',(.021,.032,.037),.30,.23)
MOSS=material('Fauna | mineral olive patina',(.10,.16,.055),.97,texture='limestone-world.png')
AMBER=material('Fauna | amber eyes',(1,.29,.035),.27,emission=3.4)
SCALE=material('Fauna | burgundy scales',(.105,.008,.015),.66,.08,texture='stone.png')
WINE=material('Fauna | wine wing membrane',(.145,.012,.022),.81,texture='stone.png')
RUST=material('Fauna | warm membrane facets',(.235,.027,.025),.77,texture='stone.png')


def set_part(name,pivot=(0,0,0)):
    global PART
    PART=name;PIVOTS[name]=pivot


def geom(mat,points,faces,smooth=False):
    vertices,polygons,flags=BATCH.setdefault((PART,mat),([],[],[]))
    n=len(vertices);vertices.extend(tuple(p) for p in points)
    polygons.extend(tuple(n+i for i in f) for f in faces)
    flags.extend([smooth]*len(faces))


def stone(p,size,mat=LIMESTONE,angle=0,irregular=.10,seed=None):
    """Individually chipped rectangular strata, three unequal octagonal rings."""
    rng=RNG if seed is None else random.Random(seed)
    sx,sy,sz=size
    outline=[(-.34,-.5),(.33,-.5),(.5,-.30),(.5,.29),(.30,.5),(-.30,.5),(-.5,.32),(-.5,-.27)]
    points=[]
    for level,scale in [(-.5,.83),(-.5+.18,1),(.5-.16,1),(.5,.81)]:
        for x,z in outline:
            xx=x*sx*scale*(1+rng.uniform(-irregular,irregular))
            zz=z*sz*scale*(1+rng.uniform(-irregular,irregular))
            yy=level*sy
            # Controlled chipping preserves horizontal geological layers.
            if abs(level)<.45:yy+=rng.uniform(-.045,.045)*sy
            points.append((p[0]+xx*math.cos(angle)-zz*math.sin(angle),p[1]+yy,p[2]+xx*math.sin(angle)+zz*math.cos(angle)))
    faces=[tuple(reversed(range(8))),tuple(24+i for i in range(8))]
    faces += [(j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i) for j in range(3) for i in range(8)]
    geom(mat,points,faces)


def rod(points,radii,mat,sides=6):
    pts=[Vector(p) for p in points];vertices=[]
    if isinstance(radii,(int,float)):radii=[radii]*len(pts)
    for i,p in enumerate(pts):
        tangent=(pts[min(i+1,len(pts)-1)]-pts[max(i-1,0)]).normalized()
        axis=Vector((0,1,0))
        if abs(tangent.dot(axis))>.94:axis=Vector((0,0,1))
        cross=tangent.cross(axis).normalized();other=tangent.cross(cross).normalized()
        for j in range(sides):
            a=j*math.tau/sides
            vertices.append(tuple(p+radii[i]*(math.cos(a)*cross+math.sin(a)*other)))
    faces=[tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+i for i in range(sides))]
    faces += [(j*sides+i,j*sides+(i+1)%sides,(j+1)*sides+(i+1)%sides,(j+1)*sides+i) for j in range(len(pts)-1) for i in range(sides)]
    geom(mat,vertices,faces)


def guardian():
    set_part('GuardianBody')
    # The inner dark mass appears only in the crevices between real strata.
    stone((0,1.08,.015),(.70,.65,.46),OBSIDIAN,irregular=.05)
    rows=[(.79,.62,.13),(.935,.72,.145),(1.095,.84,.15),(1.255,.91,.17),(1.400,.66,.12)]
    for row,(y,width,height) in enumerate(rows):
        for k in range(3):
            x=(k-1)*width/3
            for face in [-1,1]:
                z=face*(.180+.025*(row%2))
                stone((x+RNG.uniform(-.014,.014),y,z),(width/3+.022,height,.175),LIMESTONE,angle=RNG.uniform(-.10,.10))
        for side in [-1,1]:
            stone((side*width*.43,y,.02),(.15,height*.88,.35),MOSS if row==4 else LIMESTONE,angle=side*.07)
    # Layered collar and separate squared head, with deep unlit eye sockets.
    stone((0,1.487,.015),(.39,.09,.35),OBSIDIAN)
    stone((0,1.62,-.015),(.47,.26,.39),OBSIDIAN,irregular=.04)
    stone((0,1.730,.005),(.48,.09,.41),LIMESTONE,irregular=.035)
    stone((-.140,1.690,-.205),(.23,.082,.105),LIMESTONE,angle=-.08)
    stone((.137,1.690,-.200),(.23,.087,.112),LIMESTONE,angle=.05)
    for sign in [-1,1]:
        stone((sign*.120,1.641,-.214),(.069,.025,.012),AMBER,irregular=.02)
        stone((sign*.172,1.578,-.157),(.135,.107,.14),LIMESTONE,angle=sign*.17)
        stone((sign*.217,1.619,.037),(.075,.172,.24),LIMESTONE,angle=sign*.10)
    stone((0,1.619,-.205),(.082,.124,.096),LIMESTONE,irregular=.03)
    stone((0,1.525,-.151),(.30,.063,.168),LIMESTONE)
    # Unequal crest chips avoid a factory-symmetric helmet silhouette.
    for x,y,z,s in [(-.15,1.783,.06,.048),(.075,1.777,.11,.041),(.19,1.754,-.01,.028)]:
        stone((x,y,z),(s*2,.034,s*2),MOSS,angle=x)
    # Thin mineral seams, deliberately restrained compared with the amber eyes.
    for x,y,z in [(-.16,1.016,-.266),(.25,1.203,-.261),(.03,.802,-.248)]:
        stone((x,y,z),(.038,.012,.017),OBSIDIAN,irregular=.1)
    for side,label in [(-1,'L'),(1,'R')]:
        set_part('GuardianArm'+label,(side*.49,1.30,0))
        stone((side*.505,1.31,.018),(.375,.285,.395),OBSIDIAN)
        for y,dx,width,height,depth in [(1.425,.51,.36,.14,.40),(1.296,.55,.36,.15,.43),(1.150,.60,.255,.15,.315),(1.013,.628,.24,.12,.27)]:
            stone((side*dx,y,.012),(width,height,depth),LIMESTONE,angle=side*.12)
        stone((side*.641,.912,.015),(.235,.14,.26),OBSIDIAN)
        for j in range(3):
            stone((side*(.657+j*.005),.837-j*.104,-.021),(.303-j*.011,.123,.32+j*.024),LIMESTONE,angle=side*(.02+j*.035))
        stone((side*.674,.507,-.048),(.292,.25,.338),OBSIDIAN)
        stone((side*.677,.563,.032),(.34,.17,.30),LIMESTONE,angle=side*.08)
        for j in range(3):
            x=side*.677+(j-1)*.097
            stone((x,.477,-.206),(.091,.15,.128),LIMESTONE,angle=side*.03)
            stone((x,.405,-.138),(.090,.07,.18),LIMESTONE)
        stone((side*.518,.543,-.109),(.103,.189,.17),LIMESTONE,angle=side*.21)
        for j in range(3):
            stone((side*(.58+j*.053),1.461,.035+j*.027),(.087,.045,.094),MOSS,angle=side*.17)
        set_part('GuardianLeg'+label,(side*.22,.74,0))
        stone((side*.22,.587,.010),(.277,.32,.29),OBSIDIAN)
        for y,width in [(.675,.29),(.548,.27),(.400,.30)]:
            stone((side*.22,y,-.018),(width,.137,.305),LIMESTONE,angle=side*.08)
        stone((side*.223,.264,.013),(.275,.224,.291),OBSIDIAN)
        for y in [.30,.195]:
            stone((side*.23,y,-.038),(.295,.124,.33),LIMESTONE,angle=-side*.035)
        stone((side*.235,.092,-.075),(.355,.184,.457),LIMESTONE,angle=side*.022,irregular=.04)
        for j in range(3):
            stone((side*.235+(j-1)*.102,.062,-.300),(.096,.124,.135),LIMESTONE,irregular=.05)


def membrane_triangle(a,b,c,mat,side,divisions=4):
    """Thick two-sided triangular skin, with broad hand-authored facets."""
    a,b,c=Vector(a),Vector(b),Vector(c)
    points=[];lookup={}
    for i in range(divisions+1):
        for j in range(divisions+1-i):
            u,v=i/divisions,j/divisions
            p=a*(1-u-v)+b*u+c*v
            p.y-=.055*math.sin(math.pi*u)*math.sin(math.pi*v)
            lookup[i,j]=len(points);points.append((p.x*side,p.y,p.z))
    faces=[]
    for i in range(divisions):
        for j in range(divisions-i):
            faces.append((lookup[i,j],lookup[i+1,j],lookup[i,j+1]))
            if i+j<divisions-1:faces.append((lookup[i+1,j],lookup[i+1,j+1],lookup[i,j+1]))
    n=len(points);points+= [(x,y-.012,z) for x,y,z in points]
    faces += [tuple(n+k for k in reversed(f)) for f in list(faces)]
    perimeter=[lookup[i,0] for i in range(divisions+1)]+[lookup[divisions-i,i] for i in range(1,divisions+1)]+[lookup[0,divisions-i] for i in range(1,divisions)]
    faces += [(perimeter[i],perimeter[(i+1)%len(perimeter)],perimeter[(i+1)%len(perimeter)]+n,perimeter[i]+n) for i in range(len(perimeter))]
    geom(mat,points,faces)


def cavewing():
    set_part('CavewingBody')
    # A tapered, plated thorax. Broad neck/head and compact tucked feet give a calm glider silhouette.
    for j,(z,width,height,y) in enumerate([(-.42,.20,.21,.035),(-.29,.28,.27,.025),(-.14,.38,.31,.008),(.02,.38,.31,0),(.18,.30,.25,-.016),(.31,.23,.20,-.025),(.42,.15,.15,-.032)]):
        stone((0,y,z),(width,height,.195),SCALE,irregular=.05)
        if j>0:
            stone((0,y+height*.42,z),(.105,height*.20,.139),OBSIDIAN,irregular=.06)
        for sign in [-1,1]:
            stone((sign*width*.37,y+height*.12,z-.014),(width*.30,height*.44,.152),SCALE,angle=sign*.16)
    rod([(0,-.005,.39),(0,-.025,.50),(0,-.014,.61),(0,.012,.68)],[.069,.048,.024,.003],SCALE,6)
    stone((0,.055,-.572),(.257,.215,.299),SCALE,irregular=.05)
    stone((0,.031,-.746),(.212,.130,.148),SCALE,irregular=.0)
    stone((0,-.032,-.730),(.190,.032,.180),OBSIDIAN,irregular=.0)
    for side in [-1,1]:
        stone((side*.111,.079,-.702),(.045,.046,.076),OBSIDIAN,angle=side*.22)
        stone((side*.130,.080,-.709),(.018,.023,.040),AMBER,irregular=.02)
        rod([(side*.102,.133,-.535),(side*.145,.253,-.486),(side*.181,.327,-.436),(side*.168,.357,-.379)],[.052,.038,.021,.002],OBSIDIAN,5)
        # Small angular ear and cheek plates support the head profile.
        stone((side*.154,.104,-.566),(.084,.125,.16),SCALE,angle=side*.25)
        rod([(side*.105,-.091,.187),(side*.176,-.196,.265),(side*.198,-.213,.375)],[.055,.045,.027],SCALE,5)
        for k in range(2):
            rod([(side*(.181+k*.036),-.216,.338),(side*(.181+k*.036),-.258,.282)],[.015,.006],OBSIDIAN,5)
    for side,label in [(-1,'L'),(1,'R')]:
        set_part('Wing'+label,(side*.18,0,-.08))
        hub=(1.12,.14,-.53)
        boundary=[(2,.04,-.38),(1.78,-.007,-.07),(1.54,-.045,.12),(1.50,-.095,.43),(1.30,-.115,.32),(1.12,-.122,.25),(.94,-.13,.66),(.71,-.15,.44),(.51,-.13,.36),(.31,-.09,.41),(.20,0,-.05)]
        for i in range(len(boundary)-1):membrane_triangle(hub,boundary[i],boundary[i+1],WINE if i%3 else RUST,side)
        membrane_triangle((.20,0,-.05),(.74,.10,-.46),hub,RUST,side)
        mirror=lambda pts:[(side*x,y,z) for x,y,z in pts]
        rod(mirror([(.16,0,-.09),(.45,.04,-.25),(.73,.09,-.46),hub,(1.55,.11,-.47),(1.99,.04,-.38)]),[.090,.074,.060,.058,.031,.002],SCALE,6)
        # Long segmented fingers visibly support every scalloped membrane bay.
        for tip in [boundary[3],boundary[6],boundary[9]]:
            t=Vector(tip);h=Vector(hub)
            pts=[]
            for i in range(5):
                u=i/4;p=h*(1-u)+t*u;p.y+=.026*math.sin(u*math.pi)
                pts.append(tuple(p))
            rod(mirror(pts),[.034,.030,.021,.012,.002],SCALE,5)
        rod(mirror(boundary),[.008]*len(boundary),SCALE,5)
        # Bony scales along the leading edge are individual chamfered segments.
        for j in range(7):
            u=j/6
            x=.34+u*1.07;z=-.19-u*.31;y=.076+u*.066
            stone((side*x,y,z),(.11,.064,.089),OBSIDIAN if j%3==0 else SCALE,angle=side*.3,irregular=.035)
        # Short wrist spur points aft/up, never reads as an attack claw.
        rod(mirror([(1.11,.17,-.54),(1.135,.25,-.56),(1.19,.281,-.49)]),[.034,.022,.002],OBSIDIAN,5)


def clear():
    for obj in list(bpy.data.objects):bpy.data.objects.remove(obj,do_unlink=True)
    BATCH.clear();PIVOTS.clear()


def make_objects(name):
    root=bpy.data.objects.new(name.title(),None);bpy.context.collection.objects.link(root)
    nodes={}
    for part,pivot in PIVOTS.items():
        obj=bpy.data.objects.new(part,None);bpy.context.collection.objects.link(obj)
        obj.parent=root;obj.location=vec(pivot);obj.empty_display_type='PLAIN_AXES';obj.empty_display_size=.12
        obj['game_pivot_xyz_m']=list(pivot);obj['local_axes']='Y up; -Z forward; L means negative X'
        if part.startswith('Wing'):obj['flap_axis']='local Z';obj['upstroke_sign']=1 if part.endswith('R') else -1
        nodes[part]=obj
    meshes=[]
    for (part,mat),(points,faces,flags) in BATCH.items():
        pivot=Vector(PIVOTS[part]);vertices=[vec(Vector(p)-pivot) for p in points]
        mesh=bpy.data.meshes.new(part+' | '+mat);mesh.from_pydata(vertices,[],faces);mesh.update()
        bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.recalc_face_normals(bm,faces=bm.faces);bm.to_mesh(mesh);bm.free();mesh.update()
        uv=mesh.uv_layers.new(name='Shared mineral UV')
        for poly in mesh.polygons:
            poly.use_smooth=False
            axes=[i for i in range(3) if i!=max(range(3),key=lambda j:abs(poly.normal[j]))]
            for li in poly.loop_indices:
                p=mesh.vertices[mesh.loops[li].vertex_index].co+vec(pivot)
                uv.data[li].uv=(p[axes[0]]*.7+.27,p[axes[1]]*.7+.43)
        obj=bpy.data.objects.new(part+' | '+mat,mesh);bpy.context.collection.objects.link(obj)
        obj.parent=nodes[part];mesh.materials.append(MATERIALS[mat]);bpy.context.view_layer.objects.active=obj
        modifier=obj.modifiers.new('Explicit triangulation','TRIANGULATE');bpy.ops.object.modifier_apply(modifier=modifier.name)
        meshes.append(obj)
    return root,nodes,meshes


def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()


def validate_binary(doc,binary):
    total=0
    def values(index):
        acc=doc['accessors'][index];view=doc['bufferViews'][acc['bufferView']]
        components={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[acc['type']]
        fmt,width={5126:('f',4),5123:('H',2),5125:('I',4)}[acc['componentType']]
        start=view.get('byteOffset',0)+acc.get('byteOffset',0)
        stride=view.get('byteStride',components*width)
        assert start+(acc['count']-1)*stride+components*width<=len(binary)
        return [struct.unpack_from('<'+fmt*components,binary,start+i*stride) for i in range(acc['count'])]
    for mesh in doc['meshes']:
        for primitive in mesh['primitives']:
            assert primitive.get('mode',4)==4
            indices=[v[0] for v in values(primitive['indices'])]
            positions=values(primitive['attributes']['POSITION'])
            normals=values(primitive['attributes']['NORMAL'])
            assert all(math.isfinite(v) for point in positions+normals for v in point)
            for i in range(0,len(indices),3):
                a,b,c=[Vector(positions[indices[i+j]]) for j in range(3)]
                assert (b-a).cross(c-a).length>1e-12,'Degenerate triangle'
            total+=len(indices)//3
    return {'triangles_verified':total,'finite_attributes':True,'degenerate_triangles':0,'buffer_bounds':True}


def export_asset(name,root,nodes,meshes):
    source=SOURCE/(name+'.blend')
    for image in bpy.data.images:
        if image.source=='FILE' and image.filepath:image.filepath=bpy.path.relpath(bpy.path.abspath(image.filepath),start=str(SOURCE))
    bpy.context.scene.unit_settings.system='METRIC'
    bpy.ops.wm.save_as_mainfile(filepath=str(source),compress=True)
    # Export shared images directly; restore the same explicit linear factor in glTF.
    for mat in MATERIALS.values():
        if mat.name in FACTORS:
            mat.node_tree.links.new(mat.node_tree.nodes['Shared texture'].outputs['Color'],mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'])
    temp=WORK/name;temp.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(temp/(name+'.gltf')),export_format='GLTF_SEPARATE',export_yup=True,
        export_apply=False,export_animations=False,export_cameras=False,export_lights=False,export_extras=True)
    for mat in MATERIALS.values():
        if mat.name in FACTORS:mat.node_tree.links.new(mat.node_tree.nodes['Authored mineral tint'].outputs[0],mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'])
    doc=json.loads((temp/(name+'.gltf')).read_text());assert len(doc['buffers'])==1
    binary=(temp/doc['buffers'][0].pop('uri')).read_bytes()
    for image in doc.get('images',[]):
        image['uri']='../textures/'+Path(image['uri']).name
        assert (OUT/image['uri']).is_file()
    for mat in doc['materials']:
        if mat['name'] in FACTORS:mat['pbrMetallicRoughness']['baseColorFactor']=[*FACTORS[mat['name']],1]
    validation=validate_binary(doc,binary)
    doc['asset']['extras']={'generator_sha256':sha(Path(__file__)),'units':'metres','game_axes':'Y up; forward -Z; L is negative X'}
    raw=json.dumps(doc,separators=(',',':')).encode();raw+=b' '*((-len(raw))%4);binary+=b'\0'*((-len(binary))%4)
    body=struct.pack('<II',len(raw),0x4e4f534a)+raw+struct.pack('<II',len(binary),0x004e4942)+binary
    glb=OUT/(name+'.glb');glb.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
    triangle_count=sum(len(obj.data.polygons) for obj in meshes)
    points=[obj.matrix_world@v.co for obj in meshes for v in obj.data.vertices]
    # matrix_world is evaluated explicitly before measuring articulated world bounds.
    bpy.context.view_layer.update()
    points=[obj.matrix_world@v.co for obj in meshes for v in obj.data.vertices]
    bounds=[[round(min(p[i] for p in points),6),round(max(p[i] for p in points),6)] for i in [0,2,1]]
    bounds[2]=[-bounds[2][1],-bounds[2][0]]
    assert triangle_count<15000,(name,triangle_count)
    assert len(doc['materials'])<=5,(name,len(doc['materials']))
    required=['GuardianArmL','GuardianArmR','GuardianLegL','GuardianLegR'] if name=='guardian' else ['WingL','WingR']
    assert all(sum(node.get('name')==part for node in doc['nodes'])==1 for part in required)
    return {'file':glb.relative_to(ROOT).as_posix(),'sha256':sha(glb),'bytes':glb.stat().st_size,
        'source':source.relative_to(ROOT).as_posix(),'source_sha256':sha(source),'triangles':triangle_count,
        'meshes':len(doc['meshes']),'materials':len(doc['materials']),'bounds_xyz_m':bounds,
        'pivots_game_xyz_m':{part:list(PIVOTS[part]) for part in required},
        'animation':{'axis':'local Z','upstroke_sign':{'WingL':-1,'WingR':1},'suggested_angle_radians':[-.24,.48]} if name=='cavewing' else {'arms':'local X; swing around shoulder','legs':'local X; swing around hip'},
        'external_images':[im['uri'] for im in doc.get('images',[])],'validation':validation}


def area(name,position,power,color,size,target):
    data=bpy.data.lights.new(name,'AREA');data.energy=power;data.color=color;data.shape='DISK';data.size=size
    obj=bpy.data.objects.new(name,data);bpy.context.collection.objects.link(obj);obj.location=vec(position)
    obj.rotation_euler=(vec(target)-obj.location).to_track_quat('-Z','Y').to_euler()


def preview():
    clear()
    rig_report={}
    placements={'guardian':(-1.64,0,.13),'cavewing':(.98,1.18,0)}
    for name in placements:
        bpy.ops.import_scene.gltf(filepath=str(OUT/(name+'.glb')))
        selected=list(bpy.context.selected_objects)
        roots=[obj for obj in selected if obj.parent is None]
        assert len(roots)==1,(name,[obj.name for obj in roots])
        root=roots[0]
        bpy.context.view_layer.update()
        required=['GuardianArmL','GuardianArmR','GuardianLegL','GuardianLegR'] if name=='guardian' else ['WingL','WingR']
        rigs={part:next(obj for obj in selected if obj.name==part) for part in required}
        rig_report[name]={}
        for part,joint in rigs.items():
            joint.rotation_mode='XYZ'
            descendants=[obj for obj in joint.children_recursive if obj.type=='MESH']
            assert descendants,part
            assert Vector(joint.rotation_euler).length<1e-6,part
            candidate=max([(obj,v.co.copy()) for obj in descendants for v in obj.data.vertices],key=lambda pair:abs((pair[0].matrix_world@pair[1]).x))
            mesh,local=candidate;before=mesh.matrix_world@local;pivot=joint.matrix_world.translation.copy()
            # Blender -Y is game +Z; prove the specified flap direction after import.
            if part.startswith('Wing'):joint.rotation_euler.y=.35 if part.endswith('L') else -.35
            else:joint.rotation_euler.x=.25
            bpy.context.view_layer.update();after=mesh.matrix_world@local
            assert abs((after-pivot).length-(before-pivot).length)<1e-5
            if part.startswith('Wing'):assert after.z>before.z+.2,part
            joint.rotation_euler=(0,0,0);bpy.context.view_layer.update()
            rig_report[name][part]={'rigid_pivot_verified':True,'children':len(descendants)}
        root.location+=vec(placements[name])
        if name=='guardian':root.rotation_euler[2]=math.radians(12)
    # Source-only floor for the inspection render; not present in either GLB.
    floor=bpy.data.materials.new('Review floor');floor.diffuse_color=(.016,.024,.021,1);floor.use_nodes=True
    floor.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(.016,.024,.021,1)
    floor.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.87
    bpy.ops.mesh.primitive_plane_add(size=200,location=(0,0,-.015));bpy.context.object.data.materials.append(floor)
    area('Soft mineral key',(-3,5,-5),440,(.81,.93,.88),4,(0,.8,0))
    area('Cool cave bounce',(3,3,-2),290,(.42,.69,.76),3,(.5,1,0))
    area('Low amber rim',(1,2,3),460,(1,.38,.15),3,(0,1,0))
    area('Upper rim',(-2,4,2),340,(.47,.65,.38),3,(-1,1,0))
    scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=48;scene.cycles.use_denoising=True
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='CUDA';prefs.get_devices()
        gpu=False
        for device in prefs.devices:device.use=device.type=='CUDA';gpu=gpu or device.use
        if gpu:scene.cycles.device='GPU'
    except Exception:pass
    scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.06,.095,.085,1)
    scene.world.node_tree.nodes['Background'].inputs[1].default_value=.32
    data=bpy.data.cameras.new('Fauna review');camera=bpy.data.objects.new('Fauna review',data);bpy.context.collection.objects.link(camera)
    camera.location=vec((3.2,6.0,-9.5));target=vec((.02,.95,0))
    camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler();data.type='ORTHO';data.ortho_scale=6.12;scene.camera=camera
    scene.render.resolution_x=1800;scene.render.resolution_y=1000;scene.render.resolution_percentage=100
    scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(ROOT/'.dream-loop/fauna-kit.png')
    bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/'presentation.blend'),compress=True)
    bpy.ops.render.render(write_still=True)
    return rig_report


def main():
    manifest={'generator':'tools/art/build_fauna.py','generator_sha256':sha(Path(__file__)),'units':'metres',
        'axes':'Y up; forward -Z; L is negative X; all pivot local axes match game axes',
        'shared_images':[{'file':'assets/textures/'+name,'sha256':sha(ROOT/'assets/textures'/name)} for name in ['limestone-world.png','stone.png']],
        'assets':{}}
    for name,build in [('guardian',guardian),('cavewing',cavewing)]:
        clear();build();root,nodes,meshes=make_objects(name)
        manifest['assets'][name]=export_asset(name,root,nodes,meshes)
        print('FAUNA_ASSET',json.dumps(manifest['assets'][name]),flush=True)
    manifest['reimported_rig_validation']=preview()
    (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('FAUNA_KIT_COMPLETE',flush=True)


if __name__=='__main__':main()
