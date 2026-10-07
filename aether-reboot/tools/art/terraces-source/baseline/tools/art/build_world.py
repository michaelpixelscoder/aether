"""Aether Isles monumental environment kit, authored in metric Y-up coordinates.

Blender 5.2: blender --background --python tools/art/build_world.py [-- dawn crystal]
Nine distinct compositions derived from the project's visual boards 25-41.
Geometry, colliders and effect anchors share a single authored coordinate system.
The main landing pier is always 14 x 1 x 18 m at (0,0,80); its top is y=.5.
The cavern channel is genuinely empty, not a solid island with a dark decal.
"""
import bpy, math, random, json, hashlib, struct, sys
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'assets/world'
SOURCE = ROOT / 'tools/art/world-source'
PREVIEW = ROOT / 'tools/art/world-preview'
for directory in [OUT, SOURCE, PREVIEW]: directory.mkdir(parents=True, exist_ok=True)
NAMES = ['dawn', 'crystal', 'nomad', 'ember', 'frost', 'verdant', 'storm', 'hollow', 'underforge']
BATCH = {}
COLLISIONS = []
LANDMARKS = {}
MATERIALS = {}
COLORS = {}
RNG = random.Random(74019)
CURRENT = ''
TERRAIN = {}
LOD = False
ANIMATED = []
ROCK_ALBEDO = 'assets/textures/fractured-rock-r41-albedo.png'
ROCK_NORMAL = 'assets/textures/fractured-rock-r41-normal.png'
ROCK_ROUGHNESS = 'assets/textures/fractured-rock-r41-roughness-v2.png'
ROCK_TILE_METRES = 16.0
ROCK_NORMAL_SCALE = .65
# ImageGen output is copied without repainting. These measured linear-channel
# gains preserve the old rock texture's mean reflected colour under r40 light.
ROCK_ALBEDO_GAIN = (1.5771665684158558,1.52613553543546,1.4566048841559818)

def rock_factor(color):return tuple(c*g for c,g in zip(color,ROCK_ALBEDO_GAIN))

MASONRY_ALBEDO = 'assets/textures/dressed-masonry-r42-albedo.png'
MASONRY_NORMAL = 'assets/textures/dressed-masonry-r42-normal.png'
MASONRY_ROUGHNESS = 'assets/textures/dressed-masonry-r42-roughness.png'
MASONRY_TILE_METRES = 9.0
MASONRY_NORMAL_SCALE = .4
# Six dressed courses per tile: approximately 1.5m high in authored metres.
# Linear-channel compensation preserves the previous STONE mean reflectance.
MASONRY_ALBEDO_GAIN = (1.2408519164957788,1.2179610548857251,1.169802004734367)

def masonry_factor(color):return tuple(c*g for c,g in zip(color,MASONRY_ALBEDO_GAIN))

def ground_level(x,z):
    if abs(x)<25 and z>34:return 0
    if CURRENT in ['hollow','underforge']:
        if 20<abs(x)<36 and -32<z<32:return 0 if x<0 else 7
        return (14 if z<-18 else 0) if x<0 else (23 if z<-12 else 7)
    if (-60<x<-23 and abs(z-24)<12) or (23<x<60 and abs(z+4)<12):return 0
    if CURRENT=='dawn':return 7 if x<-30 and z<40 else 16 if x>33 and z<22 else 0
    if CURRENT=='crystal':return 22 if z<-38 else 10 if x>31 or x<-37 else 0
    if CURRENT=='nomad':return 8 if x<-32 else 18 if x>30 and z<12 else 0
    if CURRENT=='ember':return 14 if x<-29 else 25 if x>32 and z<8 else 0
    if CURRENT=='frost':return 18 if x<-35 else 9 if x>35 else 0
    if CURRENT=='verdant':return 13 if x<-35 else 6 if x>30 else 0
    if CURRENT=='storm':return 22 if x>34 else 9 if x<-32 else 0
    return 0

def coastline(x,z):
    a=math.atan2(z,x);r=math.hypot(x,z)
    if abs(x)<=8 and 50<z<=72:return True
    if CURRENT=='dawn' and any(math.hypot(x-px,z-pz)<10 for px,pz in [(45,-51),(-56,-30),(-58,37)]):return True
    if CURRENT=='nomad' and math.hypot(x+61,z-13)<10:return True
    if CURRENT=='ember' and math.hypot(x+48,z-43)<14:return True
    if CURRENT=='dawn':limit=69+6*math.sin(a*3+.8)+3*math.sin(a*7)
    elif CURRENT=='crystal':limit=59+12*abs(math.sin(a*3+.6))+5*math.cos(a*5)
    elif CURRENT=='nomad':limit=62+10*math.sin(a*2-.4)+6*math.cos(a*5)
    elif CURRENT=='ember':limit=64+9*math.cos(a*4+.8)+4*math.sin(a*7)
    elif CURRENT=='frost':limit=64+10*abs(math.cos(a*2))+3*math.sin(a*9)
    elif CURRENT=='verdant':limit=64+8*math.sin(a*3)+5*math.cos(a*2)
    elif CURRENT=='storm':limit=57+15*abs(math.sin(a*3+1))+4*math.cos(a*11)
    else:limit=67+7*math.sin(a*3+.4)+3*math.cos(a*7)
    return r<=limit

def clear():
    ANIMATED.clear()
    for obj in list(bpy.data.objects): bpy.data.objects.remove(obj, do_unlink=True)
    for mesh in list(bpy.data.meshes):
        if mesh.users == 0: bpy.data.meshes.remove(mesh)

def vec(p): return Vector((p[0], -p[2], p[1]))

def material(name, color, texture=None, metallic=0, roughness=.88, emission=0, normal_texture=None, roughness_texture=None, normal_scale=ROCK_NORMAL_SCALE):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Metallic'].default_value = metallic
    bsdf.inputs['Roughness'].default_value = roughness
    if texture:
        tex = mat.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image = bpy.data.images.load(str(ROOT / texture), check_existing=True)
        mix = mat.node_tree.nodes.new('ShaderNodeMixRGB')
        mix.blend_type = 'MULTIPLY'
        mix.inputs[0].default_value = 1
        mix.inputs[2].default_value = (*color, 1)
        mat.node_tree.links.new(tex.outputs['Color'], mix.inputs[1])
        mat.node_tree.links.new(mix.outputs[0], bsdf.inputs['Base Color'])
    if normal_texture:
        tex=mat.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image=bpy.data.images.load(str(ROOT/normal_texture),check_existing=True)
        tex.image.colorspace_settings.name='Non-Color'
        normal=mat.node_tree.nodes.new('ShaderNodeNormalMap')
        normal.inputs['Strength'].default_value=normal_scale
        mat.node_tree.links.new(tex.outputs['Color'],normal.inputs['Color'])
        mat.node_tree.links.new(normal.outputs['Normal'],bsdf.inputs['Normal'])
    if roughness_texture:
        tex=mat.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image=bpy.data.images.load(str(ROOT/roughness_texture),check_existing=True)
        tex.image.colorspace_settings.name='Non-Color'
        channels=mat.node_tree.nodes.new('ShaderNodeSeparateColor');channels.mode='RGB'
        mat.node_tree.links.new(tex.outputs['Color'],channels.inputs['Color'])
        mat.node_tree.links.new(channels.outputs['Green'],bsdf.inputs['Roughness'])
    if emission:
        bsdf.inputs['Emission Color'].default_value = (*color, 1)
        bsdf.inputs['Emission Strength'].default_value = emission
    mat.diffuse_color = (*color, 1)
    MATERIALS[name] = mat
    COLORS[name] = color
    return name

def palette(kind):
    global ROCK, ROCK_DARK, STONE, TRIM, GOLD, WOOD, ROOF, LEAF, LEAF_LIGHT, GLOW, CRYSTAL, SOIL
    MATERIALS.clear(); COLORS.clear()
    rock = (.64,.69,.72); stone = (.88,.87,.79); turf = (.095,.19,.032)
    roof = (.012,.053,.11); foliage = (.065,.14,.018); leaflight = (.13,.23,.027)
    glow = (1,.38,.045); crystal = (.02,.18,.82)
    if kind == 'nomad': rock=(.75,.49,.25);stone=(1,.77,.41);turf=(.50,.36,.16);roof=(.60,.16,.035)
    if kind in ['ember','underforge']: rock=(.19,.17,.19);stone=(.32,.28,.25);turf=(.13,.115,.105);roof=(.16,.14,.14);crystal=(.26,.018,.69)
    if kind == 'frost': rock=(.34,.53,.67);stone=(.71,.87,1);turf=(.72,.88,.99);roof=(.055,.13,.23);foliage=(.07,.2,.23);leaflight=(.32,.55,.57);crystal=(.12,.75,1)
    if kind == 'verdant': rock=(.53,.60,.46);stone=(.62,.65,.45);turf=(.16,.32,.046);roof=(.10,.19,.12);foliage=(.067,.17,.014);leaflight=(.24,.34,.036)
    if kind in ['storm','hollow']: rock=(.20,.23,.29);stone=(.29,.34,.38);turf=(.10,.14,.10);roof=(.09,.055,.16);crystal=(.23,.009,.71)
    if kind=='crystal':roof=(.041,.015,.18)
    if kind == 'hollow': foliage=(.12,.22,.014);leaflight=(.30,.39,.025);glow=(.26,.75,.065)
    # Quiet cornices stay distinct from dressed architectural masonry. Each
    # PBR set preserves its old mean reflected colour under the live lighting.
    stone_texture='assets/textures/dressed-limestone-r11.png'
    # Base-colour factors are linear. This pale albedo is much brighter than
    # the old 50%-sRGB photograph, so using its factors washed out the engine.
    ROCK = material('01 | Fractured limestone',rock_factor(tuple(c*.32 for c in rock)),ROCK_ALBEDO,normal_texture=ROCK_NORMAL,roughness_texture=ROCK_ROUGHNESS)
    ROCK_DARK = material('02 | Deep strata',rock_factor(tuple(c*.23 for c in rock)),ROCK_ALBEDO,normal_texture=ROCK_NORMAL,roughness_texture=ROCK_ROUGHNESS)
    STONE = material('03 | Dressed masonry',masonry_factor(tuple(c*.40 for c in stone)),MASONRY_ALBEDO,normal_texture=MASONRY_NORMAL,roughness_texture=MASONRY_ROUGHNESS,normal_scale=MASONRY_NORMAL_SCALE)
    TRIM = material('04 | Worn cornice',tuple(c*.65 for c in stone),stone_texture)
    GOLD = material('05 | Brushed antique brass',(.58,.32,.085),metallic=.68,roughness=.37)
    WOOD = material('06 | Cedar and roots',(.59,.44,.30),'assets/textures/cedar.png')
    ROOF = material('07 | Enamel and shadow',roof,metallic=.22,roughness=.49)
    LEAF = material('08 | Deep foliage',foliage,roughness=1)
    LEAF_LIGHT = material('09 | Sunlit foliage',leaflight,roughness=1)
    GLOW = material('10 | Lantern glass',glow,roughness=.3,emission=2.6)
    CRYSTAL = material('11 | Aether mineral',crystal,metallic=.26,roughness=.13,emission=.04)
    if not LOD:
        shader=MATERIALS[CRYSTAL].node_tree.nodes.get('Principled BSDF')
        shader.inputs['Transmission Weight'].default_value=.18
        shader.inputs['IOR'].default_value=1.48
    SOIL = material('12 | Surface biome',turf,'assets/textures/grass.png' if kind in ['dawn','crystal','verdant','hollow'] else None)

def geom(mat, points, faces, bevel=0):
    verts, polys, uvs = BATCH.setdefault((mat, bevel), ([],[],[]))
    offset=len(verts); verts.extend([tuple(vec(p)) for p in points])
    for face in faces:
        polys.append(tuple(offset+i for i in face))
        pp=[Vector(points[i]) for i in face]
        normal=(pp[1]-pp[0]).cross(pp[2]-pp[0])
        dominant=max(range(3),key=lambda j:abs(normal[j]))
        axes=[j for j in range(3) if j!=dominant]
        # Rock bedding has one continuous height coordinate on both X/Z
        # cliff orientations. Signed U preserves the tangent-space handedness.
        if mat in [ROCK,ROCK_DARK]:
            sign=1 if normal[dominant]>=0 else -1
            if dominant==0:coords=[(-sign*p[2]/ROCK_TILE_METRES,p[1]/ROCK_TILE_METRES) for p in pp]
            elif dominant==2:coords=[(sign*p[0]/ROCK_TILE_METRES,p[1]/ROCK_TILE_METRES) for p in pp]
            else:coords=[(p[0]/ROCK_TILE_METRES,-sign*p[2]/ROCK_TILE_METRES) for p in pp]
            uvs.append(coords)
        elif mat==STONE and STONE!=TRIM:
            # Architectural courses run horizontally on either wall axis.
            # Vault silver strata aliases STONE/TRIM and keeps its own UVs.
            sign=1 if normal[dominant]>=0 else -1
            if dominant==0:coords=[(-sign*p[2]/MASONRY_TILE_METRES,p[1]/MASONRY_TILE_METRES) for p in pp]
            elif dominant==2:coords=[(sign*p[0]/MASONRY_TILE_METRES,p[1]/MASONRY_TILE_METRES) for p in pp]
            else:coords=[(p[0]/MASONRY_TILE_METRES,-sign*p[2]/MASONRY_TILE_METRES) for p in pp]
            uvs.append(coords)
        else:uvs.append([(p[axes[0]]*.11,p[axes[1]]*.11) for p in pp])

def collision(p,size):
    if LOD:return
    if min(size) > .05:
        COLLISIONS.append({'center':[round(v,5) for v in p], 'size':[round(v,5) for v in size]})

