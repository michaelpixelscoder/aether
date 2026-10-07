"""The Courrier: articulated voxel merchant airship. Blender 5.2, local assets only.

Run: blender --background --python tools/art/build_trader.py
Game coordinates are metres, Y up, forward -Z, hull origin (0,0,0).
HD and LOD preserve MerchantPropellerL/R and their local Z rotation axes.
"""
import bpy, bmesh, math, json, struct, hashlib
from pathlib import Path
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'assets/fauna'
SOURCE=ROOT/'tools/art/traffic-source'
WORK=SOURCE/'.export'
for path in [OUT,SOURCE,WORK]:path.mkdir(parents=True,exist_ok=True)
bpy.context.preferences.filepaths.save_version=0
MATS={};FACTORS={};BATCH={};PART='MerchantBody';LOD=False
PIVOTS={'MerchantBody':(0,0,0),'MerchantPropellerL':(-2.01,.4,2.9),'MerchantPropellerR':(2.01,.4,2.9)}


def vec(p):return Vector((p[0],-p[2],p[1]))


def material(name,color,rough=.7,metal=0,texture=None,normal=False,emission=0):
    m=bpy.data.materials.new(name);m.use_nodes=True
    p=m.node_tree.nodes['Principled BSDF']
    p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=rough;p.inputs['Metallic'].default_value=metal
    if texture:
        t=m.node_tree.nodes.new('ShaderNodeTexImage');t.name='Shared image'
        t.image=bpy.data.images.load(str(ROOT/'assets/textures'/texture),check_existing=True)
        mix=m.node_tree.nodes.new('ShaderNodeMixRGB');mix.name='Authored tint';mix.blend_type='MULTIPLY';mix.inputs[0].default_value=1;mix.inputs[2].default_value=(*color,1)
        m.node_tree.links.new(t.outputs['Color'],mix.inputs[1]);m.node_tree.links.new(mix.outputs[0],p.inputs['Base Color']);FACTORS[name]=color
    if normal:
        t=m.node_tree.nodes.new('ShaderNodeTexImage');t.image=bpy.data.images.load(str(ROOT/'assets/textures/cedar-normal.png'),check_existing=True);t.image.colorspace_settings.name='Non-Color'
        n=m.node_tree.nodes.new('ShaderNodeNormalMap');n.inputs['Strength'].default_value=.30
        m.node_tree.links.new(t.outputs['Color'],n.inputs['Color']);m.node_tree.links.new(n.outputs[0],p.inputs['Normal'])
    if emission:p.inputs['Emission Color'].default_value=(*color,1);p.inputs['Emission Strength'].default_value=emission
    MATS[name]=m;return name


WOOD=material('Merchant | oiled cedar',(.62,.48,.32),.72,texture='cedar.png',normal=True)
BRASS=material('Merchant | aged brass',(.53,.29,.075),.35,.72)
NAVY=material('Merchant | indigo sewn canvas',(.35,.47,.70),.94,texture='indigo-canvas.png')
IVORY=material('Merchant | ivory canvas and stitches',(.72,.61,.39),.91)
AMBER=material('Merchant | warm lantern panes',(1,.38,.060),.28,emission=2.4)


def geom(mat,points,faces,bevel=0,smooth=False):
    vertices,polygons,flags=BATCH.setdefault((PART,mat,bevel),([],[],[]));offset=len(vertices)
    vertices.extend(tuple(p) for p in points);polygons.extend(tuple(offset+i for i in f) for f in faces);flags.extend([smooth]*len(faces))


