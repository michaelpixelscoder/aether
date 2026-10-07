"""R56 grouped terrace vegetation, additive Blender geometry only.
Frozen R54 GLBs/materials are imported; no shared generator or canonical output
is executed or changed. Helpers come from the controlled source snapshot.
"""
import bpy,math,random,json,hashlib,types,struct
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
HERE=Path(__file__).resolve().parent;BASE=HERE/'baseline'
def module(path):
    result=types.ModuleType('r56_frozen_parts');result.__file__=str(path)
    exec(compile(path.read_text(encoding='utf8'),str(path),'exec'),result.__dict__)
    return result
def world_bvh(objects):
    vertices=[];faces=[]
    for obj in objects:
        offset=len(vertices);vertices.extend(obj.matrix_world@v.co for v in obj.data.vertices)
        faces.extend(tuple(offset+i for i in face.vertices) for face in obj.data.polygons)
    return BVHTree.FromPolygons(vertices,faces,all_triangles=False)
collisions=json.loads((BASE/'assets/world/collisions.json').read_text())
marks=json.loads((BASE/'assets/world/landmarks.json').read_text())
manifest=json.loads((BASE/'assets/world/manifest.json').read_text())
layouts={
'dawn':[(-20,8,1.1),(-20,30,.2),(20,8,-.8),(20,31,.1),(-17,47,1.4),(17,46,1.4),(-45,23,.1),(-46,40,.0),(44,39,.0),(47,5,1.3),(-52,-40,.5),(57,-17,1.4),(29,-1,1.5),(-58,52,.4),(60,45,1.3),(30,-62,.1),(-6,-64,.0),(-53,15,1.5),(61,15,1.5),(-27,48,.0)],
'dawn-watch':[(-19,51,.0),(19,54,.0),(-18,33,.1),(17,34,.1),(-19,13,.0),(19,13,.0),(-18,-7,.2),(17,-10,.1),(-18,-26,.4),(7,-44,.1),(26,-52,.0),(-52,27,1.4),(-55,0,1.4),(44,22,1.3),(46,-11,1.4),(-54,-42,.2),(34,-21,.1),(-31,17,.0),(30,19,.0),(-39,36,.1)]}
# A few anchored hanging gardens, selected from true outer ground edges.
ledge_sites={'dawn':[(-65,20,-1,0),(-53,58,0,1),(63,37,1,0),(52,-62,0,-1)],
             'dawn-watch':[(-66,10,-1,0),(-51,49,0,1),(44,49,0,1),(51,-18,1,0)]}
