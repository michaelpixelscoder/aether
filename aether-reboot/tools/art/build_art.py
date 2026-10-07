"""Authored voxel art kit. Blender 5.2, background mode, no external asset fetch.

Coordinates below are game coordinates (Y up); conversion happens at export.
Each asset is merged by material, with real chamfers, UVs and weighted normals.
Run: blender --background --python tools/art/build_art.py
"""
import bpy, math, random, json, struct, hashlib
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/art"
OUT.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
MATERIALS = {}
BATCH = {}
RNG = random.Random(7391)

def material(name, color, metal=0, rough=.75, texture=None, emission=0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Metallic'].default_value = metal
    bsdf.inputs['Roughness'].default_value = rough
    if texture:
        t = m.node_tree.nodes.new('ShaderNodeTexImage')
        t.image = bpy.data.images.load(str(ROOT / texture), check_existing=True)
        m.node_tree.links.new(t.outputs['Color'], bsdf.inputs['Base Color'])
    if emission:
        bsdf.inputs['Emission Color'].default_value = (*color, 1)
        bsdf.inputs['Emission Strength'].default_value = emission
    MATERIALS[name] = m
    return name

WOOD = material('Cedar | grain', (.35,.16,.055), texture='assets/textures/cedar.png')
OAK = material('Dark oak end grain', (.115,.065,.031))
GOLD = material('Aged brass', (.53,.29,.075), .72, .34)
EDGE = material('Polished brass edge', (.78,.52,.16), .66, .28)
IRON = material('Blued iron', (.045,.068,.105), .72, .4)
ROTOR = material('Propeller rotor', (.56,.31,.09), .7, .31)
ROPE = material('Hemp cordage', (.49,.32,.15), 0, .95)
NAVY = material('Indigo canvas', (.004,.013,.04), 0, .92, texture='assets/textures/indigo-canvas.png')
NAVY2 = material('Indigo canvas variation', (.006,.018,.051), 0, .9, texture='assets/textures/indigo-canvas.png')
IVORY = material('Embroidered gold thread', (.60,.35,.075), .18, .72)
PURPLE = material('Aether crystal', (.34,.06,.8), .25, .23, emission=2.2)
LIGHT = material('Lantern amber glass', (1,.39,.065), 0, .3, emission=3)
STONE = [material('Limestone stratum '+str(i), (.29+i*.025,.31+i*.023,.32+i*.019), 0, .95, texture='assets/textures/stone.png') for i in range(5)]
MOSS = [material('Meadow foliage '+str(i), (.11+i*.018,.20+i*.028,.036+i*.005), 0, 1) for i in range(5)]
STONE_TEX = material('Cut stone', (.4,.4,.4), texture='assets/textures/stone.png')
GRASS = material('Meadow turf', (.3,.4,.1), texture='assets/textures/grass.png')
FLOWERS = [material('Meadow flower '+str(i),c,0,.85) for i,c in enumerate([(.68,.43,.12),(.42,.13,.62),(.74,.72,.58)])]

def geom(mat, points, faces, bevel=0):
    key = (mat, bevel)
    v, f, uv = BATCH.setdefault(key, ([], [], []))
    offset = len(v)
    v.extend([(x,-z,y) for x,y,z in points])
    for face in faces:
        f.append([offset+i for i in face])
        uv.append([(points[i][0]*.8,points[i][1]*.8) for i in face] if mat in [NAVY,NAVY2] else [(0,0),(1,0),(1,1),(0,1)][:len(face)])

def box(p, size, mat, bevel=.012):
    x,y,z=p; a,b,c=[n/2 for n in size]
    pts=[(x+sx*a,y+sy*b,z+sz*c) for sx,sy,sz in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
    geom(mat,pts,[(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)],min(bevel,min(size)*.15))

def beam(a,b,r,mat,n=8):
    a,b=Vector(a),Vector(b); d=(b-a).normalized()
    tangent=d.cross(Vector((0,1,0)))
    if tangent.length<.01: tangent=d.cross(Vector((1,0,0)))
    tangent.normalize(); bitangent=d.cross(tangent)
    pts=[tuple(p+r*(math.cos(i*math.tau/n)*tangent+math.sin(i*math.tau/n)*bitangent)) for p in [a,b] for i in range(n)]
    faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]
    faces.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n))
    geom(mat,pts,faces)