def box(p,size,mat,bevel=0,solid=False):
    if LOD and min(size)<.80:return
    x,y,z=p; a,b,c=[v/2 for v in size]
    if min(size)<=0: return
    points=[(x+sx*a,y+sy*b,z+sz*c) for sx,sy,sz in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
    # Tiny bevels cost five times the triangles and were forcing destructive
    # global decimation. Preserve the voxel corners on small relief / foliage.
    edge=bevel if not LOD and mat in [STONE,TRIM,WOOD] and min(size)>1 else 0
    geom(mat,points,[(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)],edge)
    if solid: collision(p,size)

def yaw_box(p,size,angle,mat):
    """Facade relief, oriented around Y, merged into the material batch."""
    if LOD and min(size)<.8:return
    x,y,z=p;w,h,d=[v/2 for v in size];c=math.cos(angle);s=math.sin(angle)
    points=[(x+xx*c+zz*s,y+yy,z-xx*s+zz*c) for xx,yy,zz in [(-w,-h,-d),(w,-h,-d),(w,h,-d),(-w,h,-d),(-w,-h,d),(w,-h,d),(w,h,d),(-w,h,d)]]
    # Inset glass and metal mullions sit against solid masonry: their hidden
    # five faces add no silhouette, unlike the carved surrounds.
    faces=[(4,5,6,7)] if size[2]<=.15 else [(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)]
    geom(mat,points,faces)

def facade_bay(x,y,z,w,h,angle=0,lit=True):
    if LOD:return
    c=math.cos(angle);s=math.sin(angle)
    def part(xx,yy,zz,size,mat):yaw_box((x+xx*c+zz*s,y+yy,z-xx*s+zz*c),size,angle,mat)
    # Recessed tall niche, deep sill and lintel, twin mullions and a triangular hood.
    part(0,h*.46,0,(w,h*.84,.12),ROOF)
    part(0,h*.43,.09,(w*.48,h*.59,.10),GLOW if lit else WOOD)
    for side in [-1,1]:
        part(side*w*.56,h*.46,.22,(w*.16,h*.96,.47),TRIM)
        part(side*w*.54,h*.05,.33,(w*.31,.45,.67),STONE)
    part(0,h*.04,.39,(w*1.45,.38,.82),TRIM)
    part(0,h*.9,.35,(w*1.40,.40,.73),TRIM)
    part(0,h*.43,.25,(.13,h*.68,.14),GOLD)
    part(0,h*.50,.25,(w*.70,.13,.14),GOLD)
    aa=(x-w*.79*c+.26*s,y+h*.95,z+w*.79*s+.26*c)
    bb=(x+.26*s,y+h*1.18,z+.26*c)
    cc=(x+w*.79*c+.26*s,y+h*.95,z-w*.79*s+.26*c)
    beam(aa,bb,.18,TRIM,4);beam(bb,cc,.18,TRIM,4)

def balustrade(x0,x1,y,z):
    if LOD:return
    box(((x0+x1)/2,y+1.25,z),(x1-x0,.28,.6),TRIM)
    box(((x0+x1)/2,y+.14,z),(x1-x0,.28,.8),STONE)
    for i in range(math.ceil((x1-x0)/1.6)+1):
        xx=x0+(x1-x0)*i/math.ceil((x1-x0)/1.6)
        box((xx,y+.69,z),(.38,1.0,.38),TRIM)

def cornice_blocks(x,y,z,w,d):
    if LOD:return
    for xx in range(math.ceil(-w/2),math.floor(w/2)+1,2):
        for side in [-1,1]:box((x+xx,y,z+side*d/2),(.6,.75,.75),TRIM)
    for zz in range(math.ceil(-d/2)+1,math.floor(d/2),2):
        for side in [-1,1]:box((x+side*w/2,y,z+zz),(.75,.75,.6),TRIM)

def beam(a,b,r,mat,n=8,r2=None):
    if LOD:
        if r<.42:return
        n=min(n,6)
    a,b=Vector(a),Vector(b); direction=(b-a).normalized()
    tangent=direction.cross(Vector((0,1,0)))
    if tangent.length<.01: tangent=direction.cross(Vector((1,0,0)))
    tangent.normalize(); bitangent=direction.cross(tangent)
    rr=[r,r if r2 is None else r2]
    points=[tuple(p+rr[k]*(math.cos(i*math.tau/n)*tangent+math.sin(i*math.tau/n)*bitangent)) for k,p in enumerate([a,b]) for i in range(n)]
    faces=[tuple(reversed(range(n))),tuple(range(n,n*2))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    geom(mat,points,faces)

def cyl(p,r,h,mat,n=12,solid=False):
    x,y,z=p; beam((x,y-h/2,z),(x,y+h/2,z),r,mat,n)
    if solid: collision(p,(r*1.414,h,r*1.414))

def torus(p,r,t,mat,axis='z',n=24,sides=6):
    if LOD:
        if t<.4:return
        n=min(n,12);sides=4
    x,y,z=p; pts=[]
    for i in range(n):
        a=i*math.tau/n
        for j in range(sides):
            b=j*math.tau/sides; rr=r+t*math.cos(b)
            pts.append((x+rr*math.cos(a),y+rr*math.sin(a),z+t*math.sin(b)) if axis=='z' else (x+rr*math.cos(a),y+t*math.sin(b),z+rr*math.sin(a)))
    faces=[(i*sides+j,((i+1)%n)*sides+j,((i+1)%n)*sides+(j+1)%sides,i*sides+(j+1)%sides) for i in range(n) for j in range(sides)]
    if axis=='y':faces=[tuple(reversed(face)) for face in faces]
    geom(mat,pts,faces)

def crystal(p,height,width=None):
    x,y,z=p; r=width or height*.19
    if LOD and height<4:return
    lean=math.sin(x*2.31+z*.54)*height*.11;lean_z=math.cos(z*1.49+x)*height*.09
    points=[(x+lean,y+height,z+lean_z),(x,y-height*.15,z)]
    for level,factor in [(.12,.72),(.69,1)]:
        for i in range(6):
            a=i*math.tau/6;points.append((x+math.cos(a)*r*factor+lean*level,y+height*level,z+math.sin(a)*r*factor+lean_z*level))
    faces=[(1,2+(i+1)%6,2+i) for i in range(6)]+[(2+i,2+(i+1)%6,8+(i+1)%6,8+i) for i in range(6)]+[(0,8+i,8+(i+1)%6) for i in range(6)]
    faces=[tuple(reversed(face)) for face in faces]
    geom(CRYSTAL,points,[face for i,face in enumerate(faces) if i%5!=2])
    geom(ROOF,points,[face for i,face in enumerate(faces) if i%5==2])
    # A tall mineral is physical scenery, including the leaning top. Inscribed
    # boxes follow the changing hexagonal section instead of enclosing empty
    # air around the prism in one large invisible collision envelope.
    if height>=5.5:
        for lo,hi,factor in [(0,.12,.16),(.12,.35,.36),(.35,.59,.42),(.59,.75,.38),(.75,.88,.15)]:
            t=(lo+hi)/2
            collision((x+lean*t,y+height*t,z+lean_z*t),(r*factor*2,height*(hi-lo),r*factor*2))

def crystal_cluster(x,y,z,h=15):
    if y==0:y=ground_level(x,z)
    crystal((x,y,z),h)
    for i in range(4):
        a=i*math.tau/4+.4;crystal((x+math.cos(a)*h*.24,y,z+math.sin(a)*h*.24),h*RNG.uniform(.3,.6))
    LANDMARKS['crystals'].append([x,y+h*.48,z])

def column(x,y,z,h=12,r=1.2):
    if y==0:y=ground_level(x,z)
    box((x,y+.4,z),(r*2.65,.8,r*2.65),STONE,.08,True)
    box((x,y+1,z),(r*2.1,.4,r*2.1),TRIM,.045)
    cyl((x,y+h/2+.2,z),r,h-1.6,STONE,12,True)
    for i in range(8):
        a=i*math.tau/8;beam((x+math.cos(a)*r,y+1.4,z+math.sin(a)*r),(x+math.cos(a)*r,y+h-1,z+math.sin(a)*r),.085,TRIM,4)
    box((x,y+h-.5,z),(r*2.4,.6,r*2.4),TRIM,.05)
    box((x,y+h,z),(r*2.7,.5,r*2.7),STONE,.05,True)

def arch(x,y,z,width=10,height=12,depth=2,mat=None,cap=True):
    if y==0:y=ground_level(x,z)
    mat=mat or STONE; r=width/2; spring=y+height-r
    for side in [-1,1]:
        box((x+side*(r+.8),y+(height-r)/2,z),(1.6,height-r,depth),mat,.06,True)
        box((x+side*(r+.8),spring,z),(2.3,.65,depth+.55),TRIM,.03)
    # Wedge stones preserve a real empty arch opening, including in the GLB.
    for i in range(12):
        aa=i*math.pi/12+.006; bb=(i+1)*math.pi/12-.006
        pts=[(x+math.cos(a)*rr,spring+math.sin(a)*rr,z+zz) for zz in [-depth/2,depth/2] for rr,a in [(r,bb),(r,aa),(r+1.55,aa),(r+1.55,bb)]]
        geom(TRIM if i%3==0 else mat,pts,[(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(0,1,5,4),(3,7,6,2)])
    if cap:box((x,y+height+1.5,z),(width+4.6,1.25,depth+.3),TRIM,.06,True)

def dome(x,y,z,r=8,h=7):
    segments=12 if LOD else 24; levels=3 if LOD else 8; pts=[]
    for j in range(levels):
        a=j*math.pi/2/levels
        for i in range(segments):
            b=i*math.tau/segments
            pts.append((x+math.cos(b)*r*math.cos(a),y+h*math.sin(a),z+math.sin(b)*r*math.cos(a)))
    pts.append((x,y+h,z));top=len(pts)-1
    faces=[(j*segments+i,j*segments+(i+1)%segments,(j+1)*segments+(i+1)%segments,(j+1)*segments+i) for j in range(levels-1) for i in range(segments)]
    faces += [((levels-1)*segments+i,(levels-1)*segments+(i+1)%segments,top) for i in range(segments)]
    faces=[tuple(reversed(face)) for face in faces]
    geom(ROOF,pts,faces)
    torus((x,y,z),r,.22,GOLD,'y',24)
    for i in range(8):
        a=i*math.tau/8
        for j in range(5):
            b=j*math.pi/10;c=(j+1)*math.pi/10
            beam((x+math.cos(a)*r*math.cos(b),y+h*math.sin(b),z+math.sin(a)*r*math.cos(b)),(x+math.cos(a)*r*math.cos(c),y+h*math.sin(c),z+math.sin(a)*r*math.cos(c)),.13,GOLD,5)
    cyl((x,y+h+1.1,z),.24,2.2,GOLD,8)
    crystal((x,y+h+2.1,z),1.35,.38)

def windows(x,y,z,width,h,depth):
    for side in [-1,1]:
        for offset in [-.28,0,.28]:
            xx=x+width*offset; zz=z+side*(depth/2+.025)
            box((xx,y+h*.56,zz),(width*.13,h*.42,.09),ROOF)
            box((xx,y+h*.56,zz+side*.055),(width*.069,h*.33,.03),GLOW)
            box((xx,y+h*.56,zz+side*.09),(.12,h*.35,.035),GOLD)
            box((xx,y+h*.58,zz+side*.09),(width*.075,.13,.035),GOLD)
            box((xx,y+h*.35,zz+side*.11),(width*.17,.18,.24),TRIM,.02)
            box((xx,y+h*.77,zz+side*.11),(width*.17,.18,.24),TRIM,.02)
            for dx in [-width*.075,width*.075]: box((xx+dx,y+h*.56,zz+side*.10),(.18,h*.44,.24),TRIM,.015)

def house(x,y,z,w=12,h=12,d=10,roof=True):
    if y==0:
        levels=[ground_level(x+dx,z+dz) for dx in [-w/2,0,w/2] for dz in [-d/2,0,d/2]]
        y=max(levels)
        if y>min(levels):box((x,(y+min(levels))/2,z),(w,y-min(levels),d),STONE,.08,True)
    box((x,y+.6,z),(w+1.2,1.2,d+1.2),STONE,.1,True)
    box((x,y+h/2,z),(w,h,d),STONE,.12,True)
    for yy in [y+1.6,y+h*.51,y+h-.3]: box((x,yy,z),(w+.65,.5,d+.65),TRIM,.05)
    for dx in [-w/2,w/2]:
        for dz in [-d/2,d/2]:box((x+dx,y+h/2,z+dz),(.75,h,.75),TRIM,.05)
    windows(x,y,z,w,h,d)
    if not LOD:
        # Return facades, quoins and offset stone courses keep every elevation
        # legible; these reliefs are shallow against an already solid building.
        for side in [-1,1]:
            for zz in [-d*.25,d*.25]:facade_bay(x+side*(w/2+.03),y+h*.20,z+zz,1.5,h*.42,side*math.pi/2)
        for row in range(1,max(2,int(h/2.4))):
            yy=y+row*2.4
            for side in [-1,1]:
                for xx in [-w*.44,w*.44]:box((x+xx,yy,z+side*(d/2+.09)),(1.0,.72,.22),TRIM)
                for zz in [-d*.4,d*.4]:box((x+side*(w/2+.09),yy,z+zz),(.22,.72,1.0),TRIM)
        cornice_blocks(x,y+h-.8,z,w+.3,d+.3)
    if roof:
        # Authored gabled roof with terraced stone eaves.
        pts=[(x-w*.61,y+h,z-d*.62),(x+w*.61,y+h,z-d*.62),(x+w*.61,y+h,z+d*.62),(x-w*.61,y+h,z+d*.62),(x,y+h+w*.37,z-d*.62),(x,y+h+w*.37,z+d*.62)]
        geom(ROOF,pts,[(0,4,1),(3,2,5),(0,3,5,4),(4,5,2,1),(0,1,2,3)])
        beam((x,y+h+w*.37,z-d*.65),(x,y+h+w*.37,z+d*.65),.22,GOLD,6)
        # Two stepped dormers interrupt the large roof planes.
        if w>=11:
            for side in [-1,1]:
                yy=y+h+w*.12
                box((x+side*w*.28,yy,z+d*.33),(2.8,2.6,2.1),STONE,0)
                box((x+side*w*.28,yy+.2,z+d*.33+1.08),(1.2,1.5,.12),ROOF)
                box((x+side*w*.28,yy+1.5,z+d*.33),(3.3,.5,2.6),ROOF)
    else: box((x,y+h+.4,z),(w+1.1,.8,d+1.1),TRIM,.09,True)

def tower(x,y,z,h=40,r=5,crown=True):
    if y==0:y=ground_level(x,z)
    box((x,y+1,z),(r*2.45,2,r*2.45),TRIM,.12,True)
    # Three retreating stone drums, stepped buttresses and overhanging cornices
    # replace the identical uninterrupted octagonal shafts.
    for tier,factor in enumerate([1.06,.93,.80]):
        lo=y+tier*h*.28;rr=r*factor
        cyl((x,lo+h*.14,z),rr,h*.28,STONE,8,True)
        cyl((x,lo+h*.28-.35,z),rr*1.22,.7,TRIM,8)
        if not LOD:
            cyl((x,lo+h*.28-.95,z),rr*1.12,.38,GOLD,8)
            for i in range(8):
                if h<25 and i%2:continue
                a=(i+.5)*math.tau/8;normal=rr*math.cos(math.pi/8)+.04
                facade_bay(x+math.sin(a)*normal,lo+h*.06,z+math.cos(a)*normal,max(.9,r*.35),h*.16,a)
            for i in range(8):
                a=i*math.tau/8
                beam((x+math.cos(a)*rr,lo+.7,z+math.sin(a)*rr),(x+math.cos(a)*rr,lo+h*.27,z+math.sin(a)*rr),r*.105,TRIM,4)
                beam((x+math.cos(a)*rr,lo+h*.21,z+math.sin(a)*rr),(x+math.cos(a)*rr*1.2,lo+h*.265,z+math.sin(a)*rr*1.2),r*.12,TRIM,4)
    if crown:
        for i in range(8):
            a=i*math.tau/8;column(x+math.cos(a)*r*.69,y+h*.84,z+math.sin(a)*r*.69,h*.16,.43)
        cyl((x,y+h,z),r*1.02,.8,TRIM,12)
        dome(x,y+h+.4,z,r*1.07,r*.98)
        cyl((x,y+h*.91,z),r*.30,h*.065,GLOW,10)
    else:
        for i in range(8):
            a=i*math.tau/8;box((x+math.cos(a)*r,y+h*.86,z+math.sin(a)*r),(1.6,3,1.6),TRIM,.07)

def stairs(x,y,z,w=16,rise=6,run=14,steps=20):
    steps=max(steps,math.ceil(rise/.12))
    if LOD:
        geom(STONE,[(x-w/2,y,z),(x+w/2,y,z),(x-w/2,y,z-run),(x+w/2,y,z-run),(x-w/2,y+rise,z-run),(x+w/2,y+rise,z-run)],[(0,2,3,1),(0,1,5,4),(2,4,5,3),(0,4,2),(1,3,5)])
        return
    for i in range(steps):
        h=rise*(i+1)/steps
        box((x,y+h/2,z-run*(i+.5)/steps),(w,h,run/steps+.015),STONE,0,True)
    for side in [-1,1]:
        for i in range(0,steps,4):
            h=rise*(i+1)/steps
            box((x+side*(w/2+.55),y+h/2+.7,z-run*(i+2)/steps),(1.1,h+1.4,run/steps*4),TRIM,.03,True)

def lantern(x,y,z,h=4):
    cyl((x,y+h/2,z),.14,h,GOLD,6)
    box((x,y+h,z),(.85,1.45,.85),GLOW,.05)
    box((x,y+h+.83,z),(1.2,.22,1.2),GOLD,.06)
    box((x,y+h-.82,z),(1.2,.2,1.2),GOLD,.03)
    for dx in [-.47,.47]:
        for dz in [-.47,.47]:beam((x+dx,y+h-.8,z+dz),(x+dx,y+h+.8,z+dz),.055,GOLD,4)

def canopy(x,y,z,w=12,d=9):
    for dx in [-w/2,w/2]:
        for dz in [-d/2,d/2]:beam((x+dx,0,z+dz),(x+dx,y+2,z+dz),.16,WOOD,7)
    for i in range(6):
        a=-w/2+i*w/6;b=-w/2+(i+1)*w/6
        yy=lambda v:y+.065*v*v
        geom(ROOF if i%2==0 else TRIM,[(x+a,yy(a),z-d/2),(x+b,yy(b),z-d/2),(x+b,yy(b),z+d/2),(x+a,yy(a),z+d/2)],[(0,1,2,3),(3,2,1,0)])
    box((x,1,z),(w*.7,2,2.5),WOOD,.08,True)
    for i in range(7): box((x+(i-3)*w*.09,2.2,z),(.85,.45,.8),GLOW if i%3==0 else SOIL,.025)

def tree(x,y,z,h=13,crown=6):
    if y==0 and TERRAIN and (round(x/8),round(z/8)) not in TERRAIN:return
    if y==0:y=ground_level(x,z)
    local=random.Random(round(x*419+z*173+h*31))
    bend=local.uniform(-1.5,1.5)
    elbow=(x+bend*.5,y+h*.39,z-.3);top=(x+bend,y+h*.79,z+.6)
    beam((x,y,z),elbow,h*.063,WOOD,7,r2=h*.039)
    beam(elbow,top,h*.039,WOOD,6,r2=.13)
    # Each branch carries two small, staggered voxel bouquets. Gaps between the
    # bouquets expose the tree's structure and eliminate square canopy slabs.
    for i in range(5):
        a=i*math.tau/5+.36;spread=crown*(.57 if i<4 else .16)
        xx=x+bend*.6+math.cos(a)*spread;zz=z+math.sin(a)*spread
        yy=y+h*(.74+.035*(i%3))
        branch=(xx,yy,zz)
        beam(elbow,branch,h*.029,WOOD,5,r2=.09)
        if LOD:
            if i<4:box((xx,yy+(2.2 if crown>9 else .7),zz),(crown*.88,6.1 if crown>9 else 2.2,crown*.86),LEAF_LIGHT if i==2 else LEAF)
            continue
        for j in range(3 if crown>9 else 2):
            center=(xx+math.cos(a+.9*j)*crown*.16,yy+.35+j*(crown*.23 if crown>9 else .8),zz+math.sin(a+.9*j)*crown*.17)
            cell=max(.72,crown*.205)
            for dx,dz in [(0,0),(-1,0),(1,0),(0,-1),(0,1)]:
                py=center[1]+(0.38 if dx==dz==0 else local.uniform(-.22,.20))
                box((center[0]+dx*cell*.77,py,center[2]+dz*cell*.77),(cell*local.uniform(.87,1.18),cell*.67,cell*local.uniform(.85,1.16)),LEAF_LIGHT if (i+j+dx+dz)%4==0 else LEAF)
        for j in range(3):
            box((xx+local.uniform(-1.2,1.2),yy-.55-j*.42,zz+local.uniform(-1.2,1.2)),(.58,.7,.62),LEAF)
    collision((x+bend*.35,y+h*.32,z+.3),(h*.09,h*.64,h*.09))

def ivy(x,y,z,length=12):
    if LOD:return
    px=x;pz=z
    for i in range(max(2,int(length/1.4))):
        px+=RNG.uniform(-.5,.5);pz+=RNG.uniform(-.3,.3)
        box((px,y-i*1.4,pz),(RNG.uniform(.5,1.3),1.75,RNG.uniform(.45,1.1)),LEAF_LIGHT if i%4==0 else LEAF,.015)

def terrain(kind):
    # Each biome has its own coastline and multiple walkable terrace heights.
    # The distant mesh is authored as a closed surface, never decimated boxes.
    global TERRAIN
    cave=kind in ['hollow','underforge']
    columns={}
    for ix in range(-9,10):
        for iz in range(-9,10):
            x=ix*8;z=iz*8
            radius=math.hypot(x,z)
            if not coastline(x,z):continue
            depth=16+71*max(math.exp(-((x+14)**2+(z+18)**2)/2200),.86*math.exp(-((x-31)**2+(z-8)**2)/1400))+4*math.sin(ix*14.7+iz*7.2+NAMES.index(kind))
            if cave and abs(x-5*math.sin(z*.045))<21+3*math.cos(z*.04) and z<52:
                # Deliberately absent rock beneath the traversable cavern void.
                continue
            columns[(ix,iz)]=depth
            level=ground_level(x,z)
            if not LOD:
                box((x,(level-depth)/2-.08,z),(8.04,depth+level,8.04),ROCK if (ix+iz)%3 else ROCK_DARK)
                box((x,level-.07,z),(8.05,.14,8.05),SOIL)
            collision((x,(level-depth)/2,z),(8.04,depth+level,8.04))
            if radius>58 and not LOD:
                for j in range(3):
                    xx=x+RNG.uniform(-3,3);zz=z+RNG.uniform(-3,3)
                    yy=-RNG.uniform(5,depth*.95)
                    box((xx,yy,zz),(RNG.uniform(2,5),RNG.uniform(3,9),RNG.uniform(2,5)),ROCK_DARK if j%2 else ROCK)
                if kind not in ['ember','frost','storm'] and RNG.random()<.55:ivy(x,level-2,z,RNG.uniform(7,24))
    # Distinct masonry and strata chips cover exposed cliff faces instead of a
    # smooth inverted cone. They never form an invisible collision envelope.
    for (ix,iz),depth in columns.items():
        x=ix*8;z=iz*8;level=ground_level(x,z)
        if LOD:
            geom(SOIL,[(x-4,level,z-4),(x-4,level,z+4),(x+4,level,z+4),(x+4,level,z-4)],[(0,1,2,3)])
            geom(ROCK,[(x-4,-depth,z-4),(x+4,-depth,z-4),(x+4,-depth,z+4),(x-4,-depth,z+4)],[(0,1,2,3)])
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            neighbor=columns.get((ix+dx,iz+dz))
            adjacent_level=ground_level((ix+dx)*8,(iz+dz)*8)
            if LOD:
                spans=[(-depth,level)] if neighbor is None else [(-depth,min(level,-neighbor)),(max(-depth,adjacent_level),level)]
                for lo,hi in spans:
                    if hi-lo<.001:continue
                    if dx:
                        pts=[(x+dx*4,lo,z-4),(x+dx*4,hi,z-4),(x+dx*4,hi,z+4),(x+dx*4,lo,z+4)]
                        face=(0,1,2,3) if dx==1 else (3,2,1,0)
                    else:
                        pts=[(x-4,lo,z+dz*4),(x+4,lo,z+dz*4),(x+4,hi,z+dz*4),(x-4,hi,z+dz*4)]
                        face=(0,1,2,3) if dz==1 else (3,2,1,0)
                    geom(ROCK,pts,[face])
                continue
            for j in range(math.floor(-depth/3.8),math.ceil(level/3.8)):
                yy=j*3.8+1.9
                if neighbor is not None and -neighbor<=yy<=adjacent_level:continue
                # Every block lies on a truly exposed face, rather than inside
                # the main island. Staggered joints produce readable strata.
                for segment in [-1,1]:
                    along=segment*2.0+RNG.uniform(-.75,.75)
                    xx=ix*8+dx*4.1+dz*along;zz=iz*8+dz*4.1+dx*along
                    size=(RNG.uniform(.8,3.0),RNG.uniform(2.1,4.9),RNG.uniform(2.3,5.0)) if dx else (RNG.uniform(2.3,5.0),RNG.uniform(2.1,4.9),RNG.uniform(.8,3.0))
                    box((xx,yy,zz),size,ROCK if (j+segment)%3 else ROCK_DARK)
            if neighbor is None and kind in ['dawn','crystal','verdant','hollow']:
                # Moss on ledges makes cliffs belong to their surface biome.
                box((ix*8+dx*4.1,level-.7,iz*8+dz*4.1),(1.1,1.3,6) if dx else (6,1.3,1.1),LEAF)
    TERRAIN=columns
    # Offset roots and overhangs break the regular plateau silhouette. Every
    # structural spur is made of the same visible boxes as its collision shape.
    for i in range(7):
        a=i*math.tau/7+NAMES.index(kind)*.33
        px=math.cos(a)*56;pz=math.sin(a)*55
        if cave and abs(px)<30:continue
        top=ground_level(px,pz)-8
        for j in range(6):
            width=18-j*2.65;xx=px+math.sin(a+1)*j*1.15;zz=pz+math.cos(a-1)*j*.75
            box((xx,top-j*12-6,zz),(width,12.4,width*.86),ROCK if j%2 else ROCK_DARK,0,True)
    # Paved approach always reaches the actual stone surface at z=70.
    box((0,0,80),(14,1,18),WOOD,.05,True)
    for z in range(72,89,2):
        box((0,.535,z),(13.8,.10,1.82),WOOD,.025)
    for side in [-1,1]:
        for z in [72,78,84,88]:
            cyl((side*6.5,-1,z),.36,5,WOOD,8)
            if z != 88:beam((side*6.5,1.5,z),(side*6.5,1.5,z+5),.065,GOLD,6)
        lantern(side*6,0,74,3)
    for iz in range(7):
        for ix in [-1,0,1]:
            box((ix*3.9,.07,68-iz*4),(3.77,.16,3.77),STONE,.028)
    for x in [-10,10]:
        for z in [45,61]:lantern(x,0,z)

def surface_dressing(kind):
    # Low relief terraces, broken coping, individual growth and flower clumps
    # make large surfaces readable at walking distance as well as from the sky.
    if LOD or kind in ['ember','storm','underforge','hollow']: return
    for i in range(50):
        a=RNG.uniform(0,math.tau);r=RNG.uniform(54,68)
        x=math.cos(a)*r;z=math.sin(a)*r
        if abs(x)<13 and z>40 or not coastline(x,z):continue
        level=ground_level(x,z)
        box((x,level+.12,z),(RNG.uniform(2,5),.24,RNG.uniform(2,5)),SOIL)
        if kind=='frost':
            box((x,level+.5,z),(RNG.uniform(1,3),1,RNG.uniform(1,3)),SOIL,.05)
            continue
        for j in range(5):
            xx=x+RNG.uniform(-1.5,1.5);zz=z+RNG.uniform(-1.5,1.5)
            box((xx,level+RNG.uniform(.3,.65),zz),(.38,RNG.uniform(.4,1.2),.35),LEAF_LIGHT if j%2 else LEAF)
            if j%3==0:box((xx,level+.95,zz),(.32,.22,.32),CRYSTAL if kind=='crystal' else GOLD)
    for x in [-60,60]:
        for z in range(-24,34,8):
            if not coastline(x,z):continue
            level=ground_level(x,z)
            box((x,level+.65,z),(3,1.3,6.8),STONE,.05,True)
            box((x,level+1.4,z),(3.4,.35,7.1),TRIM,.035)
            if kind not in ['nomad','frost']:ivy(x+(1.7 if x>0 else -1.7),level+1.5,z,5)

def pool(x,y,z,w=24,d=18,height=85):
    if y==0:y=ground_level(x,z)
    box((x,y-.9,z),(w,1,d),STONE,.1,True)
    for dx in [-w/2,w/2]:box((x+dx,y-.1,z),(1,1,d),TRIM,.06,True)
    for dz in [-d/2,d/2]:box((x,y-.1,z+dz),(w,1,1),TRIM,.06,True)
    LANDMARKS['pools'].append({'position':[x,y+.15,z],'size':[w-.9,.22,d-.9]})
    shore=max(iz*8+4 for ix,iz in TERRAIN if ix==round(x/8))+.6
    front=z+d/2
    if shore>front:
        LANDMARKS['pools'].append({'position':[x,y+.15,(front+shore)/2],'size':[w*.48,.22,shore-front]})
    LANDMARKS['waterfalls'].append({'position':[x,y+.3,shore],'width':w*.48,'height':height})

def quay(x,z,w=20):
    box((x,.4,z),(w,.8,13),STONE,.06,True)
    for xx in [x-w/2+1,x+w/2-1]:
        lantern(xx,.8,z,3.5)
        for zz in [z-5,z+5]:box((xx,1.8,zz),(1.4,2,1.4),TRIM,.06,True)

def plaza(x,y,z,r=15):
    cyl((x,y-.18,z),r,.36,STONE,32,True)
    for rr in [r*.4,r*.8,r]:torus((x,y+.04,z),rr,.07,GOLD,'y',32,4)
    for i in range(8):
        a=i*math.tau/8;beam((x+math.cos(a)*r*.25,y+.06,z+math.sin(a)*r*.25),(x+math.cos(a)*r*.94,y+.06,z+math.sin(a)*r*.94),.07,GOLD,4)

def banner(x,y,z,h=10):
    beam((x,y,z),(x,y+h+1,z),.13,GOLD,8)
    beam((x-.4,y+h,z),(x+4,y+h,z),.1,GOLD,6)
    geom(ROOF,[(x,y+h,z),(x+3.7,y+h,z+.4),(x+3.5,y+h-4,z+.3),(x+1.9,y+h-5,z+.14),(x,y+h-4,z)],[(0,1,2,3,4),(4,3,2,1,0)])
    torus((x+1.8,y+h-2,z+.32),.8,.08,GOLD,'z',12,4)
    beam((x+1.8,y+h-.9,z+.33),(x+1.8,y+h-3.1,z+.33),.06,GOLD,4)

def side_stairs(x,y,z,side,rise,run,w=11):
    count=math.ceil(rise/.12)
    if LOD:
        points=[(x,y,z-w/2),(x,y,z+w/2),(x+side*run,y,z-w/2),(x+side*run,y,z+w/2),(x+side*run,y+rise,z-w/2),(x+side*run,y+rise,z+w/2)]
        faces=[(0,1,3,2),(0,4,5,1),(2,3,5,4),(0,2,4),(1,5,3)]
        if side<0:faces=[tuple(reversed(f)) for f in faces]
        geom(STONE,points,faces);return
    for i in range(count):
        h=rise*(i+1)/count
        box((x+side*run*(i+.5)/count,y+h/2,z),(run/count+.015,h,w),STONE,0,True)
    for dz in [-w/2,w/2]:
        beam((x,y+.9,z+dz),(x+side*run,y+rise+.9,z+dz),.13,GOLD,6)

def terrace_routes():
    if CURRENT in ['hollow','underforge']:
        stairs(-28,0,27,6,14,54,60)
        stairs(28,7,27,6,16,54,60)
        for x,y in [(-28,14),(28,23)]:box((x,y-.25,-32),(7,.5,10),STONE,.05,True)
        cavern_route_finish()
    else:
        left=ground_level(-64,24);right=ground_level(64,-4)
        side_stairs(-23,0,24,-1,left,30)
        side_stairs(31,0,-4,1,right,25)
        for x,z,y in [(-57,24,left),(60,-4,right)]:
            box((x,y-.2,z),(8,.4,13),STONE,.06,True)
            lantern(x,y,z+5,3)

def bridge_x(x0,x1,y,z,w=5):
    box(((x0+x1)/2,y-.5,z),(x1-x0,1,w),STONE,.06,True)
    for zz in [z-w/2,z+w/2]:
        box(((x0+x1)/2,y+1,zz),(x1-x0,.35,.35),GOLD,.015)
        for i in range(int((x1-x0)/3)+1):
            xx=x0+i*3
            box((xx,y+.7,zz),(.45,1.4,.45),TRIM,.03)
    for xx in [x0+2,x1-2]:column(xx,ground_level(xx,z),z,max(3,y-ground_level(xx,z)-.5),.7)

def relief_stone(origin,along,normal,outline,depth,mat,seed):
    """Shallow, individually dressed stone: chipped arris and uneven face.

    Outline coordinates are metres along a wall and metres above its base.
    No collider or shared RNG is touched by the entire cavern finish pass.
    """
    local=random.Random(seed);cx=sum(p[0] for p in outline)/len(outline);cy=sum(p[1] for p in outline)/len(outline)
    def point(u,v,d):return (origin[0]+along[0]*u+normal[0]*d,origin[1]+v,origin[2]+along[1]*u+normal[1]*d)
    back=[point(u,v,.012) for u,v in outline]
    front=[point(u+(cx-u)*.065,v+(cy-v)*.085,depth+local.uniform(-.028,.028)) for u,v in outline]
    n=len(outline);points=back+front
    faces=[tuple(range(n,n*2))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    # Outward-facing relief on either bank, with a real beveled perimeter.
    if along[1]*normal[0]-along[0]*normal[1]>0:faces=[tuple(reversed(f)) for f in faces]
    geom(mat,points,faces)

def masonry_skin(origin,along,normal,length,start_height,end_height,seed):
    if LOD:return
    local=random.Random(seed);row=0;bottom=.10
    while bottom<max(start_height,end_height)-.08:
        height=local.uniform(1.05,1.60);top=bottom+height;u=-local.uniform(.2,2.4)
        while u<length-.08:
            width=local.uniform(2.15,4.15);left=max(.045,u);right=min(length-.045,u+width-.09)
            if right-left>.25:
                h0=start_height+(end_height-start_height)*left/length
                h1=start_height+(end_height-start_height)*right/length
                poly=[(left,bottom),(right,bottom),(right,top),(left,top)]
                # Clip each bed to the exact inclined wall: nothing crosses a tread.
                clipped=[]
                for aa,bb in zip(poly,poly[1:]+poly[:1]):
                    fa=aa[1]-(start_height+(end_height-start_height)*aa[0]/length)
                    fb=bb[1]-(start_height+(end_height-start_height)*bb[0]/length)
                    if fa<=0:clipped.append(aa)
                    if (fa<0<fb) or (fb<0<fa):
                        t=fa/(fa-fb);clipped.append((aa[0]+t*(bb[0]-aa[0]),aa[1]+t*(bb[1]-aa[1])))
                if len(clipped)>=3:
                    shade=TRIM if (row+int(u*3))%13==0 else ROCK_DARK if (row+int(u))%7==0 else STONE
                    relief_stone(origin,along,normal,clipped,local.uniform(.12,.25),shade,seed+row*973+int(u*31))
            u+=width
        bottom=top+.075;row+=1

def retaining_finish(origin,along,normal,length,rise,seed,parapet=0):
    masonry_skin(origin,along,normal,length,parapet,rise+parapet,seed)
    def p(u,v,d):return (origin[0]+along[0]*u+normal[0]*d,origin[1]+v,origin[2]+along[1]*u+normal[1]*d)
    # Continuous worn string course and legible tapering wall piers.
    beam(p(.2,parapet-.10,.20),p(length-.2,rise+parapet-.10,.20),.24,TRIM,4)
    if LOD:return
    for j in range(1,max(2,int(length/8))):
        u=j*length/max(2,int(length/8));h=rise*u/length+parapet-.35
        if h<1:continue
        outline=[(u-.64,.06),(u+.64,.06),(u+.43,h),(u-.43,h)]
        relief_stone(origin,along,normal,outline,.58,TRIM,seed+j)
        # Two shallow offsets read as a carved capital, never as an obstacle.
        relief_stone(origin,along,normal,[(u-.70,h-.25),(u+.70,h-.25),(u+.70,h+.12),(u-.70,h+.12)],.67,TRIM,seed+j+97)
        if CURRENT=='underforge':
            beam(p(u,h*.30,.64),p(u,h*.82,.64),.075,GOLD,5)
        elif j%2==0:
            root=[p(u-.18,h+.02,.72),p(u+.20,h*.70,.74),p(u-.30,h*.43,.76),p(u-.07,max(.2,h*.17),.78)]
            for a,b in zip(root,root[1:]):beam(a,b,.095,WOOD,5,r2=.055)
            for k in [1,2]:
                q=root[k]
                for side in [-1,1]:
                    tip=(q[0]+along[0]*side*.50,q[1]-.22,q[2]+along[1]*side*.50)
                    geom(LEAF, [q,tip,(tip[0],tip[1]-.55,tip[2]),(q[0],q[1]-.20,q[2])],[(0,1,2,3),(3,2,1,0)])

def cavern_route_finish():
    # Stair cladding lives exclusively outside the existing stair side walls.
    for index,(x,y,rise) in enumerate([(-28,0,14),(28,7,16)]):
        for side in [-1,1]:
            retaining_finish((x+side*4.115,y,27),(0,-1),(side,0),54,rise,7103+index*911+side,.67)
    for index,(x,y,side,rise) in enumerate([(-28,14,-1,21),(28,23,1,12)]):
        for outward in [-1,1]:
            retaining_finish((x,y,-29+outward*1.515),(side,0),(0,outward),18,rise,8701+index*739+outward,-.20)
        if not LOD:
            # A recessed service niche on the solid triangular stair cheek:
            # a green votive in the ruins, a caged maintenance light in the forge.
            zz=-27.08;xx=x+side*12;spring=y+4.8;radius=1.28
            shape=[(xx-radius,y+1.1),(xx+radius,y+1.1),(xx+radius,spring)]
            shape += [(xx+radius*math.cos(i*math.pi/10),spring+radius*math.sin(i*math.pi/10)) for i in range(1,11)]
            geom(ROOF,[(px,py,zz) for px,py in shape],[tuple(range(len(shape)))])
            for i in range(9):
                aa=i*math.pi/9+.025;bb=(i+1)*math.pi/9-.025
                ring=[(xx+radius*math.cos(bb),spring+radius*math.sin(bb)),(xx+radius*math.cos(aa),spring+radius*math.sin(aa)),(xx+(radius+.32)*math.cos(aa),spring+(radius+.32)*math.sin(aa)),(xx+(radius+.32)*math.cos(bb),spring+(radius+.32)*math.sin(bb))]
                relief_stone((0,0,zz),(1,0),(0,1),ring,.16,TRIM,18800+index*19+i)
            for dx in [-radius-.16,radius+.16]:box((xx+dx,y+2.95,zz+.10),(.30,3.7,.25),TRIM)
            box((xx,y+1.03,zz+.23),(3.5,.33,.52),TRIM)
            box((xx,y+3.6,zz+.035),(.43,1.2,.09),GLOW)
            for dx in [-.63,0,.63]:beam((xx+dx,y+1.32,zz+.22),(xx+dx,y+4.70,zz+.22),.045,GOLD if CURRENT=='underforge' else WOOD,4)

def cavern_bridge_finish():
    # Segmental stone arches enrich the 100 m bridge silhouette. All new
    # geometry stays above y=29.5, well clear of the ship channel y=9..21.
    for side in [-1,1]:
        z=-33+side*2.53
        # Forward end bays adjoin the upper stairs. Set their arches behind
        # the original bridge face and stop its relief before either approach.
        extent=41 if side==1 else 50
        masonry_skin((-extent,33.85,z),(1,0),(0,side),extent*2,.98,.98,14003+side)
        for bay in range(5):
            cx=-40+bay*20
            arch_z=-31.25 if side==1 and bay in [0,4] else z
            for i in range(8):
                aa=i*math.pi/8+.008;bb=(i+1)*math.pi/8-.008
                outline=[(cx+9.9*math.cos(bb),30.15+3.15*math.sin(bb)),(cx+9.9*math.cos(aa),30.15+3.15*math.sin(aa)),(cx+10.1*math.cos(aa),30.78+3.13*math.sin(aa)),(cx+10.1*math.cos(bb),30.78+3.13*math.sin(bb))]
                relief_stone((0,0,arch_z),(1,0),(0,side),outline,.31,TRIM if i in [3,4] else STONE,15000+bay*19+i)
                # Recessed spandrels connect the curved voussoirs to the deck.
                a,b=outline[2:]
                geom(STONE,[(a[0],a[1],arch_z),(b[0],b[1],arch_z),(b[0],33.99,arch_z),(a[0],33.99,arch_z)],[(0,1,2,3),(3,2,1,0)])
        for x in [-50,-30,-10,10,30,50]:
            relief_stone((x,29.55,z),(1,0),(0,side),[(-.46,0),(.46,0),(.83,4.35),(-.83,4.35)],.49,TRIM,16000+x)
            if not LOD:
                relief_stone((x,33.35,z),(1,0),(0,side),[(-1.0,0),(1.0,0),(1.0,.55),(-1.0,.55)],.60,TRIM,17000+x)
                if CURRENT=='underforge':
                    for yy in [30.55,32.10]:cyl((x,yy,z+side*.55),.18,.25,GOLD,6)
        if not LOD:
            beam((-extent,34.90,z+side*.12),(extent,34.90,z+side*.12),.14,TRIM,4)

def cavern_roof_finish(x,y,z,kind,seed):
    if LOD:return
    local=random.Random(seed)
    # Split bedding planes carved into broad, irregular flakes; these sit
    # immediately below the closed original roof and never alter its envelope.
    outline=[(-3.85,-2.8),(-1.0,-3.82),(3.6,-3.0),(3.9,.4),(2.2,3.75),(-2.9,3.50),(-3.90,.6)]
    drop=local.uniform(.32,.78)
    pts=[(x+u,y-.025,z+v) for u,v in outline]+[(x+u*.91,y-drop+local.uniform(-.08,.08),z+v*.93) for u,v in outline]
    faces=[tuple(range(7,14))]+[(i,7+i,7+(i+1)%7,(i+1)%7) for i in range(7)]
    geom(ROCK if seed%3 else ROCK_DARK,pts,faces)
    if seed%5==0:
        material=LEAF if kind=='hollow' else CRYSTAL
        vein=[(x-2.3,y-drop-.06,z-.6),(x-.7,y-drop-.09,z+.15),(x+.3,y-drop-.05,z-.4),(x+2.2,y-drop-.04,z+.3)]
        for a,b in zip(vein,vein[1:]):beam(a,b,.07,material,4)
    if kind=='hollow' and abs(x)>30 and seed%4==0:
        for j in range(3):
            q=(x-1+j*.7,y-drop,z+1.2);mid=(q[0]+.45,q[1]-2.3-j*.3,q[2]+.3);tip=(q[0]-.1,q[1]-4.3-j*.7,q[2]+.45)
            beam(q,mid,.14,WOOD,5,r2=.085);beam(mid,tip,.085,WOOD,5,r2=.018)

def buttress(x,y,z,h,w=2):
    for i in range(4):
        height=h*(1-i*.17)
        box((x,y+height/2,z+i*.85),(w,height,1.2),TRIM,.06,True)

def district_details(x,y,z):
    if LOD:return
    # At walking scale: a stone bench, stacked shipping crates, a timber cart,
    # planter, grain sacks and a lantern. They are tied to the terrace height.
    box((x,y+.65,z),(4,.4,1.2),WOOD,.06)
    for dx in [-1.4,1.4]:box((x+dx,y+.3,z),(.45,.6,1),STONE,.04)
    for i in range(4):
        px=x+3.5+(i%2)*1.1;yy=y+.55+(i//2)*1.1
        box((px,yy,z+1),(1.05,1.05,1.05),WOOD,.05)
        for dz in [-.45,.45]:box((px,yy,z+1+dz),(1.1,.1,.07),GOLD)
    box((x-3,y+.55,z),(2.5,1.1,2.5),STONE,.05,True)
    for i in range(7):box((x-3+RNG.uniform(-.8,.8),y+1.5,z+RNG.uniform(-.8,.8)),(.5,1.3,.5),LEAF_LIGHT)
    lantern(x+5,y,z-2,3.2)

def palace_rotunda():
    # The hero building is a layered rotunda with inhabited wings and an open
    # lantern gallery; the original blank 30 x 21 m cube is removed.
    cyl((0,25.7,-31),16.2,1.4,TRIM,12,True)
    cyl((0,34,-31),13.6,16.6,STONE,12,True)
    cyl((0,42.1,-31),15.1,1.0,TRIM,12)
    cyl((0,43.0,-31),14.6,.7,ROOF,12)
    for i in range(12):
        a=(i+.5)*math.tau/12
        facade_bay(math.sin(a)*13.2,28,-31+math.cos(a)*13.2,2.4,10.7,a)
        a=i*math.tau/12
        if not LOD:
            column(math.sin(a)*13.9,25.7,-31+math.cos(a)*13.9,15.5,.56)
        column(math.sin(a)*12.6,43.3,-31+math.cos(a)*12.6,5.3,.56)
    cyl((0,49,-31),15.4,.8,TRIM,12)
    dome(0,49.4,-31,15.8,13.0)
    # Two recessed wings, a front balcony, stair turrets and stepped rooflines
    # create the asymmetrical castle silhouette from the Dawn board.
    for side in [-1,1]:
        house(side*18,25,-31,9,13 if side<0 else 16,19,True)
        tower(side*16.8,25,-14.8,17 if side<0 else 21,2.35)
        if not LOD:
            balustrade(side*9-5,side*9+5,25.6,-8.3)
            for z in [-39,-28,-18]:ivy(side*22.1,26,z,6+int(abs(z)%5))
    cornice_blocks(0,22.6,-27,43,37)
    for x in [-16,16]:
        box((x,26.5,-10),(6,1.2,2.7),STONE,0,True)
        box((x,27.2,-10),(5.2,.6,2.1),LEAF)
    # Dark wall joints remain coarse, shallow and architectural.
    if not LOD:
        for row in range(3):
            for side in [-1,1]:
                for j in range(4):
                    x=side*(13.7+j*4.4+(row%2)*.35)
                    box((x,1.15+row*2.0,-9.96),(4.22,1.82,.10),TRIM if (j+row)%4==0 else STONE)

def dawn():
    plaza(0,.2,24,17)
    stairs(0,0,5,22,7,15,24)
    box((0,3.5,-33),(60,7,46),STONE,.12,True)
    for x in [-24,24]:
        for z in [-47,-18]:tower(x,7,z,42 if z==-18 else 58,4.4)
    for x in [-14,0,14]:arch(x,7,-13,10,14,3)
    box((0,24,-27),(43,3,37),TRIM,.13,True)
    palace_rotunda()
    tower(0,50,-42,32,3.2)
    for x,z,h in [(-46,8,16),(-47,-17,19),(-32,32,12),(40,24,13),(54,25,18),(42,-31,15),(-21,-59,14),(20,-61,15)]:house(x,0,z,12,h,12)
    for x in [-23,23]:
        for z in [18,38]:tree(x,0,z,12,5)
    for x,z in [(-56,35),(56,30),(-40,-49),(40,-49)]:tree(x,0,z,13,5)
    for x in [-16,16]:banner(x,.2,43,9)
    pool(-38,0,50,20,17,83)
    pool(41,0,49,17,17,72)
    for x in [-60,60]:tower(x,0,-11,23,3)
    # The west borough is lower and packed around a courtyard; the eastern
    # observatory quarter climbs two terraces and reaches the palace by bridges.
    for x,z,w,h,d in [(-58,37,9,12,10),(-61,6,9,17,11),(-56,-30,11,21,10),(-39,-48,12,13,12),(56,-33,10,17,11),(45,-51,12,24,11),(27,47,9,10,8),(-39,49,10,8,8)]:
        house(x,0,z,w,h,d)
        district_details(x-3,ground_level(x,z),z+d/2+2)
    for x in [-46,-33]:arch(x,7,-3,9,10,2)
    box((-39.5,19,-3),(28,1.5,8),STONE,.08,True)
    for z in [-45,-20,20]:buttress(35,16,z,15,2.2)
    for x in [-14,14]:buttress(x,7,-14,17,1.6)
    bridge_x(25,45,31,-19,5)
    bridge_x(-48,-24,24,-33,5)
    for x,z,h in [(-62,21,10),(-41,8,8),(48,-12,11),(59,-48,10),(-23,44,8),(22,35,7)]:tree(x,0,z,h,4)
    for x,z in [(-37,36),(45,10),(10,32),(-11,17)]:district_details(x,ground_level(x,z),z)
    LANDMARKS['portal']=[0,8,-9]

def crystal_island():
    pool(0,.1,30,35,22,90)
    for x in [-25,25]:
        for z in [-44,-23,-2,19]:column(x,0,z,20,1.5)
        box((x,21,-12),(4,2,74),TRIM,.08,True)
    for x in [-30,30]:tower(x,0,-45,51,4.7)
    stairs(0,0,-1,22,8,17,24)
    box((0,4,-33.5),(29,8,31),STONE,.1,True)
    crystal_cluster(0,8,-32,47)
    for x,z,h in [(-47,14,23),(48,11,29),(-39,-26,18),(36,-22,20),(-19,58,10),(24,55,13)]:crystal_cluster(x,0,z,h)
    for x,z in [(-48,40),(45,41),(-55,-32),(50,-37)]:tree(x,0,z,9,4)
    for x in [-43,43]:arch(x,0,8,13,17,2.4)
    tower(8,22,-47,66,6.8)
    # Tall offset pinnacles and two inhabited galleries turn the rear tower
    # into a sanctuary silhouette, while the harvestable mineral IDs stay put.
    for side in [-1,1]:
        xx=8+side*11.5;zz=-48 if side<0 else -55;yy=22
        column(xx,yy,zz,28 if side<0 else 37,1.5)
        crown_y=yy+(28 if side<0 else 37)
        cyl((xx,crown_y,zz),3.0,.85,TRIM,8)
        beam((xx,crown_y+.4,zz),(xx,crown_y+15,zz),3.3,STONE,6,r2=.15)
        crystal((xx,crown_y+12,zz),6,1.0)
        if not LOD:
            beam((xx,yy+20,zz),(8+side*5,yy+41,-47),.72,TRIM,6)
    for yy,rr in [(43,10.2),(67,8.2)]:
        cyl((8,yy,-47),rr,1.0,TRIM,12)
        if not LOD:
            for i in range(12):
                a=i*math.tau/12
                beam((8+math.sin(a)*(rr-2),yy-4,-47+math.cos(a)*(rr-2)),(8+math.sin(a)*rr,yy-.6,-47+math.cos(a)*rr),.4,TRIM,5)
    for side in [-1,1]:
        # Pool galleries have actual empty arches and carved overhangs.
        for zz in [1,19]:
            xx=side*25
            if not LOD:
                for row in range(3):
                    cyl((xx,4+row*6,zz),1.72,.35,TRIM,8)
        if not LOD:
            balustrade(side*25-2,side*25+2,22,23.8)
            for zz in [-36,-18,0,18]:ivy(side*26.8,21,zz,5+abs(zz)%5)
    for x,z,h in [(-47,-29,29),(47,-19,34),(-31,-54,21)]:
        y=ground_level(x,z)
        for j in range(5):box((x+j*.5,y+(j+.5)*h/5,z),(12-j*1.6,h/5,12-j*1.6),ROCK if j%2 else ROCK_DARK,.09,True)
        crystal_cluster(x+2,y+h,z,16)
    pool(-47,10,42,17,18,91)
    pool(45,10,-20,16,16,107)
    bridge_x(-38,-18,24,-38,5)
    for x,z in [(-27,13),(30,35),(-51,-8)]:
        column(x,ground_level(x,z),z,11,1.1)
        crystal_cluster(x,ground_level(x,z)+12,z,8)
    LANDMARKS['portal']=[0,10,-15]

def nomad():
    plaza(0,.2,25,17)
    for x,z in [(-28,38),(29,36),(-27,19),(29,17)]:canopy(x,5,z,17,10)
    for x,z,h,w in [(-48,-5,22,16),(48,9,25,17),(-39,-37,18,14),(32,-40,23,15),(-61,13,13,12),(51,28,15,11)]:
        house(x,0,z,w,h,14,False)
        for dx in [-w/2,w/2]:box((x+dx,h+1,z),(1.4,2,15),TRIM,.05)
    stairs(0,0,3,18,8,16,24)
    box((0,4,-32),(35,8,38),STONE,.1,True)
    for x in [-12,0,12]:arch(x,8,-15,8,12,3)
    house(0,22,-29,30,13,27,False)
    dome(0,35,-29,15,9)
    for x in [-24,24]:tower(x,0,-30,61 if x<0 else 48,3.6)
    for x,z in [(-51,47),(51,46),(-57,-25),(53,-28)]:tree(x,0,z,14,5)
    for x in [-17,17]:banner(x,0,51,11)
    LANDMARKS['portal']=[0,9,-11]

def gear(x,y,z,r=6):
    global BATCH
    saved=None
    if CURRENT=='underforge':
        saved=BATCH;BATCH={}
    torus((x,y,z),r,.7,GOLD,'z',28,6)
    torus((x,y,z),r*.35,.5,GOLD,'z',16,6)
    for i in range(12):
        a=i*math.tau/12
        beam((x+math.cos(a)*r*.30,y+math.sin(a)*r*.30,z),(x+math.cos(a)*r*.92,y+math.sin(a)*r*.92,z),.30,GOLD,6)
        box((x+math.cos(a)*(r+.55),y+math.sin(a)*(r+.55),z),(1.5,1.5,1.8),GOLD,.09)
    if saved is not None:
        names={(-42,39):'UnderforgeGearWestForge',(40,-37):'UnderforgeGearEastForge',(43,37):'UnderforgeGearEastWorkshop',(-47,0):'UnderforgeGearWestDrive',(47,0):'UnderforgeGearEastDrive'}
        ANIMATED.append({'name':names[x,z],'pivot':(x,y,z),'batches':BATCH})
        BATCH=saved

def chimney(x,y,z,h=48,r=3):
    if y==0:y=ground_level(x,z)
    cyl((x,y+h/2,z),r,h,ROCK_DARK,12,True)
    for yy in range(4,int(h),7):cyl((x,y+yy,z),r*1.11,.7,GOLD,12)
    cyl((x,y+h,z),r*1.3,1.4,TRIM,12)
    cyl((x,y+h+.8,z),r*.75,.05,GLOW,12)

def forge_building(x,y,z,w=24,h=31,d=19):
    if y==0:y=ground_level(x,z)
    box((x,y+h/2,z),(w,h,d),STONE,.12,True)
    for yy in [y+.6,y+h*.68,y+h-.4]:box((x,yy,z),(w+1.3,1.1,d+1.3),TRIM,.07)
    for dx in [-w*.48,w*.48]:
        for dz in [-d*.48,d*.48]:box((x+dx,y+h/2,z+dz),(1.25,h,1.25),ROCK_DARK,.08)
    arch(x,y,z+d/2+.6,w*.45,h*.63,2.8)
    box((x,y+h*.32,z+d/2+.065),(w*.40,h*.50,.11),ROOF)
    # Recessed fire grate: many narrow hot slots and a heavy dark gate. It reads
    # as machinery at deck height, rather than one flat luminous rectangle.
    for i in range(7):
        xx=x+(i-3)*w*.047
        box((xx,y+h*.31,z+d/2+.13),(w*.035,h*.44,.1),GLOW)
    for yy in [y+h*.15,y+h*.31,y+h*.48]:box((x,yy,z+d/2+.29),(w*.44,.65,.38),ROOF,.04)
    for dx in [-w*.24,w*.24]:
        box((x+dx,y+h*.32,z+d/2+.30),(1.1,h*.55,.55),GOLD,.06)
        for j in range(7):box((x+dx,y+2+j*h*.076,z+d/2+.61),(.30,.30,.2),ROCK_DARK,.035)
    gear(x,y+h*.77,z+d/2+1,w*.19)
    for dx in [-w*.39,w*.39]:chimney(x+dx,y+h,z-d*.24,h*.7,1.4)
    for dx in [-w*.46,w*.46]:
        beam((x+dx,y+2,z+d/2+1.3),(x+dx,y+h*.85,z+d/2+1.3),.6,GOLD,8)

def ember():
    plaza(0,.1,32,16)
    stairs(0,0,12,21,6,15,22)
    box((0,3,-26),(49,6,46),ROCK_DARK,.1,True)
    forge_building(0,6,-25,36,39,28)
    for x,z,h in [(-44,-22,61),(43,-28,72),(-29,-47,45),(27,-53,50)]:chimney(x,0,z,h,3.4)
    for x,z in [(-48,43),(43,19)]:forge_building(x,0,z,20,23,18)
    for x in [-61,61]:tower(x,0,1,29,3.3,False)
    for x in [-34,34]:
        pool(x,.1,52,15,12,93)
        beam((x,3,44),(x,3,21),1.1,GOLD,10)
        for z in [27,35,42]:torus((x,3,z),1.25,.18,GOLD,'z',12)
    for x in [-17,17]:banner(x,0,49,9)
    LANDMARKS['portal']=[0,7,-7]

def frost():
    stairs(0,0,24,24,9,25,32)
    box((0,4.5,-27.75),(58,9,53.5),STONE,.1,True)
    # Cathedral has actual pointed aisles and flying buttress silhouettes.
    house(0,9,-26,30,28,36)
    arch(0,9,-6,15,23,3)
    for x in [-24,24]:
        for z in [-8,-44]:
            tower(x,9,z,48 if z==-8 else 58,4.1,False)
            beam((x,50 if z==-8 else 58,z),(x,73 if z==-8 else 85,z),6.6,ROOF,8,r2=.3)
            box((x,48 if z==-8 else 56,z),(13,1.2,13),SOIL,.04)
    for x in [-1,1]:
        for z in [-41,-28,-15]:
            beam((x*29,11,z),(x*14,34,z),.95,TRIM,8)
            column(x*31,0,z,15,1.3)
    for x,z in [(-54,40),(45,22),(-39,-44),(38,-48)]:tower(x,0,z,24,3.5)
    for x,z in [(-44,46),(46,43),(-54,-15),(54,-12)]:crystal_cluster(x,0,z,11)
    for i in range(56):
        a=i*math.tau/56;r=69;h=RNG.uniform(8,27)
        beam((math.cos(a)*r,-2,math.sin(a)*r),(math.cos(a)*r,-h,math.sin(a)*r),RNG.uniform(.65,1.8),CRYSTAL,5,r2=.04)
    for x in [-16,16]:banner(x,0,37,10)
    LANDMARKS['portal']=[0,10,-5]

def verdant():
    stairs(0,0,20,19,7,19,26)
    box((0,3.5,-15.5),(48,7,33),STONE,.13,True)
    for x in [-16,0,16]:arch(x,7,-1,10,16,3)
    box((0,26,-9),(52,3,25),STONE,.14,True)
    for x in [-22,22]:tower(x,0,-28,33,4,False)
    # The hero tree is a bent network of tapering roots, trunk and branches.
    spine=[(7,0,-27),(4,20,-27),(-2,42,-24),(3,63,-30),(-6,84,-29)]
    for i in range(4):beam(spine[i],spine[i+1],9-i*1.6,WOOD,10,r2=7-i*1.4)
    for i in range(12):
        a=i*math.tau/12
        end=(math.cos(a)*RNG.uniform(27,41),-RNG.uniform(0,13),-25+math.sin(a)*RNG.uniform(22,35))
        middle=(math.cos(a)*13,10,-25+math.sin(a)*13)
        beam((4,20,-27),middle,3.1,WOOD,7,r2=2.3);beam(middle,end,2.3,WOOD,7,r2=.7)
    for i in range(10):
        a=i*math.tau/10;r=RNG.uniform(17,31);yy=RNG.uniform(55,83)
        end=(math.cos(a)*r,yy,-26+math.sin(a)*r)
        beam((0,42,-25),end,3.5,WOOD,8,r2=.8)
        tree(end[0],end[1]-6,end[2],11,13)
    for x,z in [(-48,18),(46,20),(-35,45),(34,45),(-49,-23),(47,-19)]:tree(x,0,z,20,8)
    for i in range(28):
        a=i*math.tau/28;ivy(math.cos(a)*24,27,-8+math.sin(a)*14,RNG.uniform(12,23))
    pool(-38,0,51,17,17,91)
    pool(40,0,49,17,19,98)
    LANDMARKS['portal']=[0,8,4]

def chain(a,b,link=1.5):
    a,b=Vector(a),Vector(b);distance=(b-a).length;n=max(2,int(distance/(link*1.5)))
    for i in range(n):
        p=a.lerp(b,i/(n-1));torus(tuple(p),link,.18,GOLD if CURRENT=='underforge' else ROOF,'z' if i%2==0 else 'y',10,4)

def storm():
    stairs(0,0,21,18,6,17,22)
    box((0,3,-16.5),(45,6,41),ROCK_DARK,.1,True)
    for x,z,h in [(-20,-1,46),(21,-8,56),(-18,-35,62),(19,-37,41)]:tower(x,6,z,h,4.7,False)
    for x in [-45,45]:tower(x,0,12,30,4,False)
    arch(0,6,3,13,22,3)
    crystal_cluster(0,10,-20,35)
    torus((0,34,-20),15,1.1,GOLD,'z',32,7)
    for x in [-20,20]:chain((x,49,-10),(0,31,-20),.85)
    for x,z in [(-43,-23),(45,-21),(-26,49),(29,47)]:crystal_cluster(x,0,z,12)
    for x in [-46,46]:
        for z in [-5,5,15,25]:box((x,2,z),(7,4,6),STONE,.11,True)
    for x in [-14,14]:banner(x,0,39,12)
    LANDMARKS['portal']=[0,7,4]

def ceiling(kind):
    # The local cavern is a continuous eroded rock cap, with a carved underside
    # and unequal upper ridges. Emit only its exposed skin, not a pile of cubes
    # with hidden faces. Physical boxes exactly fill these visible columns.
    cells={}
    for ix in range(-8,9):
        for iz in range(-8,9):
            x=ix*8;z=iz*8
            if math.hypot(x,z)>70+7*math.sin(math.atan2(z,x)*3):continue
            if x<-40 and z>40:continue
            if abs(x)<12 and z>48:continue
            lower=51-10*math.sin(z*.034)+7*math.cos(x*.053)+(9 if x>25 else 0)
            ridge=max(31*math.exp(-((x+31)/24)**2-((z+21)/49)**2),
                      42*math.exp(-((x-35)/23)**2-((z+9)/37)**2),
                      24*math.exp(-((x+2)/39)**2-((z+52)/17)**2))
            upper=max(lower+10,64+ridge+4*math.sin(x*.12+z*.071))
            upper=math.ceil(upper/4)*4
            cells[ix,iz]=(lower,upper)
    for (ix,iz),(lower,upper) in cells.items():
        x=ix*8;z=iz*8
        collision((x,(lower+upper)/2,z),(8,upper-lower,8))
        geom(ROCK,[(x-4,upper,z-4),(x-4,upper,z+4),(x+4,upper,z+4),(x+4,upper,z-4)],[(0,1,2,3)])
        geom(ROCK_DARK if (ix+iz)%4 else ROCK,[(x-4,lower,z-4),(x+4,lower,z-4),(x+4,lower,z+4),(x-4,lower,z+4)],[(0,1,2,3)])
        if (ix*7+iz*11)%3==0:cavern_roof_finish(x,lower,z,kind,23003+ix*971+iz*193)
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            other=cells.get((ix+dx,iz+dz))
            spans=[(lower,upper)] if other is None else [(lower,min(upper,other[0])),(max(lower,other[1]),upper)]
            for bottom,top in spans:
                if top-bottom<.01:continue
                levels=[bottom,top] if LOD else [bottom]+[float(y) for y in range(40,113,6) if bottom<y<top]+[top]
                for lo,hi in zip(levels,levels[1:]):
                    if dx:
                        pts=[(x+dx*4,lo,z-4),(x+dx*4,hi,z-4),(x+dx*4,hi,z+4),(x+dx*4,lo,z+4)]
                        if dx<0:pts.reverse()
                    else:
                        pts=[(x-4,lo,z+dz*4),(x+4,lo,z+dz*4),(x+4,hi,z+dz*4),(x-4,hi,z+dz*4)]
                        if dz<0:pts.reverse()
                    layer=int((lo+hi)*.5/6)
                    geom(ROCK_DARK if layer%5==0 else STONE if layer%7==0 else ROCK,pts,[(0,1,2,3)])
            # Broad fractured ledges break long exposed faces; keep their
            # horizontal reach inside the original roof footprint.
            if other is None and not LOD and (ix*7+iz*11)%3==0:
                for row in [0,1]:
                    yy=lower+5+row*8
                    if yy+2>upper:continue
                    box((x-dx*.3,yy,z-dz*.3),(7.8 if dx else 5.9,2.8,5.9 if dx else 7.8),ROCK_DARK,0,True)
        if abs(x)>25 and (ix*7+iz*11)%4==0:
            drop=10+9*math.sin(ix+iz*2)**2
            for j in range(3):
                width=6-j*1.65
                box((x+j*.5,lower-drop*(j+.5)/3,z-j*.4),(width,drop/3+.1,width*.85),ROCK,.03,True)
            if kind=='hollow':ivy(x+1,lower-drop,z,7)
    for index,(x,z,width) in enumerate([(-49,-43,13),(-60,5,10),(-48,42,8),(47,-36,9),(60,13,15),(42,52,7)]):
        top=51-10*math.sin(z*.034)+7*math.cos(x*.053)+(9 if x>25 else 0)
        bottom=ground_level(x,z)
        count=5 if LOD else 9
        for j in range(count):
            t=(j+.5)/count
            xx=x+math.sin(index+t*3)*2.5;zz=z+math.cos(index+t*2)*1.5
            yy=bottom+(top-bottom)*t
            w=width*(.89+.2*math.sin(j*1.7+index)**2+.2*abs(t-.5))
            box((xx,yy,zz),(w,(top-bottom)/count+.2,width*(.74+.12*math.cos(j+index)**2)),ROCK_DARK if j%4==0 else ROCK,0,True)
            if not LOD and j%2==0:
                # Split buttress flutes remain against the old side pillars;
                # no added mass approaches the stairs or the sailing channel.
                side=-1 if x<0 else 1
                box((xx+side*w*.43,yy+.2,zz-width*.25),(width*.39,(top-bottom)/count+1.1,width*.47),STONE if j%3==0 else ROCK_DARK,0,True)
        if kind=='hollow':ivy(x-3,top-2,z+width*.42,top-bottom-3)
    # A narrow pedestrian bridge runs below sailing height, leaving ship clearance.
    box((0,0,19),(45,1,9),WOOD,.05,True)
    for x in range(-20,21,4):box((x,.56,19),(3.8,.12,9),WOOD,.03)
    for z in [14.5,23.5]:
        for x in range(-20,21,8):beam((x,.4,z),(x,2.3,z),.11,GOLD,6)
        beam((-21,2.1,z),(21,2.1,z),.10,GOLD,6)
    bridge_x(-50,50,35,-33,5)
    cavern_bridge_finish()
    box((-50,34,-28),(8,2,15),STONE,.08,True)
    box((50,34,-28),(8,2,15),STONE,.08,True)
    side_stairs(-28,14,-29,-1,21,18,3)
    side_stairs(28,23,-29,1,12,18,3)

def hollow():
    ceiling('hollow')
    for x,z,h in [(-39,-35,24),(39,-35,30),(-38,7,21),(40,6,20),(-40,38,13),(42,38,14)]:
        tower(x,0,z,h,3.5,False)
        crystal_cluster(x+(-5 if x<0 else 5),0,z+7,6)
    for x in [-36,36]:
        for z in [-30,5,34]:
            top=51-10*math.sin(z*.034)+7*math.cos(x*.053)+(9 if x>25 else 0)
            hanging=27 if x<0 else 36
            chain((x,top,z),(x,hanging,z),.65)
            lantern(x,hanging-2,z,2)
    for x in [-51,51]:tree(x,0,-7,31,7)
    for x in [-28,28]:
        for z in [-43,-5,38]:crystal_cluster(x,0,z,8)
    arch(-42,35,-32,12,16,3)
    for x,z in [(-45,-9),(42,15),(35,-49)]:
        y=ground_level(x,z)
        box((x,y+1,z),(12,2,10),STONE,.09,True)
        for j in range(4):column(x-4+j*3,y+2,z,7+(j%2)*3,.65)
    tree(39,25,-36,25,8)
    crystal_cluster(-56,27,16,17)
    LANDMARKS['waterfalls'] += [{'position':[-33,43,-42],'width':4,'height':90},{'position':[38,45,-8],'width':5,'height':85}]
    LANDMARKS['portal']=[-38,2,8]

def underforge():
    ceiling('underforge')
    for x,z,w,h in [(-42,29,23,25),(40,-47,25,27)]:
        forge_building(x,0,z,w,h,18)
        dome(x,ground_level(x,z)+h+1,z,9,7)
    tower(-42,14,-41,40,5)
    house(43,0,28,18,19,15)
    gear(43,ground_level(43,28)+12,37,6)
    house(-38,38,3,10,10,9,False)
    for x in [-43,-33]:chain((x,61,3),(x,48,3),.6)
    for x,z in [(-40,-10),(45,-1)]:district_details(x,ground_level(x,z),z)
    for x in [-29,29]:
        beam((x,39,-48),(x,39,49),1,GOLD,10)
        for z in range(-42,45,12):torus((x,39,z),1.3,.22,GOLD,'z',12,5)
        for z in [-32,24]:chain((x,45,z),(x,29,z),.65)
    for x in [-47,47]:
        gear(x,15,0,7)
        crystal_cluster(x,0,-5,8)
        pool(x,0,50,16,12,100)
    for x in [-33,33]:
        for z in [-48,-5,38]:lantern(x,0,z,6)
    LANDMARKS['waterfalls'] += [{'position':[-27,45,-34],'width':3,'height':85},{'position':[33,43,-14],'width':4,'height':90}]
    LANDMARKS['portal']=[-39,2,18]

def portal_gate():
    # The same arrival landmark provides a legible traversal contract in every
    # biome. It is a true open structure, not a billboard or collision cylinder.
    arch(0,0,54,18,24,2.5,cap=False)
    for side in [-1,1]:
        column(side*12,0,54,17,1.05)
        crystal_cluster(side*12,18,54,5.6)
        box((side*12,8.5,55.4),(.25,12,.12),GOLD)
    for i in range(16):
        a=i*math.pi/16;b=(i+1)*math.pi/16
        beam((math.cos(a)*9.25,15+math.sin(a)*9.25,55.36),(math.cos(b)*9.25,15+math.sin(b)*9.25,55.36),.13,GOLD,5)
    for side in [-1,1]:beam((side*9.25,0,55.36),(side*9.25,15,55.36),.13,GOLD,5)
    LANDMARKS['portal']=[0,12,54]

def write_glb(path,objects,kind,lod=False):
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:obj.select_set(True)
    bpy.context.view_layer.objects.active=objects[0]
    bpy.ops.export_scene.gltf(filepath=str(path),export_format='GLB',use_selection=True,export_yup=True,export_apply=True,export_texcoords=True,export_normals=True,export_tangents=True,export_materials='EXPORT',export_extras=True)
    raw=path.read_bytes();length=struct.unpack_from('<I',raw,12)[0]
    doc=json.loads(raw[20:20+length]);old_binary=raw[28+length:]
    # Blender's glTF exporter writes the image but drops the legacy MixRGB
    # MULTIPLY constant. Keep the authored linear stone factors explicit in
    # the delivered PBR material, then verify them by reimporting this file.
    for exported_material in doc.get('materials',[]):
        name=exported_material.get('name','')
        for key in [ROCK,ROCK_DARK,STONE,TRIM]+([LEAF,LEAF_LIGHT] if LEAF.startswith('08 |') else [])+([CRYSTAL] if globals().get('CRYSTAL') in COLORS else []):
            if name==key or name.startswith(key+'.'):
                exported_material.setdefault('pbrMetallicRoughness',{})['baseColorFactor']=[*COLORS[key],1]
                break
    for m in doc.get('materials',[]):
        if m.get('name','').startswith('11 | Aether mineral'):
            m['emissiveFactor']=list(COLORS[CRYSTAL])
            m.setdefault('extensions',{})['KHR_materials_emissive_strength']={'emissiveStrength':CRYSTAL_EMISSION_STRENGTH}
            if 'KHR_materials_emissive_strength' not in doc.setdefault('extensionsUsed',[]):doc['extensionsUsed'].append('KHR_materials_emissive_strength')
            # This export is reviewed using exactly the same opaque PBR paths
            # as Bevy: albedo, roughness/metallic, tangent normals and emission.
            m.get('extensions',{}).pop('KHR_materials_transmission',None)
            m['alphaMode']='OPAQUE'
        if m.get('name','').startswith('08 |'):
            m['alphaMode']='MASK';m['alphaCutoff']=.33;m['doubleSided']=True
        elif m.get('name','').startswith('09 |'):
            m['alphaMode']='OPAQUE';m.pop('alphaCutoff',None);m['doubleSided']=True
    # Identical texture images are stored once for all eighteen GLBs. Relative
    # dependencies work with Bevy's AssetServer and with portable distribution.
    image_views=set()
    for image in doc.get('images',[]):
        if 'bufferView' not in image:continue
        index=image.pop('bufferView');image_views.add(index);view=doc['bufferViews'][index]
        pixels=old_binary[view.get('byteOffset',0):view.get('byteOffset',0)+view['byteLength']]
        suffix='.png' if image.get('mimeType')=='image/png' else '.jpg'
        filename=hashlib.sha256(pixels).hexdigest()+suffix
        directory=OUT/'textures';directory.mkdir(exist_ok=True)
        if not (directory/filename).exists():(directory/filename).write_bytes(pixels)
        image['uri']='textures/'+filename
    remap={};views=[];new_binary=bytearray()
    for old_index,view in enumerate(doc.get('bufferViews',[])):
        if old_index in image_views:continue
        new_binary+=b'\0'*((-len(new_binary))%4)
        remap[old_index]=len(views);updated=dict(view);updated['byteOffset']=len(new_binary)
        start=view.get('byteOffset',0);new_binary+=old_binary[start:start+view['byteLength']];views.append(updated)
    for accessor in doc.get('accessors',[]):
        if 'bufferView' in accessor:accessor['bufferView']=remap[accessor['bufferView']]
        if 'sparse' in accessor:
            for key in ['indices','values']:
                accessor['sparse'][key]['bufferView']=remap[accessor['sparse'][key]['bufferView']]
    doc['bufferViews']=views;doc['buffers'][0]['byteLength']=len(new_binary)
    new_binary+=b'\0'*((-len(new_binary))%4)
    binary=struct.pack('<II',len(new_binary),0x004E4942)+new_binary
    doc['asset'].setdefault('extras',{}).update({'aether_world_generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'biome':kind,'level_of_detail':1 if lod else 0,'coordinate_system':'Y-up metres','dock':[0,3,88]})
    encoded=json.dumps(doc,separators=(',',':')).encode();encoded+=b' '*((-len(encoded))%4)
    path.write_bytes(struct.pack('<III',0x46546C67,2,20+len(encoded)+len(binary))+struct.pack('<II',len(encoded),0x4E4F534A)+encoded+binary)
    tris=sum(doc['accessors'][prim['indices']]['count']//3 for mesh in doc.get('meshes',[]) for prim in mesh['primitives'])
    return {'file':path.name,'triangles':tris,'materials':len(doc.get('materials',[])),'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

def make_objects():
    objects=[]
    for (mat,bevel),(verts,faces,uvs) in BATCH.items():
        mesh=bpy.data.meshes.new(mat);mesh.from_pydata(verts,[],faces);mesh.update()
        uv=mesh.uv_layers.new(name='Metric UV')
        for poly,coords in zip(mesh.polygons,uvs):
            for li,co in zip(poly.loop_indices,coords):uv.data[li].uv=co
        obj=bpy.data.objects.new(mat,mesh);bpy.context.collection.objects.link(obj)
        mesh.materials.append(MATERIALS[mat]);bpy.context.view_layer.objects.active=obj
        if mat in [LEAF,LEAF_LIGHT] and mat.startswith(('08 |','09 |')):
            for poly in mesh.polygons:poly.use_smooth=True
        if bevel:
            mod=obj.modifiers.new('Small authored edge chips','BEVEL');mod.width=bevel;mod.segments=1
            bpy.ops.object.modifier_apply(modifier=mod.name)
            mod=obj.modifiers.new('Weighted stone corner normals','WEIGHTED_NORMAL');mod.keep_sharp=True
            bpy.ops.object.modifier_apply(modifier=mod.name)
        objects.append(obj)
    # Consolidate by material: at most twelve draw meshes per detail island.
    merged=[]
    groups={mat:[obj for obj in objects if obj.data.materials[0]==MATERIALS[mat]] for mat in MATERIALS}
    for mat,group in groups.items():
        if not group:continue
        bpy.ops.object.select_all(action='DESELECT')
        for obj in group:obj.select_set(True)
        bpy.context.view_layer.objects.active=group[0]
        if len(group)>1:bpy.ops.object.join()
        group[0].name=mat
        group[0].data.validate(clean_customdata=False)
        triangulate=group[0].modifiers.new('Explicit export triangles','TRIANGULATE')
        bpy.ops.object.modifier_apply(modifier=triangulate.name)
        group[0].data.validate(clean_customdata=False)
        merged.append(group[0])
    return merged

def make_animated_objects():
    global BATCH
    saved=BATCH;result=[]
    for moving in ANIMATED:
        BATCH=moving['batches'];objects=make_objects()
        assert len(objects)==1,'Each existing gear uses one shared brass material'
        obj=objects[0];obj.name=moving['name'];obj.data.name=moving['name']+'Geometry'
        pivot=vec(moving['pivot'])
        for vertex in obj.data.vertices:vertex.co-=pivot
        obj.location=pivot
        obj['aether_animation_axis']='local Z'
        obj['aether_animation_role']='visual gear'
        result.append(obj)
    BATCH=saved
    return result

def render(kind,objects,interior=False):
    scene=bpy.context.scene
    scene.render.engine='CYCLES';scene.cycles.samples=32
    scene.cycles.use_denoising=True
    # CPU fallback is reliable in background mode; use available CUDA devices.
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='CUDA';prefs.get_devices()
        found=False
        for device in prefs.devices:
            device.use=device.type=='CUDA';found=found or device.use
        if found:scene.cycles.device='GPU'
    except Exception:pass
    scene.render.resolution_x=1672;scene.render.resolution_y=941;scene.render.resolution_percentage=100
    scene.world.use_nodes=True
    scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.27,.40,.63,1)
    scene.world.node_tree.nodes['Background'].inputs[1].default_value=.45
    scene.view_settings.view_transform='AgX'
    data=bpy.data.lights.new('Late afternoon soft key','SUN');data.energy=3.0;data.angle=math.radians(9)
    sun=bpy.data.objects.new('Late afternoon soft key',data);bpy.context.collection.objects.link(sun);sun.rotation_euler=(math.radians(25),math.radians(-28),math.radians(-30))
    data=bpy.data.lights.new('Sky fill','AREA');data.energy=110000;data.shape='DISK';data.size=170
    light=bpy.data.objects.new('Sky fill',data);bpy.context.collection.objects.link(light);light.location=vec((-80,120,140));light.rotation_euler=(vec((0,0,0))-light.location).to_track_quat('-Z','Y').to_euler()
    camera_data=bpy.data.cameras.new('Asset review camera');camera=bpy.data.objects.new('Asset review camera',camera_data);bpy.context.collection.objects.link(camera)
    if interior:
        camera.location=vec((8,22,145));target=vec((0,24,-8));camera_data.type='PERSP';camera_data.lens=32
    else:
        camera.location=vec((175,130,245));target=vec((0,-2,-5));camera_data.type='ORTHO';camera_data.ortho_scale=335
    camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler();scene.camera=camera
    if not interior:
        rotation=camera.rotation_euler.to_quaternion();inverse=rotation.inverted()
        points=[inverse@(obj.matrix_world@vertex.co-target) for obj in objects for vertex in obj.data.vertices]
        minimum=[min(p[j] for p in points) for j in range(2)];maximum=[max(p[j] for p in points) for j in range(2)]
        offset=rotation@Vector(((minimum[0]+maximum[0])/2,(minimum[1]+maximum[1])/2,0))
        camera.location+=offset
        camera_data.ortho_scale=max((maximum[0]-minimum[0])*1.1,(maximum[1]-minimum[1])*1672/941*1.1)
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(PREVIEW/('world-art-'+kind+('-interior' if interior else '')+'.png'))
    bpy.ops.render.render(write_still=True)
    for obj in [sun,light,camera]:bpy.data.objects.remove(obj,do_unlink=True)

def validate_cavern(kind):
    if kind not in ['hollow','underforge']:return
    for c in COLLISIONS:
        p=c['center'];s=c['size']
        if abs(p[0]) < 8+s[0]/2 and abs(p[1]-15)<6+s[1]/2 and abs(p[2])<52+s[2]/2:
            raise AssertionError(f'{kind}: solid in ship passage: {c}')

def render_export(path,kind,interior=False):
    # Review the delivered geometry/materials, including external texture URIs,
    # instead of a richer Blender source that the glTF exporter may simplify.
    if '--no-render' in sys.argv:return
    clear()
    bpy.ops.import_scene.gltf(filepath=str(path))
    imported=[obj for obj in bpy.context.scene.objects if obj.type=='MESH']
    assert imported,f'No meshes reimported from {path}'
    render(kind,imported,interior)

def compose(kind):
    palette(kind);terrain(kind)
    {'dawn':dawn,'crystal':crystal_island,'nomad':nomad,'ember':ember,'frost':frost,'verdant':verdant,'storm':storm,'hollow':hollow,'underforge':underforge}[kind]()
    surface_dressing(kind);terrace_routes();portal_gate()

def main():
    global CURRENT,BATCH,COLLISIONS,LANDMARKS,RNG,LOD
    selected=[arg for arg in sys.argv[sys.argv.index('--')+1:] if not arg.startswith('--')] if '--' in sys.argv else NAMES
    if not selected:selected=NAMES
    all_collisions=json.loads((OUT/'collisions.json').read_text()) if (OUT/'collisions.json').exists() else {}
    all_landmarks=json.loads((OUT/'landmarks.json').read_text()) if (OUT/'landmarks.json').exists() else {}
    report=json.loads((OUT/'manifest.json').read_text()) if (OUT/'manifest.json').exists() else {}
    for kind in selected:
        assert kind in NAMES
        print('BUILDING',kind,flush=True)
        clear();CURRENT=kind;LOD=False;RNG=random.Random(74019+NAMES.index(kind)*271)
        BATCH={};COLLISIONS=[];LANDMARKS={'waterfalls':[],'pools':[],'crystals':[],'portal':None}
        compose(kind)
        validate_cavern(kind)
        objects=make_objects()+make_animated_objects()
        full_total=sum(len(obj.data.polygons) for obj in objects)
        # A violated budget must be fixed in the authoring functions. Collapsing
        # disjoint stone blocks globally creates holes and triangular cliff chips.
        assert full_total<=200000,(kind,full_total)
        bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/(kind+'.blend')))
        hi=write_glb(OUT/(kind+'.glb'),objects,kind)
        render_export(OUT/(kind+'.glb'),kind)
        if kind in ['hollow','underforge']:render_export(OUT/(kind+'.glb'),kind,True)
        hi_collisions=COLLISIONS;hi_landmarks=LANDMARKS
        # Re-author a closed low resolution representation. Never collapse the
        # disconnected stone blocks: that creates the triangle holes seen in
        # the first engine review. Every large volume survives at a distance.
        clear();LOD=True;BATCH={};COLLISIONS=[];LANDMARKS={'waterfalls':[],'pools':[],'crystals':[],'portal':None}
        RNG=random.Random(74019+NAMES.index(kind)*271)
        compose(kind);objects=make_objects()+make_animated_objects()
        low=write_glb(OUT/(kind+'-lod.glb'),[obj for obj in objects if len(obj.data.polygons)>0],kind,True)
        render_export(OUT/(kind+'-lod.glb'),kind+'-lod')
        assert hi['materials']<=12 and low['materials']<=12
        assert hi['triangles']<=200000,hi
        assert low['triangles']<=12000,low
        all_collisions[kind]=hi_collisions;all_landmarks[kind]=hi_landmarks
        report[kind]={'high':hi,'lod':low,'colliders':len(hi_collisions),'dock':[0,3,88],'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
        for path,value in [('collisions.json',all_collisions),('landmarks.json',all_landmarks),('manifest.json',report)]:
            (OUT/path).write_text(json.dumps(value,indent=2),encoding='utf8')
        print('COMPLETED',kind,hi,low,'colliders',len(hi_collisions),flush=True)
    print('World kit complete',flush=True)

# Foliage authoring implementation, embedded for complete source hashing.
"""R44 botanical foliage; embedded in the shared world generator namespace.

Botanical axes carry curved alpha-masked shoots. A smaller, irregular closed
core supplies canopy coverage where subpixel leaves would disappear at distance.
No camera-facing billboard, global RNG consumption, or new collider is used.
"""
_legacy_tree=tree
_legacy_palette=palette
_legacy_box=box
_legacy_geom=geom
FOLIAGE_IMAGE='assets/textures/evergreen-oak-sprig-r44.png'
CANOPY_IMAGE='assets/textures/evergreen-oak-canopy-r44.png'
FOLIAGE_METRICS=[]

def palette(kind):
    _legacy_palette(kind)
    # Reuse exactly the two established foliage slots. Other nodes/factors stay
    # untouched, including the r41 rock and r42 stone/cornice PBR materials.
    for name,factor,rough in [(LEAF,(.45,.9,.32),.83),(LEAF_LIGHT,(.70,.91,.55),.87)]:
        m=MATERIALS[name];nt=m.node_tree;bsdf=nt.nodes.get('Principled BSDF')
        tex=nt.nodes.new('ShaderNodeTexImage')
        tex.image=bpy.data.images.load(str(ROOT/(FOLIAGE_IMAGE if name==LEAF else CANOPY_IMAGE)),check_existing=True)
        mix=nt.nodes.new('ShaderNodeMixRGB');mix.blend_type='MULTIPLY'
        mix.inputs[0].default_value=1;mix.inputs[2].default_value=(*factor,1)
        nt.links.new(tex.outputs['Color'],mix.inputs[1]);nt.links.new(mix.outputs[0],bsdf.inputs['Base Color'])
        # The final GLB uses MASK at .33. Clipped silhouettes need no blend sort.
        if name==LEAF:
            threshold=nt.nodes.new('ShaderNodeMath');threshold.operation='GREATER_THAN';threshold.inputs[1].default_value=.33
            nt.links.new(tex.outputs['Alpha'],threshold.inputs[0]);nt.links.new(threshold.outputs[0],bsdf.inputs['Alpha'])
        bsdf.inputs['Roughness'].default_value=rough
        m.diffuse_color=(*factor,1);COLORS[name]=factor

def _foliage_geom(mat,points,faces,uvs):
    verts,polys,coords=BATCH.setdefault((mat,0),([],[],[]))
    offset=len(verts);verts.extend([tuple(vec(p)) for p in points])
    polys.extend([tuple(offset+i for i in f) for f in faces]);coords.extend(uvs)

def _textured_core(points,faces):
    uvs=[]
    for f in faces:
        p=[Vector(points[i]) for i in f];normal=(p[1]-p[0]).cross(p[2]-p[0])
        dominant=max(range(3),key=lambda i:abs(normal[i]));axes=[i for i in range(3) if i!=dominant]
        uvs.append([(v[axes[0]]/2.8,v[axes[1]]/2.8) for v in p])
    _foliage_geom(LEAF_LIGHT,points,faces,uvs)

def _woody_axis(points,radii,sides):
    points=[Vector(p) for p in points];verts=[]
    for j,p in enumerate(points):
        d=(points[min(j+1,len(points)-1)]-points[max(0,j-1)]).normalized()
        q=d.cross(Vector((0,1,0)))
        if q.length<.01:q=d.cross(Vector((1,0,0)))
        q.normalize();r=d.cross(q).normalized()
        for k in range(sides):
            angle=k*math.tau/sides+.15*j
            radius=radii[j]*(1+.07*math.sin(k*2.17+j))
            verts.append(tuple(p+radius*(math.cos(angle)*q+math.sin(angle)*r)))
    faces=[]
    for j in range(len(points)-1):
        for k in range(sides):faces.append((j*sides+k,j*sides+(k+1)%sides,(j+1)*sides+(k+1)%sides,(j+1)*sides+k))
    faces += [tuple(reversed(range(sides))),tuple((len(points)-1)*sides+k for k in range(sides))]
    geom(WOOD,verts,faces)

def _shoot(start,direction,length,width,roll,mat):
    """A botanical shoot on a bent strip, oriented in its supporting branch frame."""
    d=Vector(direction).normalized();u=d.cross(Vector((0,1,0)))
    if u.length<.05:u=d.cross(Vector((1,0,0)))
    u.normalize();n=u.cross(d).normalized()
    side=u*math.cos(roll)+n*math.sin(roll);normal=side.cross(d).normalized()
    verts=[]
    for row,t in enumerate([0,.5,1]):
        center=Vector(start)+d*(length*t)+normal*(length*.10*math.sin(math.pi*t))
        for sign in [-1,1]:verts.append(tuple(center+side*(width*.5*sign)+normal*(width*.035)))
    faces=[(0,1,3,2),(2,3,5,4)]
    uvs=[[(0,0),(1,0),(1,.5),(0,.5)],[(0,.5),(1,.5),(1,1),(0,1)]]
    _foliage_geom(mat,verts,faces,uvs)

def _crown_core(center,radii,phase,mat,low):
    """Asymmetric closed lobe with an offset upper ridge, never a subdivided box."""
    cx,cy,cz=center;rx,ry,rz=radii;segments=4 if low else 8
    if low:
        verts=[]
        for j,yy in enumerate([-.33,.34]):
            for i in range(segments):
                a=i*math.tau/segments+phase+j*.42
                verts.append((cx+rx*math.cos(a)*(.89+.10*math.sin(a*3)),cy+ry*(yy+.09*math.sin(a*2)),cz+rz*math.sin(a)*(.89+.08*math.cos(a*3))))
        verts.extend([(cx-rx*.17,cy-ry*.88,cz+rz*.10),(cx+rx*.21,cy+ry*.92,cz-rz*.12)])
        faces=[f for i in range(segments) for f in [(i,(i+1)%segments,4+(i+1)%segments,4+i),(8,(i+1)%segments,i),(9,4+i,4+(i+1)%segments)]]
        faces=[tuple(reversed(f)) for f in faces]
        _textured_core(verts,faces)
        return
    verts=[]
    # Lower/upper shoulders are independently lobed, not spherical primitives.
    levels=[(-.32,.82),(.34,.78)]
    for j,(height,spread) in enumerate(levels):
        for i in range(segments):
            a=i*math.tau/segments+phase
            wav=1+.15*math.sin(a*3+phase)+.09*math.cos(a*5-.7)
            verts.append((cx+rx*math.cos(a)*spread*wav+rx*.12*j,cy+ry*(height+.07*math.sin(a*2)),cz+rz*math.sin(a)*spread*wav-rz*.08*j))
    verts.extend([(cx-rx*.12,cy-ry*.72,cz+rz*.1),(cx+rx*.19,cy+ry*.80,cz-rz*.1)])
    bottom=2*segments;top=bottom+1;faces=[]
    for i in range(segments):
        k=(i+1)%segments
        faces.extend([(i,k,segments+k,segments+i),(bottom,k,i),(top,segments+i,segments+k)])
    # Dense opaque canopy texture remains stable in distant mip levels.
    faces=[tuple(reversed(f)) for f in faces]
    _textured_core(verts,faces)

def _distant_crown(root,bend,h,crown,species):
    """Same five botanical branch masses as HD, retaining the major gaps."""
    elbow=root+Vector((bend*.5,h*.39,-.3))
    for i in range(5):
        a=i*math.tau/5+.36;spread=crown*(.55 if i<4 else .14)
        sweep=crown*.14 if species=='windswept' else 0
        yy=h*(.73+.04*(i%3))
        if species=='oak':yy+=crown*(.05 if i<4 else .26)
        elif species=='olive':yy-=crown*.05*(i%2)
        center=root+Vector((bend*.6+math.cos(a)*spread+sweep,yy+.60,math.sin(a)*spread))
        radius=crown*(.39 if species=='oak' else .43);aspect=.73 if species=='olive' else .95
        _crown_core(center,(radius*.87,radius*aspect*.85,radius*.81),a,LEAF_LIGHT,True)
        if i<4:
            # Closed tips sit inside trunk/crown, so hidden end caps are omitted.
            before=len(BATCH.get((WOOD,0),([],[],[]))[1])
            _woody_axis([elbow,center],[h*.026,.04],3)
            verts,faces,uvs=BATCH[(WOOD,0)];del faces[-2:];del uvs[-2:]

def _triangle_estimate():
    return sum(sum(len(face)-2 for face in faces) for _,faces,_ in BATCH.values())

def tree(x,y,z,h=13,crown=6,species=None):
    if y==0 and TERRAIN and (round(x/8),round(z/8)) not in TERRAIN:return
    if y==0:y=ground_level(x,z)
    seed=round(x*419+z*173+h*31);local=random.Random(seed)
    bend=local.uniform(-1.5,1.5) # exact same collider offset as the canonical tree
    species=species or ['oak','olive','windswept'][abs(seed)%3]
    before=_triangle_estimate();root=Vector((x,y,z))
    elbow=root+Vector((bend*.5,h*.39,-.3));top=root+Vector((bend,h*.79,.6))
    stem=[root,elbow,top] if LOD else [root,root+Vector((bend*.08,h*.10,-.08)),root+Vector((bend*.15,h*.20,-.12)),elbow,top]
    _woody_axis(stem,[h*.063,h*.039,.13] if LOD else [h*.063,h*.060,h*.052,h*.039,.13],3 if LOD else 8)
    if LOD:
        _distant_crown(root,bend,h,crown,species)
        collision((x+bend*.35,y+h*.32,z+.3),(h*.09,h*.64,h*.09))
        FOLIAGE_METRICS.append({'type':'tree','species':species,'lod':True,'triangles':_triangle_estimate()-before,'position':[x,y,z],'height':h,'crown':crown})
        return
    lobe_data=[]
    for i in range(5):
        a=i*math.tau/5+.36
        spread=crown*(.55 if i<4 else .14)
        sweep=crown*.14 if species=='windswept' else 0
        yy=h*(.73+.04*(i%3))
        if species=='oak':yy+=crown*(.05 if i<4 else .26)
        elif species=='olive':yy-=crown*.05*(i%2)
        center=root+Vector((bend*.6+math.cos(a)*spread+sweep,yy+.60,math.sin(a)*spread))
        branch_end=center-Vector((0,.35,0))
        start=elbow.lerp(top,.15 if i<4 else .5)
        middle=start.lerp(branch_end,.55)+Vector((math.cos(a)*.3,-.25,math.sin(a)*.3))
        _woody_axis([start,branch_end] if LOD else [start,middle,branch_end],[h*.027,.07] if LOD else [h*.027,h*.019,.07],3 if LOD else 5)
        radius=crown*(.39 if species=='oak' else .43)
        aspect=.73 if species=='olive' else .95
        # LOD core reaches the HD shoot envelope, keeping the same five lobes.
        core_scale=1.02 if LOD else .53
        core_r=(radius*core_scale,radius*aspect*core_scale,radius*.9*core_scale)
        mat=LEAF_LIGHT if i in [1,4] else LEAF
        _crown_core(center,core_r,a,mat,LOD)
        if not LOD:
            # Branch tips follow coherent radial phyllotaxis with three tiers.
            # Twenty-two independently pitched shoots wrap each lobe in 3D.
            for j in range(22):
                angle=a+j*2.39996323
                elevation=[-.48,.05,.54,.84][j%4]
                d=Vector((math.cos(angle)*math.sqrt(1-elevation*elevation),elevation,math.sin(angle)*math.sqrt(1-elevation*elevation)))
                base=center+d*(radius*.08)-Vector((0,radius*.15,0))
                length=radius*(1.04+local.uniform(-.13,.10))
                roll=1.2*math.sin(j*2.1)+.15
                _shoot(base,d,length,length*.83,roll,LEAF)
            for j in [0,1]:
                a2=a+(j-.5)*1.1
                tip=center+Vector((math.cos(a2)*radius*.75,.2,math.sin(a2)*radius*.75))
                _woody_axis([branch_end,tip],[.085,.025],3)
        lobe_data.append({'center':list(center),'radius':radius})
    collision((x+bend*.35,y+h*.32,z+.3),(h*.09,h*.64,h*.09))
    FOLIAGE_METRICS.append({'type':'tree','species':species,'lod':LOD,'triangles':_triangle_estimate()-before,'position':[x,y,z],'height':h,'crown':crown,'lobes':lobe_data})

def shrub(x,y,z,width=2.6,height=1.6,depth=2.4,seed=42):
    """Three woody stems with a spreading, ragged leafy envelope; no collider."""
    before=_triangle_estimate();local=random.Random(seed)
    if LOD:
        _crown_core((x,y+height*.5,z),(width*.5,height*.65,depth*.5),seed*.31,LEAF,True)
    else:
        for i in range(3):
            a=i*math.tau/3+.24
            base=Vector((x,y,z));end=base+Vector((math.cos(a)*width*.18,height*.46,math.sin(a)*depth*.18))
            _woody_axis([base,end],[.038,.012],3)
            for j in range(5):
                aa=a+j*2.39996323;pitch=.12+.22*(j%3)
                d=Vector((math.cos(aa),pitch,math.sin(aa))).normalized()
                _shoot(end,d,max(width,depth)*(.42+local.random()*.09),height*.95,local.uniform(-.5,.5),LEAF)
        _crown_core((x,y+height*.54,z),(width*.25,height*.27,depth*.25),seed*.31,LEAF,False)
    FOLIAGE_METRICS.append({'type':'shrub','lod':LOD,'triangles':_triangle_estimate()-before,'position':[x,y,z]})

def geom(mat,points,faces,bevel=0):
    # Non-tree foliage strips use the dense leaf texture, not arbitrary regions
    # of the sparse terminal-twig alpha atlas. Geological vault slots bypass it.
    if mat in [globals().get('LEAF'),globals().get('LEAF_LIGHT')] and str(mat).startswith(('08 |','09 |')):
        _textured_core(points,faces)
    else:_legacy_geom(mat,points,faces,bevel)

def box(p,size,mat,bevel=0,solid=False):
    """Migrate legacy leaf boxes to bounded botanical clusters, never more costly."""
    if mat not in [globals().get('LEAF'),globals().get('LEAF_LIGHT')] or not str(mat).startswith(('08 |','09 |')):
        return _legacy_box(p,size,mat,bevel,solid)
    if min(size)<=0 or (LOD and min(size)<.8):return
    x,y,z=p;w,h,d=size
    if LOD or max(w,d)>h*1.25:
        # Closed asymmetric 8-triangle leaf cushion plus one 4-triangle shoot.
        phase=(x*.72+z*.41)%math.tau
        points=[(x+math.cos(i*math.pi/2+phase)*w*.45,y+h*.05*math.sin(i*1.7),z+math.sin(i*math.pi/2+phase)*d*.45) for i in range(4)]
        points.extend([(x-w*.04,y-h*.46,z+d*.03),(x+w*.07,y+h*.46,z-d*.06)])
        faces=[f for i in range(4) for f in [(4,i,(i+1)%4),(5,(i+1)%4,i)]]
        _textured_core(points,faces)
        if not LOD:_shoot((x,y-h*.25,z),(math.cos(phase)*.25,1,math.sin(phase)*.25),h*.80,min(w,d)*.62,phase,LEAF)
    else:
        # Slender stems/ivy: three bent shoots occupy the original flower box.
        for i in range(3):
            a=i*math.tau/3+(x*.41+z*.27)
            _shoot((x,y-h*.45,z),(math.cos(a)*.13,1,math.sin(a)*.13),h*.82,min(w,d)*.72,a,LEAF)
    if solid:collision(p,size)


# Mineral surface r45. Embedded in build_world.py so its existing source hash
# describes the actual geometry and all four PBR texture bindings.
CRYSTAL_MAPS = {
    'albedo':'assets/textures/aether-quartz-r45-albedo.png',
    'normal':'assets/textures/aether-quartz-r45-normal.png',
    'roughness':'assets/textures/aether-quartz-r45-roughness.png',
    'emission':'assets/textures/aether-quartz-r45-emission.png',
}
CRYSTAL_EMISSION_STRENGTH = 2.5 # same factor as the current Bevy scene binding
_pre_mineral_palette = palette

def crystal_tint(color):
    mat=MATERIALS[CRYSTAL];COLORS[CRYSTAL]=color;mat.diffuse_color=(*color,1)
    for name in ['Mineral albedo tint','Mineral emission tint']:
        mat.node_tree.nodes[name].inputs[2].default_value=(*color,1)

def palette(kind):
    _pre_mineral_palette(kind)
    mat=MATERIALS[CRYSTAL];nodes=mat.node_tree.nodes;links=mat.node_tree.links
    shader=nodes.get('Principled BSDF')
    # These are opaque StandardMaterial inputs. Internal cloudy inclusions are
    # albedo, shallow cleavage is normal, and only selected veins emit light.
    shader.inputs['Transmission Weight'].default_value=0
    shader.inputs['Metallic'].default_value=.08
    shader.inputs['IOR'].default_value=1.5
    shader.inputs['Emission Strength'].default_value=CRYSTAL_EMISSION_STRENGTH
    for role in ['albedo','emission','normal','roughness']:
        tex=nodes.new('ShaderNodeTexImage');tex.name='Mineral '+role
        tex.image=bpy.data.images.load(str(ROOT/CRYSTAL_MAPS[role]),check_existing=True)
        if role in ['normal','roughness']:tex.image.colorspace_settings.name='Non-Color'
        if role in ['albedo','emission']:
            tint=nodes.new('ShaderNodeMixRGB');tint.name='Mineral '+role+' tint'
            tint.blend_type='MULTIPLY';tint.inputs[0].default_value=1
            links.new(tex.outputs['Color'],tint.inputs[1])
            links.new(tint.outputs[0],shader.inputs['Base Color' if role=='albedo' else 'Emission Color'])
        elif role=='normal':
            normal=nodes.new('ShaderNodeNormalMap');normal.inputs['Strength'].default_value=.10
            links.new(tex.outputs['Color'],normal.inputs['Color']);links.new(normal.outputs['Normal'],shader.inputs['Normal'])
        else:
            channels=nodes.new('ShaderNodeSeparateColor');channels.mode='RGB'
            links.new(tex.outputs['Color'],channels.inputs['Color']);links.new(channels.outputs['Green'],shader.inputs['Roughness'])
    color=(.82,.92,1)
    if kind=='frost':color=(.70,1,1)
    elif kind in ['ember','underforge']:color=(1,.64,1)
    elif kind in ['hollow','storm']:color=(.91,.61,1)
    crystal_tint(color)

def _mineral_face(points,uvs):
    verts,faces,coords=BATCH.setdefault((CRYSTAL,0),([],[],[]))
    start=len(verts);verts.extend(tuple(vec(p)) for p in points)
    faces.append(tuple(range(start,start+len(points))));coords.append(uvs)

def _mineral_shell(points,height,root_y,phase,detailed):
    faces=[(2+i,2+(i+1)%6,1) for i in range(6)]
    faces += [(8+i,8+(i+1)%6,2+(i+1)%6,2+i) for i in range(6)]
    faces += [(8+(i+1)%6,8+i,0) for i in range(6)]
    for index,face in enumerate(faces):
        pp=[Vector(points[i]) for i in face]
        # Full-height root-to-tip mapping. Face windows do not restart the
        # gradient; registered cleavage maps follow exactly the same UV set.
        u0=((index%6)*.173+phase)% .66
        if index<6:uv=[(u0,.12),(u0+.32,.12),(u0+.16,0)]
        elif index<12:uv=[(u0,.69),(u0+.32,.69),(u0+.32,.12),(u0,.12)]
        else:uv=[(u0+.32,.69),(u0,.69),(u0+.16,1)]
        if detailed and 6<=index<12:
            # A shallow recessed growth plane leaves narrow polished bevels.
            # Every outer edge is the original hull edge. The inset remains
            # well outside the legacy inscribed collision sections.
            center=sum(pp,Vector())/4;normal=(pp[1]-pp[0]).cross(pp[2]-pp[0]).normalized()
            def bevel_inset(corners):
                result=[]
                for i in range(4):
                    adjacent=[1,0,3,2][i];opposite=[3,2,1,0][i]
                    a=Vector(corners[i]).lerp(Vector(corners[adjacent]),.045)
                    b=Vector(corners[opposite]).lerp(Vector(corners[[1,0,3,2][opposite]]),.045)
                    result.append(a.lerp(b,.008))
                return result
            inset=[p-normal*height*.002 for p in bevel_inset(pp)]
            insetuv=[tuple(p) for p in bevel_inset(uv)]
            _mineral_face(inset,insetuv)
            for i in range(4):
                j=(i+1)%4
                _mineral_face([pp[i],pp[j],inset[j],inset[i]],[uv[i],uv[j],insetuv[j],insetuv[i]])
        else:_mineral_face(pp,uv)

def crystal(p,height,width=None):
    x,y,z=p;r=width or height*.19
    if LOD and height<4:return
    lean=math.sin(x*2.31+z*.54)*height*.11;lean_z=math.cos(z*1.49+x)*height*.09
    points=[(x+lean,y+height,z+lean_z),(x,y-height*.15,z)]
    for level,factor in [(.12,.72),(.69,1)]:
        for i in range(6):
            a=i*math.tau/6
            points.append((x+math.cos(a)*r*factor+lean*level,y+height*level,z+math.sin(a)*r*factor+lean_z*level))
    phase=(math.sin(x*7.31+z*5.43)*.5+.5)*.6
    _mineral_shell(points,height,y,phase,not LOD and height>=5.5)
    if not LOD and height>=5.5:
        # Two short attached hexagonal growths sit within the original crystal
        # AABB, below its first third; they never create a new distant spike.
        bounds=[(min(q[k] for q in points),max(q[k] for q in points)) for k in range(3)]
        for j in range(2):
            a=((round(x*19+z*31)+j*2)%6+.5)*math.tau/6
            direction=Vector((math.cos(a),0,math.sin(a)))
            center=Vector((x+lean*.09,y+height*.055,z+lean_z*.09))+direction*r*.75
            hh=height*(.22 if j==0 else .17);rr=r*(.21 if j==0 else .17)
            tilt=direction*rr*.6
            bud=[tuple(center+Vector((0,hh,0))+tilt),tuple(center-Vector((0,hh*.15,0)))]
            for level,factor in [(.12,.72),(.69,1)]:
                for i in range(6):
                    angle=i*math.tau/6
                    q=center+Vector((math.cos(angle)*rr*factor,hh*level,math.sin(angle)*rr*factor))+tilt*level
                    bud.append(tuple(q))
            # Axial bounds are a contract, including the leaning original tip.
            bud=[tuple(max(bounds[k][0],min(bounds[k][1],q[k])) for k in range(3)) for q in bud]
            _mineral_shell(bud,hh,center.y,(phase+j*.27)% .66,False)
    # Byte-identical legacy colliders and order. No new gameplay surfaces,
    # resource IDs, RNG draws or landmarks are introduced by the surface work.
    if height>=5.5:
        for lo,hi,factor in [(0,.12,.16),(.12,.35,.36),(.35,.59,.42),(.59,.75,.38),(.75,.88,.15)]:
            t=(lo+hi)/2
            collision((x+lean*t,y+height*t,z+lean_z*t),(r*factor*2,height*(hi-lo),r*factor*2))


"""R46 replacement cliff envelope. Closed rock volumes; no physics or RNG edits."""
CLIFF_METRICS=[]

def _emit_closed_rock(points,faces):
    # Fractured bevels are deliberately non-planar. Choose each triangle's
    # metric projection from its final geometric normal, not the first three
    # vertices of an n-gon that Blender triangulates later.
    from mathutils.geometry import tessellate_polygon
    triangles=[]
    for face in faces:
        ring=[Vector(points[i]) for i in face]
        tessellation=tessellate_polygon([ring])
        assert len(tessellation)==len(face)-2,'fracture face failed tessellation'
        triangles.extend(tuple(face[i] for i in tri) for tri in tessellation)
    geom(ROCK,points,triangles)

def _clip_rock_polygon(poly,a,b,c):
    result=[]
    for p,q in zip(poly,poly[1:]+poly[:1]):
        fp=a*p[0]+b*p[1]-c;fq=a*q[0]+b*q[1]-c
        if fp<=1e-9:result.append(p)
        if (fp<0<fq) or (fq<0<fp):
            t=fp/(fp-fq);result.append((p[0]+t*(q[0]-p[0]),p[1]+t*(q[1]-p[1])))
    return result

def _fracture_panel_polygons(width,lo,hi,seed):
    """Elongated geological joint cells; no global horizontal courses."""
    local=random.Random(seed);height=hi-lo
    if height<3.0:return [[(-width/2,lo),(width/2,lo),(width/2,hi),(-width/2,hi)]]
    sites=[]
    lanes=max(1,math.ceil(width/5.5))
    for lane in range(lanes):
        y=lo+local.uniform(1,6)
        while y<hi-1:
            u=-width/2+(lane+.5)*width/lanes+local.uniform(-.32,.32)*width/lanes
            sites.append((u,y));y+=local.uniform(6,13)
    if len(sites)<2:sites=[(-width*.22,lo+height*.25),(width*.22,lo+height*.75)]
    result=[];stretch=.72
    for i,(sx,sy) in enumerate(sites):
        poly=[(-width/2,lo),(width/2,lo),(width/2,hi),(-width/2,hi)]
        for j,(tx,ty) in enumerate(sites):
            if i==j:continue
            a=2*(tx-sx);b=2*stretch*stretch*(ty-sy)
            c=tx*tx-sx*sx+stretch*stretch*(ty*ty-sy*sy)
            poly=_clip_rock_polygon(poly,a,b,c)
            if len(poly)<3:break
        # Boolean clipping can leave sub-centimetre splinters at a canvas
        # corner. They have no readable relief and produce ill-conditioned
        # UV derivatives, so simplify only those near-collinear corners.
        while len(poly)>3:
            thin=None
            for k,p in enumerate(poly):
                a=poly[k-1];b=poly[(k+1)%len(poly)]
                length=math.hypot(b[0]-a[0],b[1]-a[1])
                distance=abs((b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0]))/max(length,1e-9)
                if distance<.08 or math.hypot(p[0]-a[0],p[1]-a[1])<.16:thin=k;break
            if thin is None:break
            poly.pop(thin)
        if len(poly)>=3:
            area2=abs(sum(p[0]*q[1]-p[1]*q[0] for p,q in zip(poly,poly[1:]+poly[:1])))
            perimeter=sum(math.hypot(q[0]-p[0],q[1]-p[1]) for p,q in zip(poly,poly[1:]+poly[:1]))
            if area2/perimeter>.05:result.append(poly)
    return result

def _closed_joint_plate(origin,along,normal,outline,seed,recess,walk_edge=False):
    """A closed tapered rock slab with a deeply inset joint rim and broken face."""
    local=random.Random(seed);n=len(outline);cx=sum(p[0] for p in outline)/n;cy=sum(p[1] for p in outline)/n
    ridge=local.uniform(.75,2.15);tu=local.uniform(-.13,.13);ty=local.uniform(-.022,.022)
    def point(u,y,d):return (origin[0]+along[0]*u+normal[0]*d,y,origin[1]+along[1]*u+normal[1]*d)
    rim=[point(u,y,recess) for u,y in outline];front=[]
    for u,y in outline:
        dist=math.hypot(cx-u,cy-y);t=min(.24,local.uniform(.28,.48)/max(.01,dist))
        uu=u+(cx-u)*t;yy=y+(cy-y)*t
        depth=max(.35,min(2.35,ridge+tu*(uu-cx)+ty*(yy-cy)))
        # An interior terrace face may only recede into its original solid:
        # projecting into the adjacent lower terrace would obstruct walking.
        if walk_edge:depth=-.40+.34*(depth-.35)/2.0
        front.append(point(uu,yy,depth))
    vertices=rim+front;faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]
    for row in range(1):
        for i in range(n):
            j=(i+1)%n;faces.append((row*n+i,row*n+j,(row+1)*n+j,(row+1)*n+i))
    edges={}
    for face in faces:
        for a,b in zip(face,face[1:]+face[:1]):
            key=tuple(sorted((a,b)));edges[key]=edges.get(key,0)+1
    assert all(n==2 for n in edges.values()),'open joint plate'
    _emit_closed_rock(vertices,faces)
    return sum(len(f)-2 for f in faces)

def _continuous_cliff_skin(cells,spacing,shift):
    """Join neighboring exposed wall spans before sculpting fracture plates.

    The geological pattern crosses the cell boundaries instead of repeating a
    slab at every six/eight-metre grid line. A recessed closed core guarantees
    continuity behind the real joints; every outer slab is also a closed volume.
    """
    half=spacing/2+(.02 if spacing==8 else 0);groups={};triangles=0;plates=0
    for (ix,iz),(h,d) in cells.items():
        x,z=ix*spacing,iz*spacing;top=h+shift-(.035 if spacing==6 else 0);bottom=-d+shift
        # Shrink only the external side faces by 65cm. Internal cells remain
        # contiguous; the original soil/moss and collision cap are untouched.
        def inset(dx,dz):
            neighbor=cells.get((ix+dx,iz+dz))
            return .65 if neighbor is None or neighbor[0]<h else 0
        xmin=x-half+inset(-1,0);xmax=x+half-inset(1,0)
        zmin=z-half+inset(0,-1);zmax=z+half-inset(0,1)
        box(((xmin+xmax)/2,(top+bottom)/2,(zmin+zmax)/2),(xmax-xmin,top-bottom,zmax-zmin),ROCK)
        triangles+=12
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            neighbor=cells.get((ix+dx,iz+dz));along=(dz,-dx)
            spans=[(bottom,top)] if neighbor is None else [(bottom,min(top,-neighbor[1]+shift)),(max(bottom,neighbor[0]+shift),top)]
            plane=round(x*dx+z*dz+half,6);u=x*along[0]+z*along[1]
            for lo,hi in spans:
                if hi-lo<1.6:continue
                walk_edge=neighbor is not None and lo>=neighbor[0]+shift-.001
                groups.setdefault((dx,dz,plane,neighbor is None,walk_edge),[]).append((u-spacing/2,u+spacing/2,lo,hi))
    for group,rectangles in groups.items():
        dx,dz,plane,external,walk_edge=group;along=(dz,-dx);normal=(dx,dz);origin=(dx*plane,dz*plane)
        pending=sorted(rectangles);chunks=[]
        while pending:
            chunk=[pending.pop(0)]
            while pending and len(chunk)<4:
                r=pending[0]
                if abs(r[0]-chunk[-1][1])>.001:break
                lo=max(p[2] for p in chunk+[r]);hi=min(p[3] for p in chunk+[r])
                if hi-lo<3:break
                chunk.append(pending.pop(0))
            chunks.append(chunk)
        for chunk in chunks:
            left=chunk[0][0];right=chunk[-1][1];lo=max(r[2] for r in chunk);hi=min(r[3] for r in chunk)
            canvases=[(left,right,lo,hi)]
            for a,b,low,high in chunk:
                if lo-low>=1.6:canvases.append((a,b,low,lo))
                if high-hi>=1.6:canvases.append((a,b,hi,high))
            for a,b,low,high in canvases:
                seed=round(plane*911+a*73+low*129+dx*479+dz*139)
                for i,polygon in enumerate(_fracture_panel_polygons(b-a,low,high,seed)):
                    outline=[(u+(a+b)/2,y) for u,y in polygon]
                    triangles+=_closed_joint_plate(origin,along,normal,outline,seed+i*713,-.60 if external or walk_edge else .025,walk_edge);plates+=1
    return triangles,plates

def _cliff_crown_outcrops(cells,spacing):
    """Discontinuous angular crown rocks and torn shelves, outside walk planes.

    Each rock has an irregular chipped polygon, a leaning back, a broken bevel
    and a tilted front. Large shoulders, medium fragments and small wedges share
    geological direction without becoming repeated boxes or continuous courses.
    """
    triangles=0;rocks=0;rejected=0;max_protrusion=0
    half=spacing/2+(.02 if spacing==8 else 0)
    water_x={'dawn':[-38,41],'dawn-watch':[-36,38],'dawn-garden':[-33,28,-24],'dawn-ruin':[]}.get(CURRENT,[])
    for (ix,iz),(height,depth) in cells.items():
        x,z=ix*spacing,iz*spacing
        for side,(dx,dz) in enumerate([(-1,0),(1,0),(0,-1),(0,1)]):
            if (ix+dx,iz+dz) in cells:continue
            # The walking approaches are cleared against their actual planes
            # below; rocks below the quay remain possible.
            pier=abs(x)<14 and z>42
            if dz>0 and z>30 and any(abs(x-q)<6 for q in water_x):continue
            if CURRENT=='dawn-ruin' and dx<0 and abs(z+3)<9:continue
            local=random.Random(ix*15473+iz*5711+side*673+46069);along=(dz,-dx)
            count=local.randint(6,9)
            for index in range(count):
                emerging=index<2 and local.random()<(.85 if index==0 else .45) and not pier
                if index<2:
                    w=local.uniform(spacing*.50,spacing*1.10);h=local.uniform(3.2,6.3)
                    top=height+(local.uniform(.6,2.9) if emerging else -local.uniform(.25,2.8))
                elif index<5:
                    w=local.uniform(spacing*.48,spacing*1.36);h=local.uniform(2.3,5.8)
                    top=height-local.uniform(.8,13.0)
                else:
                    w=local.uniform(1.2,3.1);h=local.uniform(1.4,4.0)
                    top=height-local.uniform(.5,24)
                if pier:top=min(top,height-4)
                center=local.uniform(-.36,.36)*spacing;y=top-h*.5;roll=local.uniform(-.36,.36)
                # A weathered six-corner section, with its largest cut at a
                # different corner each time. Rotation is geological tilt.
                outline=[(-.48,-.32),(-.20,-.51),(.34,-.43),(.50,.02),(.21,.50),(-.37,.37)]
                raw=[]
                for u,v in outline:
                    uu=u*w*local.uniform(.90,1.08);vv=v*h*local.uniform(.88,1.10)
                    raw.append((uu*math.cos(roll)-vv*math.sin(roll),uu*math.sin(roll)+vv*math.cos(roll)))
                # Let shoulders bridge grid boundaries. The geometric walk
                # clearance below rejects concave-corner intrusions, rather
                # than forcing every fracture back into a repeated cell width.
                ring=[(center+u,y+v) for u,v in raw]
                front=local.uniform(2.05,4.4) if index==0 else local.uniform(1.4,3.7)
                # Raised rocks sit wholly beyond the walk edge. Lower rocks
                # reach into the recessed substrate and are visibly attached.
                back=.20 if emerging else -.45
                def point(u,yy,d):return (x+dx*(half+d)+along[0]*u,yy,z+dz*(half+d)+along[1]*u)
                points=[point(u,yy,back) for u,yy in ring]
                points += [point(u*.93+center*.07,yy*.94+y*.06,front-.26) for u,yy in ring]
                points += [point(u*.80+center*.20,yy*.84+y*.16,front+local.uniform(-.18,.18)) for u,yy in ring]
                n=len(ring);faces=[tuple(reversed(range(n))),tuple(range(n*2,n*3))]
                faces += [(row*n+i,row*n+(i+1)%n,(row+1)*n+(i+1)%n,(row+1)*n+i) for row in range(2) for i in range(n)]
                edges={}
                for face in faces:
                    for a,b in zip(face,face[1:]+face[:1]):
                        key=tuple(sorted((a,b)));edges[key]=edges.get(key,0)+1
                assert all(v==2 for v in edges.values()),'open crown rock'
                # Clip the complete edge graph at each original walk plane.
                # AABB of this clipped volume is conservative: any overlap is
                # rejected, including a sloping bevel crossing the ground.
                safe=True
                for (cx,cz),(ground,_) in cells.items():
                    plane=ground-.15
                    high=[p for p in points if p[1]>=plane]
                    for ia,ib in edges:
                        a,b=points[ia],points[ib]
                        if (a[1]<plane<b[1]) or (b[1]<plane<a[1]):
                            t=(plane-a[1])/(b[1]-a[1]);high.append(tuple(a[k]+t*(b[k]-a[k]) for k in range(3)))
                    if not high:continue
                    if max(p[0] for p in high)>cx*spacing-half-.04 and min(p[0] for p in high)<cx*spacing+half+.04 and max(p[2] for p in high)>cz*spacing-half-.04 and min(p[2] for p in high)<cz*spacing+half+.04:
                        safe=False;break
                if not safe:rejected+=1;continue
                _emit_closed_rock(points,faces);triangles+=sum(len(f)-2 for f in faces);rocks+=1
                max_protrusion=max(max_protrusion,max((p[0]-x)*dx+(p[2]-z)*dz-half for p in points))
    return triangles,rocks,rejected,max_protrusion

def _replace_cliff_terrain(original,get_cells,spacing,shift=0):
    """Consume the legacy calls/RNG and keep all non-rock geometry and physics."""
    global geom,box
    if LOD:return original()
    original_geom=geom;original_box=box;keep_solid=False;removed=0;roots=0
    def filtered_geom(mat,points,faces,bevel=0):
        nonlocal removed,roots
        if mat in [ROCK,ROCK_DARK]:
            amount=sum(len(f)-2 for f in faces)
            if not keep_solid:removed+=amount;return
            roots+=amount
        return original_geom(mat,points,faces,bevel)
    def preserve_box(p,size,mat,bevel=0,solid=False):
        nonlocal keep_solid
        before=keep_solid;keep_solid=solid and mat in [ROCK,ROCK_DARK]
        try:return original_box(p,size,mat,bevel,solid)
        finally:keep_solid=before
    geom=filtered_geom;box=preserve_box
    try:original()
    finally:geom=original_geom;box=original_box
    cells=get_cells();added,plates=_continuous_cliff_skin(cells,spacing,shift)
    crown,crown_rocks,rejected,max_projection=_cliff_crown_outcrops(cells,spacing);added+=crown
    CLIFF_METRICS.append({'kind':CURRENT,'lod':False,'removed_rock_triangles':removed,'new_closed_rock_triangles':added,'unchanged_root_triangles':roots,'net_triangles':added-removed,'closed_bodies':len(cells),'closed_fracture_plates':plates,'crown_rocks':crown_rocks,'crown_triangles':crown,'crown_walk_conflicts_rejected':rejected,'maximum_protrusion_metres':max_projection,'maximum_recess_metres':.65})

_unfractured_base_terrain=terrain
def terrain(kind):
    if kind!='dawn':return _unfractured_base_terrain(kind)
    return _replace_cliff_terrain(lambda:_unfractured_base_terrain(kind),lambda:{key:(ground_level(key[0]*8,key[1]*8),d) for key,d in TERRAIN.items()},8,-.08)


if __name__=='__main__': main()