records=[]
for kind in ['dawn','dawn-watch']:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    file=BASE/'assets/world'/(kind+'.glb');bpy.ops.import_scene.gltf(filepath=str(file))
    originals=[o for o in bpy.context.scene.objects if o.type=='MESH']
    soil=world_bvh([o for o in originals if o.name.startswith('12 |')])
    paving=world_bvh([o for o in originals if o.name.startswith(('03 |','04 |','06 |'))])
    materials={m.name:m for m in bpy.data.materials}
    b=module(BASE/'tools/art/build_world.py')
    for token,prefix in [('ROCK','01 |'),('ROCK_DARK','02 |'),('STONE','03 |'),('TRIM','04 |'),('GOLD','05 |'),('WOOD','06 |'),('ROOF','07 |'),('LEAF','08 |'),('LEAF_LIGHT','09 |'),('GLOW','10 |'),('CRYSTAL','11 |'),('SOIL','12 |')]:
        setattr(b,token,next(n for n in materials if n.startswith(prefix)))
    b.MATERIALS={n:m for n,m in materials.items() if n.startswith(('06 |','08 |','09 |'))}
    b.BATCH={};b.LOD=False;b.CURRENT=kind;b.COLLISIONS=[]
    occupied=collisions[kind];landmarks=marks[kind];groups=[];rejections=[]
    def ground(x,z):
        hit,normal,index,distance=soil.ray_cast(Vector((x,-z,150)),Vector((0,0,-1)),200)
        return float(hit.z) if hit is not None else None
    def free(x,z,expected=None):
        if abs(x)<13.5 and z>-56:return False
        y=ground(x,z)
        if y is None or y<-.1 or expected is not None and abs(y-expected)>.30:return False
        for collider in occupied:
            c=collider['center'];s=collider['size']
            if c[1]+s[1]/2<=y+.28 or c[1]-s[1]/2>y+2.7:continue
            if abs(x-c[0])<s[0]/2+.45 and abs(z-c[2])<s[2]/2+.45:return False
        for pool in landmarks['pools']:
            c=pool['position'];s=pool['size']
            if abs(x-c[0])<s[0]/2+1.4 and abs(z-c[2])<s[2]/2+1.4:return False
        hit,normal,index,distance=paving.ray_cast(Vector((x,-z,y+.27)),Vector((0,0,-1)),.6)
        if hit is not None:return False
        return True
    def footprint(x,z,radius):
        y=ground(x,z)
        return y if y is not None and all(free(x+math.cos(a)*radius,z+math.sin(a)*radius,y) for a in [i*math.tau/12 for i in range(12)]) and free(x,z,y) else None
    def lobe(x,y,z,rx,rz,height,phase,seed):
        local=random.Random(seed)
        # A rooted, open shrub with three unequal branch shoulders. Small gaps
        # between them retain the supporting wood and avoid an oval green rock.
        for q,(offset,size,level) in enumerate([(-.43,.48,.36),(.34,.57,.52),(.07,.38,.79)]):
            a=phase+q*1.91
            xx=x+math.cos(a)*rx*offset;zz=z+math.sin(a)*rz*offset;yy=y+height*level
            b._crown_core((xx,yy,zz),(rx*size,height*.35*size/.57,rz*size),phase+q*.73,b.LEAF_LIGHT,False)
            b._woody_axis([(x,y+.05,z),(x+math.cos(a)*rx*.18,y+height*.28,z+math.sin(a)*rz*.18),(xx,yy,zz)],[.063,.038,.014],4)
        # Low coherent botanical shoots emerge from the same asymmetric mass.
        for j in range(8):
            a=phase+j*2.39996323;pitch=.16+.18*(j%3)
            direction=(math.cos(a),pitch,math.sin(a))
            b._shoot((x+math.cos(a)*rx*.38,y+height*.35,z+math.sin(a)*rz*.38),direction,max(rx,rz)*.78,height*.60,local.uniform(-.5,.6),b.LEAF)
        b._woody_axis([(x-rx*.15,y+.06,z),(x,y+height*.43,z),(x+rx*.3,y+height*.50,z-rz*.18)],[.08,.05,.018],4)
    for number,(cx,cz,angle) in enumerate(layouts[kind]):
        accepted=[]
        for j in [-1,0,1]:
            x=cx+math.cos(angle)*j*2.55;z=cz+math.sin(angle)*j*2.55+math.cos(j+number)*.35
            y=footprint(x,z,2.25)
            if y is not None:accepted.append((x,y,z,j))
        if len(accepted)<2:
            rejections.append({'anchor':[cx,cz],'reason':'insufficient continuous planting footprint outside protected surfaces'});continue
        before=b._triangle_estimate()
        for x,y,z,j in accepted:lobe(x,y,z,1.95 if j else 2.10,1.7,1.45+.22*((number+j)%4),angle+j*.4,3000+number*13+j)
        groups.append({'type':'grounded bed','anchor':[cx,cz],'centers':accepted,'triangle_delta':b._triangle_estimate()-before})
        if kind=='dawn' and len(groups)>=12:break
    for number,(cx,cz,dx,dz) in enumerate(ledge_sites[kind]):
        # March toward the real soil boundary, never toward an imaginary ledge.
        anchor=None
        for along in [0,-4,4,-8,8]:
            found=[]
            for step in range(-12,13):
                x=cx+dx*step+dz*along;z=cz+dz*step+dx*along;y=ground(x,z)
                if y is not None:found.append((x,y,z))
            if not found:continue
            x,y,z=found[-1]
            x-=dx*.75;z-=dz*.75
            if not free(x-dx,z-dz,y):continue
            if any(math.hypot(x-f['position'][0],z-f['position'][2])<f['width']/2+3 for f in landmarks['waterfalls']):continue
            anchor=(x,y,z);break
        if anchor is None:continue
        x,y,z=anchor;before=b._triangle_estimate()
        b._woody_axis([(x-dx*2,y+.08,z-dz*2),(x,y+.16,z),(x+dx*.7,y-1.9,z+dz*.7),(x+dx*.9,y-5.4-number*.5,z+dz*.9)],[.22,.17,.10,.026],5)
        for j in range(3):
            xx=x+dx*(.65+.11*j)+dz*math.sin(j+number)*.42;zz=z+dz*(.65+.11*j)+dx*math.sin(j+number)*.42;yy=y-1.0-j*1.5
            b._crown_core((xx,yy,zz),(1.1-j*.16,.85,1.0-j*.12),number+j*.6,b.LEAF_LIGHT,False)
            for q in range(4):
                aa=q*math.tau/4+number
                b._shoot((xx,yy+.1,zz),(math.cos(aa)*.35,-1,math.sin(aa)*.35),1.45-j*.15,.85-j*.13,aa,b.LEAF)
        groups.append({'type':'rooted ledge garden','anchor':anchor,'triangle_delta':b._triangle_estimate()-before})
    assert b.COLLISIONS==[]
    total=b._triangle_estimate();assert manifest[kind]['high']['triangles']+total<=200000,(kind,total)
    patches=b.make_objects()
    for obj in originals:obj['r56_role']='immutable reference'
    for obj in patches:obj['r56_role']='additive landscape geometry'
    for image in bpy.data.images:
        if image.source=='FILE' and image.filepath:image.pack()
    bpy.ops.wm.save_as_mainfile(filepath=str(HERE/'source'/(kind+'-terraces.blend')),compress=True)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in patches:obj.select_set(True)
    bpy.context.view_layer.objects.active=patches[0]
    output=HERE/'source'/(kind+'-additions.glb')
    bpy.ops.export_scene.gltf(filepath=str(output),export_format='GLB',use_selection=True,export_yup=True,export_apply=True,export_texcoords=True,export_normals=True,export_tangents=True,export_materials='EXPORT',export_extras=False)
    records.append({'kind':kind,'base_triangles':manifest[kind]['high']['triangles'],'added_triangles':total,'combined_triangles':manifest[kind]['high']['triangles']+total,'groups':groups,'rejected_anchors':rejections,'collision_changes':0,'lod_changes':0,'materials_reused':['06','08','09'],'shader_material_mutations':False})
    print('TERRACES_R56',json.dumps(records[-1]),flush=True)
(HERE/'evidence/authoring.json').write_text(json.dumps({'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'frozen_helpers_sha256':hashlib.sha256((BASE/'tools/art/build_world.py').read_bytes()).hexdigest(),'islands':records},indent=2)+'\n')