def ring(p,r,t,mat,axis='z',n=24):
    p=Vector(p)
    def point(a):
        return p+ (Vector((math.cos(a)*r,math.sin(a)*r,0)) if axis=='z' else Vector((math.cos(a)*r,0,math.sin(a)*r)))
    for i in range(n): beam(point(i*math.tau/n),point((i+1)*math.tau/n),t,mat,6)

def crystal(p,size):
    x,y,z=p; a,b,c=size
    points=[(x,y+b/2,z),(x,y-b/2,z)]+[(x+math.cos(i*math.tau/6)*a/2,y,z+math.sin(i*math.tau/6)*c/2) for i in range(6)]
    geom(PURPLE,points,[(0,2+i,2+(i+1)%6) for i in range(6)]+[(1,2+(i+1)%6,2+i) for i in range(6)])

def export(name):
    for obj in list(bpy.data.objects): bpy.data.objects.remove(obj,do_unlink=True)
    for (mat, bevel),(vertices,faces,uvs) in BATCH.items():
        mesh=bpy.data.meshes.new(mat)
        mesh.from_pydata(vertices,[],faces); mesh.update()
        uv=mesh.uv_layers.new(name='UVMap')
        for poly,coords in zip(mesh.polygons,uvs):
            for j,li in enumerate(poly.loop_indices): uv.data[li].uv=coords[j%len(coords)]
        obj=bpy.data.objects.new(mat,mesh); bpy.context.collection.objects.link(obj)
        obj.data.materials.append(MATERIALS[mat]); bpy.context.view_layer.objects.active=obj
        if bevel:
            mod=obj.modifiers.new('Crafted edge chamfer','BEVEL'); mod.width=bevel; mod.segments=1
            bpy.ops.object.modifier_apply(modifier=mod.name)
            mod=obj.modifiers.new('Weighted corner normals','WEIGHTED_NORMAL'); mod.keep_sharp=True
            bpy.ops.object.modifier_apply(modifier=mod.name)
        if name=='sail' and mat in [NAVY,NAVY2,IVORY]:
            # Shape keys retain the authored voxel cloth. The same displacement
            # field moves canvas and embroidery; the spars/rigging never morph.
            basis=[v.co.copy() for v in obj.data.vertices]
            obj.shape_key_add(name='Basis',from_mix=False)
            for key_name in ['WindPressure','RippleSin','RippleCos']:
                key=obj.shape_key_add(name=key_name,from_mix=False)
                key.slider_min=-1.0; key.slider_max=1.0
                for index,vertex in enumerate(key.data):
                    vertex.co=basis[index]
                    x,y=vertex.co.x,vertex.co.z # Blender Z is game Y.
                    u=max(0,min(1,(x+1.25)/2.5));v=max(0,min(1,(y+1.15)/2.30))
                    envelope=math.sin(math.pi*u)*math.sin(math.pi*v)
                    wave=3.7*x+1.9*y
                    delta=(.18 if key_name=='WindPressure' else .065*(math.sin(wave) if key_name=='RippleSin' else math.cos(wave)))*envelope
                    vertex.co.y-=delta # Blender -Y is game Z.
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/(name+'.blend')),compress=True)
    bpy.ops.export_scene.gltf(filepath=str(OUT/(name+'.glb')),export_format='GLB',export_yup=True,export_apply=False,export_morph=True,export_morph_normal=True,export_image_format='AUTO',export_animations=False,export_cameras=False,export_lights=False)
    # glTF defines baseColorFactor in linear space. Preserve a deliberate tint
    # alongside the image texture; Blender's linked Color socket exports white.
    file=OUT/(name+'.glb'); data=file.read_bytes()
    length=struct.unpack_from('<I',data,12)[0]
    doc=json.loads(data[20:20+length])
    doc['asset']['extras']={'aether_geometry_sha256':hashlib.sha256((ROOT/'tools/art/scene.json').read_bytes()).hexdigest(),'aether_generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    factors={'Meadow turf':(.36,.7,.32,1),'Cedar | grain':(.70,.62,.52,1),'Cut stone':(.7,.75,.8,1),'Indigo canvas':(.30,.44,.72,1),'Indigo canvas variation':(.33,.47,.75,1)}
    for m in doc.get('materials',[]):
        if m['name'] in factors: m['pbrMetallicRoughness']['baseColorFactor']=factors[m['name']]
        if m['name'].startswith('Limestone stratum'):
            i=int(m['name'].split()[-1]);m['pbrMetallicRoughness']['baseColorFactor']=(.47+i*.025,.51+i*.023,.58+i*.019,1)
    encoded=json.dumps(doc,separators=(',',':')).encode();encoded+=b' '*((-len(encoded))%4)
    body=struct.pack('<II',len(encoded),0x4e4f534a)+encoded+data[20+length:]
    file.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
    print('ART_ASSET',name,len(bpy.data.objects),sum(len(o.data.polygons) for o in bpy.data.objects if o.type=='MESH'),flush=True)
    BATCH.clear()

def sail():
    # A tessellated, slightly bellied square sail with an embroidered compass rose.
    def triangle_contains(x,y,a,b,c):
        def cross(p,q): return (x-q[0])*(p[1]-q[1])-(p[0]-q[0])*(y-q[1])
        signs=[cross(a,b),cross(b,c),cross(c,a)]
        return min(signs)>=0 or max(signs)<=0
    points=[]
    for i in range(8):
        angle=i*math.pi/4
        tip=(math.sin(angle)*(.90 if i%2==0 else .72),math.cos(angle)*(.90 if i%2==0 else .72))
        a=(math.sin(angle-.39)*.21,math.cos(angle-.39)*.21)
        b=(math.sin(angle+.39)*.21,math.cos(angle+.39)*.21)
        points.append((tip,a,b))
    for iy in range(40):
        for ix in range(40):
            x=(ix+.5)/40*2.5-1.25; y=(iy+.5)/40*2.30-1.15
            u=(ix+.5)/40;v=(iy+.5)/40
            belly=math.sin(u*math.pi)*math.sin(v*math.pi)
            z=-.24*belly + .022*math.sin(u*math.pi*6)*belly
            y-=.055*math.sin(u*math.pi)*(1-v)**2
            r=math.hypot(x,y)
            emblem=abs(r-.60)<.033 or r<.16 or any(triangle_contains(x,y,*p) for p in points)
            mat=IVORY if emblem else NAVY if (ix+iy)%3 else NAVY2
            box((x,y,z),(.0625,.0575,.031),mat,.0007)
    beam((0,-1.49,.13),(0,1.65,.13),.065,WOOD)
    for y in [-1.22,1.23]:
        beam((-1.37,y,0),(1.37,y,0),.055,WOOD)
        for x in [-1.31,-.7,0,.7,1.31]:
            beam((x-.026,y,0),(x+.026,y,0),.067,GOLD)
    for x in [-1.3,1.3]:
        beam((x,1.25,0),(0,1.61,.13),.015,ROPE,6)
        beam((x,1.25,0),(x,-1.24,0),.012,ROPE,6)
    for y in [-1.4,-.4,.5,1.5]: beam((0,y-.04,.13),(0,y+.04,.13),.076,GOLD)
    # Reinforced sail corners and narrow sewn border, following the cloth plane.
    for x in [-1.22,1.22]: beam((x,-1.12,-.019),(x,1.12,-.019),.009,IVORY,4)
    for y in [-1.12,1.12]: beam((-1.22,y,-.019),(1.22,y,-.019),.009,IVORY,4)
    for x in [-1.15,1.15]:
        for y in [-1.07,1.07]: ring((x,y,-.027),.025,.008,GOLD,'z',8)
    for i in range(7): box((.11+i*.09,1.50-i*.018,.14+math.sin(i)*.03),(.091,.19,.02),NAVY,.003)
    box((0,1.69,.13),(.14,.14,.14),EDGE)
    export('sail')

def machinery(kind):
    if kind=='helm':
        box((0,-.11,.02),(.16,.22,.12),WOOD)
        ring((0,.025,-.02),.145,.018,GOLD)
        for i in range(8):
            a=i*math.tau/8
            beam((0,.025,-.02),(.19*math.cos(a),.025+.19*math.sin(a),-.02),.011,WOOD)
        beam((0,.025,-.07),(0,.025,.08),.034,EDGE)
    elif kind=='tank':
        beam((0,-.4,0),(0,.4,0),.18,IRON,12)
        for i in range(8):
            a=i*math.tau/8; x,z=.155*math.cos(a),.155*math.sin(a)
            beam((x,-.32,z),(x,.32,z),.033,PURPLE,6)
            beam((x*1.2,-.41,z*1.2),(x*1.2,.41,z*1.2),.014,GOLD,6)
        for y in [-.4,-.22,.22,.4]: ring((0,y,0),.19,.021,GOLD,'y',12)
        for y in [-.435,.435]: beam((0,y-.012,0),(0,y+.012,0),.15,EDGE,12)
        ring((0,.13,-.197),.055,.009,EDGE,'z',12)
        beam((0,.13,-.208),(.035,.16,-.21),.006,IVORY,4)
    elif kind=='lift':
        # Transverse gimbal: the real lift crystal is readable from either flank.
        # The ornamental rim projects 10 cm beyond the compact functional core.
        box((0,-.16,0),(.38,.065,.38),IRON)
        for x in [-.17,.17]:
            for i in range(16):
                a=i*math.tau/16;b=(i+1)*math.tau/16
                beam((x,.015+math.cos(a)*.255,math.sin(a)*.255),(x,.015+math.cos(b)*.255,math.sin(b)*.255),.039,GOLD,6)
                if i%2==0: box((x,.015+math.cos(a)*.255,math.sin(a)*.255),(.085,.085,.085),EDGE,.008)
        for i in range(8):
            a=i*math.tau/8
            y,z=.015+math.cos(a)*.235,math.sin(a)*.235
            beam((-.17,y,z),(.17,y,z),.023,IRON,6)
        crystal((0,.015,0),(.27,.36,.27))
    else:
        box((0,-.07,.06),(.18,.06,.43),WOOD)
        beam((0,0,-.27),(0,0,.24),.067,IRON,10)
        for z in [-.25,-.16,.09,.2]: ring((0,0,z),.069,.014,GOLD,'z',10)
        beam((-.078,0,.1),(.078,0,.1),.035,GOLD)
        beam((0,0,-.34),(0,0,-.27),.028,EDGE,4)
        box((0,.05,.21),(.13,.06,.1),PURPLE)
    export(kind)

def authored_environment(name,pieces):
    for i,piece in enumerate(pieces):
        p=piece['center']; size=piece['size']; surface=piece['surface']
        x,y,z=p; a,b,c=size
        if surface=='rock':
            if name.startswith('island') and y<3.0:
                count=math.ceil(b/1.6); h=b/count
                # Split exposed faces into hand-cut strata and fractured ledges;
                # shallow recesses remain within the shared collider envelope.
                for j in range(count):
                    segments=2 if a>=2 else 1
                    for k in range(segments):
                        width=a/segments; inset=RNG.uniform(.015,.065)
                        box((x-a/2+(k+.5)*width,y+b/2-(j+.5)*h,z),(width-.035,h-.025,c-inset),STONE[(i+j+k)%5],.045)
            else: box(p,size,NAVY if y>=14 else STONE_TEX if y>=2 else STONE[i%5],.045)
        elif surface=='grass':
            box(p,size,GRASS,.035)
            for k in range(12):
                px,pz=x+RNG.uniform(-a*.45,a*.45),z+RNG.uniform(-c*.45,c*.45)
                stem=RNG.uniform(.12,.27)
                box((px,y+b/2+stem/2,pz),(.035,stem,.035),RNG.choice(MOSS),.004)
                if k%3==0:
                    for dx,dz in [(-.035,0),(.035,0),(0,-.035),(0,.035)]:
                        box((px+dx,y+b/2+stem,pz+dz),(.055,.025,.055),FLOWERS[(i+k)%3],.003)
                else: box((px+.03,y+b/2+.08,pz),(.08,.035,.06),RNG.choice(MOSS),.004)
        elif surface=='trunk': box(p,size,WOOD,.055)
        elif surface=='foliage':
            for ix in range(3):
                for iz in range(3):
                    s=a/3; yy=y+RNG.uniform(-.04,.04)
                    box((x+(ix-1)*s,yy,z+(iz-1)*s),(s*.99,b*.93,s*.99),RNG.choice(MOSS),.05)
            # Hanging shoots break up the rigid canopy contour without hiding it.
            for k in range(4):
                px=x+RNG.choice([-1,1])*a*.42;pz=z+RNG.uniform(-c*.4,c*.4)
                box((px,y-b*.48-.13,pz),(.12,.42,.13),RNG.choice(MOSS),.015)
        elif surface=='crystal': crystal(p,size)
    export(name)

def propeller():
    box((0,-.195,0),(.43,.065,.43),IRON,.018)
    beam((0,0,-.19),(0,0,.16),.09,IRON,16)
    for z in [-.16,-.04,.08]:
        ring((0,0,z),.096,.016,GOLD,'z',16)
    ring((0,0,.195),.224,.012,IRON,'z',32)
    for x in [-.15,.15]:
        beam((x,-.18,-.15),(x,0,.185),.014,GOLD)
        for z in [-.16,.12]: box((x,-.15,z),(.036,.025,.035),EDGE,.004)
    crystal((0,.115,-.08),(.11,.10,.14))
    # All rotor geometry pivots about local Z at the component origin.
    beam((0,0,.15),(0,0,.23),.040,ROTOR,12)
    for i in range(4):
        angle=i*math.tau/4
        shape=[(.03,-.018,.17),(.17,-.037,.185),(.208,-.012,.215),(.18,.018,.22),(.065,.022,.19)]
        points=[(x*math.cos(angle)-y*math.sin(angle),x*math.sin(angle)+y*math.cos(angle),z) for x,y,z in shape]
        geom(ROTOR,points,[(0,1,2,3,4),(4,3,2,1,0)])
    export('propeller')

def anchor():
    # A faceted magnetic beacon in a forged armillary cage, with a visible aim core.
    crystal((0,0,0),(.8,1.25,.8))
    for r in [.72,.81]: ring((0,0,0),r,.047,GOLD,'z',24)
    ring((0,0,0),.86,.035,IRON,'y',24)
    for i in range(8):
        a=i*math.tau/8;x,y=math.cos(a),math.sin(a)
        beam((x*.61,y*.61,0),(x*1.10,y*1.10,0),.053,EDGE,6)
        box((x*.83,y*.83,0),(.13,.13,.16),IRON,.012)
    for z in [-.38,.38]: ring((0,0,z),.53,.025,GOLD,'z',16)
    export('anchor')

anchor()
propeller()
for kind in ['helm','tank','lift','harpoon']: machinery(kind)
sail()
for name,pieces in json.loads((ROOT/'tools/art/scene.json').read_text(encoding='utf-8-sig')).items(): authored_environment(name,pieces)
# Editable Blender sources are tools/art/source, not shipped runtime assets.
sources=ROOT/'tools/art/source'; sources.mkdir(exist_ok=True)
for f in OUT.glob('*.blend'): f.replace(sources/f.name)
print('Art kit complete',flush=True)