def box(p,size,mat,bevel=.018):
    x,y,z=p;a,b,c=[s/2 for s in size]
    points=[(x+sx*a,y+sy*b,z+sz*c) for sx,sy,sz in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
    geom(mat,points,[(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)],0 if LOD else min(bevel,min(size)*.15))


def rod(points,radii,mat,sides=6,closed=False):
    points=[Vector(p) for p in points];vertices=[]
    if isinstance(radii,(float,int)):radii=[radii]*len(points)
    for i,p in enumerate(points):
        tangent=(points[(i+1)%len(points)]-points[(i-1)%len(points)]).normalized() if closed else (points[min(i+1,len(points)-1)]-points[max(i-1,0)]).normalized()
        axis=Vector((0,1,0))
        if abs(tangent.dot(axis))>.95:axis=Vector((0,0,1))
        cross=tangent.cross(axis).normalized();other=tangent.cross(cross)
        for j in range(sides):
            a=j*math.tau/sides;vertices.append(tuple(p+radii[i]*(math.cos(a)*cross+math.sin(a)*other)))
    faces=[]
    for j in range(len(points) if closed else len(points)-1):
        k=(j+1)%len(points)
        faces.extend((j*sides+i,j*sides+(i+1)%sides,k*sides+(i+1)%sides,k*sides+i) for i in range(sides))
    if not closed:faces.extend([tuple(reversed(range(sides))),tuple((len(points)-1)*sides+i for i in range(sides))])
    geom(mat,vertices,faces,smooth=False)


def ring(p,radius,thickness,mat=BRASS,axis='z',steps=16):
    x,y,z=p
    points=[(x+radius*math.cos(i*math.tau/steps),y+radius*math.sin(i*math.tau/steps),z) if axis=='z' else
            (x+radius*math.cos(i*math.tau/steps),y,z+radius*math.sin(i*math.tau/steps)) for i in range(steps)]
    rod(points,thickness,mat,4 if LOD else 6,True)


PROFILE=[(-7,0),(-6.72,.24),(-6.0,.56),(-5,.80),(-3.5,.94),(-1.5,1),(1.5,1),(3.5,.94),(5,.78),(6,.51),(6.7,.22),(7,0)]
RY=2.3;RX=2.94;CY=5.64


def envelope_point(z,r,a,lift=0):return ((RX*r+lift)*math.cos(a),CY+(RY*r+lift)*math.sin(a),z)


def envelope():
    sides=12 if LOD else 24
    # The silhouette is a tailored multi-station cigar, not a stretched sphere.
    for station in range(len(PROFILE)-1):
        z0,r0=PROFILE[station];z1,r1=PROFILE[station+1]
        for gore in range(sides):
            a0=gore*math.tau/sides;a1=(gore+1)*math.tau/sides
            # Four broad ivory gores alternate with long indigo panels.
            band=int((gore+.5)/sides*12)%12
            mat=IVORY if band in [2,3,8,9] else NAVY
            if r0==0:
                points=[(0,CY,z0),envelope_point(z1,r1,a0),envelope_point(z1,r1,a1)];faces=[(0,1,2)]
            elif r1==0:
                points=[envelope_point(z0,r0,a0),(0,CY,z1),envelope_point(z0,r0,a1)];faces=[(0,1,2)]
            else:
                points=[envelope_point(z0,r0,a0),envelope_point(z1,r1,a0),envelope_point(z1,r1,a1),envelope_point(z0,r0,a1)];faces=[(0,1,2,3)]
            geom(mat,points,faces)
    if not LOD:
        for gore in range(12):
            a=gore*math.tau/12
            rod([envelope_point(z,r,a,.003) for z,r in PROFILE[1:-1]],.026,IVORY,5)
        # Cross-sewn panel boundaries and short stitch pairs are separate relief.
        for index in [3,5,6,8]:
            z,r=PROFILE[index]
            rod([envelope_point(z,r,i*math.tau/36,.006) for i in range(36)],.020,IVORY,4,True)
            for i in range(24):
                a=i*math.tau/24+.055
                rod([envelope_point(z-.080,r,a,.008),envelope_point(z+.080,r,a+.015,.008)],.011,IVORY,4)
    # Harness belts descend from the sewn envelope to the gondola at four stations.
    for z,r in [PROFILE[3],PROFILE[5],PROFILE[6],PROFILE[8]]:
        if not LOD:rod([envelope_point(z,r,i*math.tau/24,.028) for i in range(24)],.031,WOOD,5,True)
        for sign in [-1,1]:
            a=-math.pi/4 if sign>0 else 5*math.pi/4
            top=envelope_point(z,r,a,.024)
            bottom=(sign*(.76 if abs(z)>4 else 1.08),.78,max(-4.1,min(3.6,z*.80)))
            rod([top,bottom],.040 if LOD else .032,WOOD,4 if LOD else 6)
            if not LOD:
                box(bottom,(.16,.20,.16),BRASS,.018)
                ring((bottom[0],bottom[1]+.12,bottom[2]),.080,.019,BRASS,'z',10)
    if not LOD:
        # A sewn compass identifies a merchant guild on both broad flanks.
        for sign in [-1,1]:
            def project(y,z):
                radius=math.sqrt(max(.01,1-((y-CY)/RY)**2))
                return (sign*(RX*radius+.016),y,z)
            rod([project(CY+.88*math.sin(i*math.tau/24),-.25+.88*math.cos(i*math.tau/24)) for i in range(24)],.013,IVORY,4,True)
            for i in range(8):
                a=i*math.tau/8;length=1.11 if i%2==0 else .83
                corners=[Vector((CY+length*math.sin(a),-.25+length*math.cos(a))),Vector((CY+.21*math.sin(a-.6),-.25+.21*math.cos(a-.6))),Vector((CY+.21*math.sin(a+.6),-.25+.21*math.cos(a+.6)))]
                # Project every interior stitch vertex too: one large planar
                # triangle would cut through the curved balloon surface.
                points=[];lookup={};division=5
                for j in range(division+1):
                    for k in range(division+1-j):
                        u,v=j/division,k/division;p=corners[0]*(1-u-v)+corners[1]*u+corners[2]*v
                        lookup[j,k]=len(points);points.append(project(p.x,p.y))
                faces=[]
                for j in range(division):
                    for k in range(division-j):
                        faces.append((lookup[j,k],lookup[j+1,k],lookup[j,k+1]))
                        if j+k<division-1:faces.append((lookup[j+1,k],lookup[j+1,k+1],lookup[j,k+1]))
                geom(IVORY,points,faces+[tuple(reversed(f)) for f in faces])


HULL=[(-4.9,.07),(-4.3,.54),(-3.2,1.03),(-1.5,1.20),(1.5,1.23),(3.1,1.08),(4.2,.75),(4.65,.30)]


def hull():
    # Six distinct clinker courses, wider at the deck and narrower at the keel.
    courses=3 if LOD else 6
    for course in range(courses):
        y0=-1.62+course*1.77/courses;y1=-1.62+(course+1)*1.77/courses-.012
        scale0=.31+.69*(y0+1.62)/1.77;scale1=.31+.69*(y1+1.62)/1.77
        for (z0,w0),(z1,w1) in zip(HULL,HULL[1:]):
            pts=[(-w0*scale0,y0,z0),(w0*scale0,y0,z0),(w0*scale1,y1,z0),(-w0*scale1,y1,z0),(-w1*scale0,y0,z1),(w1*scale0,y0,z1),(w1*scale1,y1,z1),(-w1*scale1,y1,z1)]
            geom(WOOD,pts,[(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(3,7,6,2),(0,1,5,4)])
    box((0,-1.85,-.05),(.24,.30,6.6),WOOD)
    for sign in [-1,1]:
        for y,scale in [(-.55,.76),(.15,1.01)]:
            rod([(sign*w*scale,y,z) for z,w in HULL],.048,BRASS,4 if LOD else 6)
        # Structural ribs sit proud of the plank courses and carry real rivets.
        if not LOD:
            for z,w in HULL[1:-1]:
                rod([(sign*w*.33,-1.60,z),(sign*w*.68,-.70,z),(sign*w*1.02,.16,z)],.060,WOOD,6)
                for y,sc in [(-1.34,.42),(-.71,.70),(.02,.98)]:
                    box((sign*(w*sc+.023),y,z),(.065,.064,.064),BRASS,.014)
    # Deck boards fill the tapered cross sections; each plank has real relief.
    for (z0,w0),(z1,w1) in zip(HULL,HULL[1:]):
        count=1 if LOD else max(1,round((z1-z0)/.25))
        for j in range(count):
            t=(j+.5)/count;z=z0*(1-t)+z1*t;w=w0*(1-t)+w1*t
            box((0,.17,z),(max(.08,w*2),.052,(z1-z0)/count-.012),WOOD,.008)
    # A raised bow step and carved mooring horns frame the forward working deck.
    box((0,.33,-3.60),(1.56,.25,1.0),WOOD)
    for sign in [-1,1]:
        rod([(sign*.65,.44,-3.50),(sign*.65,.79,-3.50)],[.075,.052],WOOD,6)
        box((sign*.65,.80,-3.50),(.30,.075,.10),BRASS,.016)
        posts=HULL[1:-1] if not LOD else [HULL[2],HULL[4],HULL[6]]
        for z,w in posts:
            rod([(sign*w,.20,z),(sign*w,.92,z)],[.064,.047],WOOD,5)
            if not LOD:box((sign*w,.94,z),(.145,.105,.145),BRASS)
        rod([(sign*w,.94,z) for z,w in HULL[1:-1]],.049,WOOD,4 if LOD else 6)
        rod([(sign*w,.56,z) for z,w in HULL[1:-1]],.026,WOOD,4)


def cabin():
    # Octagonal chart house at the stern, inset amber windows between solid posts.
    box((0,.67,2.60),(1.47,1.02,1.65),WOOD)
    for sign in [-1,1]:
        for z in [2.12,2.69,3.13]:
            box((sign*.746,.87,z),(.023,.39,.34),AMBER,.009)
            if not LOD:
                for dz in [-.195,.195]:box((sign*.773,.87,z+dz),(.065,.49,.042),BRASS,.005)
                for y in [.63,1.11]:box((sign*.773,y,z),(.065,.041,.43),BRASS,.005)
                box((sign*.775,.87,z),(.067,.04,.37),WOOD,.004)
    for x in [-.49,0,.49]:
        box((x,.85,1.765),(.35,.42,.030),AMBER,.007)
        if not LOD:
            for dx in [-.21,.21]:box((x+dx,.85,1.73),(.04,.53,.069),BRASS,.004)
    box((0,1.285,2.60),(1.61,.235,1.79),WOOD,.015)
    for x in [-.62,-.31,0,.31,.62]:
        y=1.38+(1-abs(x)/.75)*.25
        box((x,y,2.60),(.34,.16,1.91),WOOD)
    if not LOD:
        for z in [1.70,3.50]:rod([(-.82,1.37,z),(0,1.68,z),(.82,1.37,z)],.038,BRASS,6)
        # Short chimney/vent, with an open slotted crown under the balloon.
        rod([(.41,1.57,2.9),(.41,2.17,2.9)],[.09,.08],BRASS,8)
        box((.41,2.21,2.9),(.29,.09,.27),BRASS,.012)


def cargo():
    # Merchant payload: strapped crates and two fabric-wrapped barrels.
    for p,size in [((-.50,.51,-.75),(.69,.62,.76)),((.43,.47,-.60),(.65,.54,.70)),((-.45,.47,.23),(.66,.54,.70))]:
        box(p,size,WOOD)
        if not LOD:
            for x in [-size[0]*.29,size[0]*.29]:
                box((p[0]+x,p[1],p[2]),(.055,size[1]+.028,size[2]+.028),BRASS,.006)
            for y in [-size[1]*.27,0,size[1]*.27]:box((p[0],p[1]+y,p[2]-size[2]*.51),(size[0]-.03,.015,.021),BRASS,.003)
    for x,z in [(.46,.30),(.35,.92)]:
        rod([(x,.24,z),(x,.30,z),(x,.76,z),(x,.84,z)],[.22,.265,.265,.21],NAVY,8)
        for y in [.29,.77]:ring((x,y,z),.27,.029,BRASS,'y',8 if LOD else 12)
    if not LOD:
        # Coil and its loose tail remain distinct from the square cargo forms.
        for k in range(3):ring((.1,.25,-2.20),.18+k*.052,.022,WOOD,'y',18)
        rod([(.37,.25,-2.20),(.51,.26,-2.31),(.62,.26,-2.60)],.020,WOOD,5)
        box((0,.53,-3.18),(.20,.60,.20),WOOD)
        ring((0,.95,-3.25),.25,.027,BRASS,'z',16)
        for i in range(6):
            a=i*math.tau/6;rod([(0,.95,-3.25),(.29*math.cos(a),.95+.29*math.sin(a),-3.25)],.020,WOOD,5)


def lantern(p):
    x,y,z=p
    rod([(x,y,z),(x,y+.07,z),(x,y+.39,z),(x,y+.48,z)],[.12,.15,.15,.08],AMBER,6)
    for h,r in [(0,.15),(.42,.19)]:
        ring((x,y+h,z),r,.035,BRASS,'y',6 if LOD else 12)
    if not LOD:
        for i in range(6):
            a=i*math.tau/6;rod([(x+.145*math.cos(a),y,z+.145*math.sin(a)),(x+.145*math.cos(a),y+.41,z+.145*math.sin(a))],.018,BRASS,5)
        rod([(x,y+.43,z),(x,y+.61,z)],[.19,.04],BRASS,6)
        ring((x,y+.67,z),.072,.016,BRASS,'z',12)


def engines():
    global PART
    for sign,label in [(-1,'L'),(1,'R')]:
        x=sign*2.01
        rod([(sign*1.02,.18,2.1),(x,.25,2.1)],.110,BRASS,6)
        rod([(sign*1.05,-.60,1.8),(x,.22,2.1)],.052,BRASS,4 if LOD else 6)
        rod([(x,.4,1.63),(x,.4,1.82),(x,.4,2.48),(x,.4,2.69)],[.13,.25,.25,.15],WOOD,8 if LOD else 12)
        for z in [1.91,2.42]:ring((x,.4,z),.257,.035,BRASS,'z',8 if LOD else 16)
        if not LOD:
            for z in [2.80,3.00]:ring((x,.4,z),.92,.026,BRASS,'z',24)
            for i in range(4):
                a=i*math.pi/2;rod([(x+.245*math.cos(a),.4+.245*math.sin(a),2.42),(x+.92*math.cos(a),.4+.92*math.sin(a),2.82)],.025,BRASS,5)
        PART='MerchantPropeller'+label
        for blade in range(4):
            a=blade*math.pi/2
            profile=[(.15,-.060,-.038),(.47,-.16,-.014),(.85,-.105,.048),(.89,.035,.045),(.49,.105,.012),(.15,.060,.038)]
            points=[]
            for depth in [-.022,.022]:
                for radial,tangent,pitch in profile:
                    points.append((x+radial*math.cos(a)-tangent*math.sin(a),.4+radial*math.sin(a)+tangent*math.cos(a),2.90+pitch+depth))
            n=len(profile);faces=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
            geom(WOOD,points,faces)
            if not LOD:
                tip=[points[i] for i in [2,3,9,8]];geom(BRASS,tip,[(0,1,2,3),(3,2,1,0)])
        rod([(x,.4,2.79),(x,.4,3.02)],[.17,.10],BRASS,8 if LOD else 12)
        PART='MerchantBody'


def compose():
    envelope();hull();cabin();cargo();engines()
    for sign in [-1,1]:lantern((sign*.79,.51,-3.75))
    if not LOD:
        # Fabric tail fins are sewn over light spars, not aerodynamic spheres.
        for sign in [-1,1]:
            pts=[(sign*.10,5.2,4.45),(sign*1.62,5.22,6.02),(sign*1.29,5.16,6.84),(sign*.05,5.08,6.25)]
            geom(NAVY,pts,[(0,1,2,3),(3,2,1,0)])
            rod(pts,.032,IVORY,5,True)


def clear():
    for obj in list(bpy.data.objects):bpy.data.objects.remove(obj,do_unlink=True)
    BATCH.clear()


def objects():
    root=bpy.data.objects.new('MerchantAirship',None);bpy.context.collection.objects.link(root)
    joints={}
    for part,pivot in PIVOTS.items():
        obj=bpy.data.objects.new(part,None);bpy.context.collection.objects.link(obj);obj.parent=root;obj.location=vec(pivot)
        obj['game_pivot_xyz_m']=list(pivot)
        if part!='MerchantBody':obj['rotation_axis']='local Z, game coordinates'
        joints[part]=obj
    raw=[]
    for (part,mat,bevel),(points,faces,flags) in BATCH.items():
        pivot=Vector(PIVOTS[part]);mesh=bpy.data.meshes.new(part+' | '+mat)
        mesh.from_pydata([vec(Vector(p)-pivot) for p in points],[],faces);mesh.update()
        bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.recalc_face_normals(bm,faces=bm.faces);bm.to_mesh(mesh);bm.free();mesh.update()
        uv=mesh.uv_layers.new(name='Metric shared image UV')
        for poly in mesh.polygons:
            axes=[i for i in range(3) if i!=max(range(3),key=lambda j:abs(poly.normal[j]))]
            for li in poly.loop_indices:
                p=mesh.vertices[mesh.loops[li].vertex_index].co+vec(pivot)
                uv.data[li].uv=(p[axes[0]]*.40+.15,p[axes[1]]*.40+.23)
        obj=bpy.data.objects.new(part+' | '+mat,mesh);bpy.context.collection.objects.link(obj);obj.parent=joints[part];mesh.materials.append(MATS[mat]);bpy.context.view_layer.objects.active=obj
        if bevel:
            mod=obj.modifiers.new('Crafted edge chamfers','BEVEL');mod.width=bevel;mod.segments=1;bpy.ops.object.modifier_apply(modifier=mod.name)
            mod=obj.modifiers.new('Weighted broad surfaces','WEIGHTED_NORMAL');mod.keep_sharp=True;bpy.ops.object.modifier_apply(modifier=mod.name)
        raw.append(obj)
    groups={(part,mat):[obj for obj in raw if obj.parent==joints[part] and obj.data.materials[0]==MATS[mat]] for part in PIVOTS for mat in MATS}
    result=[]
    for (part,mat),group in groups.items():
        if not group:continue
        bpy.ops.object.select_all(action='DESELECT')
        for obj in group:obj.select_set(True)
        bpy.context.view_layer.objects.active=group[0]
        if len(group)>1:bpy.ops.object.join()
        obj=group[0];obj.name=part+' | '+mat
        mod=obj.modifiers.new('Explicit triangles','TRIANGULATE');bpy.ops.object.modifier_apply(modifier=mod.name)
        obj.data.validate(clean_customdata=False);result.append(obj)
    bpy.context.view_layer.update();return root,joints,result


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def binary_validation(doc,binary):
    count=0;degenerate=0
    def values(index):
        a=doc['accessors'][index];v=doc['bufferViews'][a['bufferView']]
        components={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[a['type']];fmt,width={5126:('f',4),5123:('H',2),5125:('I',4)}[a['componentType']]
        start=v.get('byteOffset',0)+a.get('byteOffset',0);stride=v.get('byteStride',width*components)
        assert start+(a['count']-1)*stride+width*components<=len(binary)
        return [struct.unpack_from('<'+fmt*components,binary,start+i*stride) for i in range(a['count'])]
    for mesh in doc['meshes']:
        for primitive in mesh['primitives']:
            indices=[v[0] for v in values(primitive['indices'])];positions=values(primitive['attributes']['POSITION'])
            normals=values(primitive['attributes']['NORMAL']);assert all(math.isfinite(n) for p in positions+normals for n in p)
            for i in range(0,len(indices),3):
                a,b,c=[Vector(positions[indices[i+j]]) for j in range(3)]
                if (b-a).cross(c-a).length<1e-10:degenerate+=1
            count+=len(indices)//3
    assert degenerate==0,degenerate
    for name in ['MerchantPropellerL','MerchantPropellerR']:
        joint=next(n for n in doc['nodes'] if n.get('name')==name)
        assert 'rotation' not in joint and 'scale' not in joint
        assert max(abs(a-b) for a,b in zip(joint['translation'],PIVOTS[name]))<1e-5
    return {'triangles':count,'finite_attributes':True,'degenerate_triangles':0,'buffer_bounds':True,'named_pivots':True}


def export(name,meshes):
    source=SOURCE/(name+'.blend')
    for image in bpy.data.images:
        if image.source=='FILE' and image.filepath:image.filepath=bpy.path.relpath(bpy.path.abspath(image.filepath),start=str(SOURCE))
    bpy.context.scene.unit_settings.system='METRIC';bpy.ops.wm.save_as_mainfile(filepath=str(source),compress=True)
    for mat in MATS.values():
        if mat.name in FACTORS:mat.node_tree.links.new(mat.node_tree.nodes['Shared image'].outputs['Color'],mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'])
    temp=WORK/name;temp.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(temp/(name+'.gltf')),export_format='GLTF_SEPARATE',export_yup=True,export_apply=False,export_animations=False,export_cameras=False,export_lights=False,export_extras=True,export_tangents=True)
    for mat in MATS.values():
        if mat.name in FACTORS:mat.node_tree.links.new(mat.node_tree.nodes['Authored tint'].outputs[0],mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'])
    doc=json.loads((temp/(name+'.gltf')).read_text());assert len(doc['buffers'])==1
    binary=(temp/doc['buffers'][0].pop('uri')).read_bytes()
    for im in doc.get('images',[]):
        im['uri']='../textures/'+Path(im['uri']).name;assert (OUT/im['uri']).is_file()
    for mat in doc['materials']:
        if mat['name'] in FACTORS:mat['pbrMetallicRoughness']['baseColorFactor']=[*FACTORS[mat['name']],1]
    validation=binary_validation(doc,binary)
    assert validation['triangles']<=(5000 if LOD else 45000),(name,validation)
    assert len(doc['materials'])<=5
    doc['asset']['extras']={'generator_sha256':sha(Path(__file__)),'axes':'Y up; forward -Z; propellers rotate local Z','units':'metres'}
    encoded=json.dumps(doc,separators=(',',':')).encode();encoded+=b' '*((-len(encoded))%4);binary+=b'\0'*((-len(binary))%4)
    body=struct.pack('<II',len(encoded),0x4e4f534a)+encoded+struct.pack('<II',len(binary),0x004e4942)+binary
    glb=OUT/(name+'.glb');glb.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
    points=[obj.matrix_world@v.co for obj in meshes for v in obj.data.vertices]
    gp=[(p.x,p.z,-p.y) for p in points]
    bounds=[[round(min(p[i] for p in gp),6),round(max(p[i] for p in gp),6)] for i in range(3)]
    assert bounds[0][0]>=-3.001 and bounds[0][1]<=3.001,bounds
    assert bounds[1][0]>=-2.001 and bounds[1][1]<=8.001,bounds
    assert bounds[2][0]>=-7.001 and bounds[2][1]<=7.001,bounds
    assert max(Vector(p).length for p in gp)<12
    return {'file':glb.relative_to(ROOT).as_posix(),'sha256':sha(glb),'bytes':glb.stat().st_size,'source':source.relative_to(ROOT).as_posix(),'source_sha256':sha(source),
        'triangles':validation['triangles'],'materials':len(doc['materials']),'meshes':len(doc['meshes']),'bounds_xyz_m':bounds,'validation':validation,'external_images':[im['uri'] for im in doc.get('images',[])]}


def area(name,position,power,color,size,target):
    data=bpy.data.lights.new(name,'AREA');data.energy=power;data.color=color;data.shape='DISK';data.size=size
    obj=bpy.data.objects.new(name,data);bpy.context.collection.objects.link(obj);obj.location=vec(position);obj.rotation_euler=(vec(target)-obj.location).to_track_quat('-Z','Y').to_euler()


def render_review(name,style):
    clear();bpy.ops.import_scene.gltf(filepath=str(OUT/(name+'.glb')))
    # Confirm each imported rotor moves rigidly around local game Z.
    rig={}
    for part in ['MerchantPropellerL','MerchantPropellerR']:
        joint=bpy.data.objects[part];joint.rotation_mode='XYZ';bpy.context.view_layer.update()
        mesh=next(o for o in joint.children if o.type=='MESH');v=max(mesh.data.vertices,key=lambda v:v.co.length).co.copy()
        before=mesh.matrix_world@v;pivot=joint.matrix_world.translation.copy();joint.rotation_euler.y=.55;bpy.context.view_layer.update();after=mesh.matrix_world@v
        assert (after-before).length>.03 and abs((before-pivot).length-(after-pivot).length)<1e-5
        joint.rotation_euler=(0,0,0);rig[part]=True
    scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=48;scene.cycles.use_denoising=True
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='CUDA';prefs.get_devices();gpu=False
        for d in prefs.devices:d.use=d.type=='CUDA';gpu=gpu or d.use
        if gpu:scene.cycles.device='GPU'
    except Exception:pass
    scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.20,.30,.44,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.45
    area('Broad warm sky',(-10,17,-12),2400,(1,.81,.59),9,(0,3,0))
    area('Cool sky fill',(10,8,-2),1600,(.53,.72,1),8,(0,2,0))
    area('Aft rim',(-2,12,13),3200,(1,.74,.41),8,(0,3,0))
    data=bpy.data.cameras.new('Merchant review');camera=bpy.data.objects.new('Merchant review',data);bpy.context.collection.objects.link(camera)
    camera.location=vec((18,12,-22) if style=='kit' else (10,4,-12) if style=='gondola' else (24,16,-30))
    target=vec((0,3,0) if style!='gondola' else (0,.15,.05));camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler();data.type='ORTHO';data.ortho_scale=18.8 if style=='kit' else 11.8 if style=='gondola' else 38
    scene.camera=camera;scene.view_settings.view_transform='AgX';scene.render.resolution_x=1800;scene.render.resolution_y=1200;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(ROOT/'.dream-loop'/('trader-'+style+'.png'))
    if style=='kit':bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/'presentation.blend'),compress=True)
    bpy.ops.render.render(write_still=True);return rig


def main():
    global LOD,PART
    manifest={'generator':'tools/art/build_trader.py','generator_sha256':sha(Path(__file__)),'axes':'Y up; forward -Z; hull-centre origin',
        'nominal_dimensions_xyz_m':[6,10,14],'required_y_range_m':[-2,8],'physics_radius_m':12,'pivots_game_xyz_m':{k:list(v) for k,v in PIVOTS.items() if k!='MerchantBody'},
        'animation_axis':'local Z for both propellers','shared_images':[{'file':'assets/textures/'+p,'sha256':sha(ROOT/'assets/textures'/p)} for p in ['cedar.png','cedar-normal.png','indigo-canvas.png']], 'assets':{}}
    for name,lod in [('trader',False),('trader-lod',True)]:
        clear();PART='MerchantBody';LOD=lod;compose();root,joints,meshes=objects();report=export(name,meshes);manifest['assets'][name]=report;print('TRADER_ASSET',json.dumps(report),flush=True)
    manifest['reimported_propeller_validation']=render_review('trader','kit')
    render_review('trader','gondola');render_review('trader-lod','distant')
    (OUT/'trader-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('TRADER_COMPLETE',flush=True)


if __name__=='__main__':main()
