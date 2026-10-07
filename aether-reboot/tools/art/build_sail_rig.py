"""Authored naval spars; stage 2 of the reproducible sail pipeline.

Run build_art.py first, this script second, merge_sail_rig.mjs last.
No cloth is regenerated here. Four static material batches are exported.
Blender --background --threads 4 --python tools/art/build_sail_rig.py
"""
import bpy, math, types, hashlib, json
from pathlib import Path
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[2]
HERE=ROOT/'tools/art/sail-source'
HERE.mkdir(parents=True,exist_ok=True)
source=ROOT/'tools/art/build_ship.py'
b=types.ModuleType('naval_authoring');b.__file__=str(source)
code=source.read_text(encoding='utf8').split("if __name__=='__main__':main()",1)[0]
code=code.replace("OUT = ROOT / 'assets/ship'","OUT = Path("+repr(str(ROOT/'.dream-loop/sail-export/static'))+")")
code=code.replace("SOURCE = ROOT / 'tools/art/ship-source'","SOURCE = Path("+repr(str(HERE))+")")
code=code.replace("WORK = ROOT / '.dream-loop/ship-export'","WORK = Path("+repr(str(ROOT/'.dream-loop/sail-export/work'))+")")
exec(compile(code,str(source),'exec'),b.__dict__)
b.clear();b.MATERIALS.clear()
wood=b.material('Cedar | grain',(.35,.16,.055),roughness=.75,texture=True)
brass=b.material('Aged brass',(.53,.29,.075),metallic=.72,roughness=.34)
edge=b.material('Polished brass edge',(.78,.52,.16),metallic=.66,roughness=.28)
rope=b.material('Hemp cordage',(.49,.32,.15),roughness=.95)

def sweep(points,radii,mat,sides=8):
    points=[Vector(p) for p in points];vertices=[]
    for i,p in enumerate(points):
        tangent=(points[min(i+1,len(points)-1)]-points[max(i-1,0)]).normalized()
        cross=tangent.cross(Vector((0,0,1))).normalized()
        binormal=tangent.cross(cross).normalized()
        for j in range(sides):
            a=(j+.5)*math.tau/sides
            vertices.append(tuple(p+radii[i]*(cross*math.cos(a)+binormal*math.sin(a))))
    faces=[tuple(reversed(range(sides))),tuple((len(points)-1)*sides+j for j in range(sides))]
    for i in range(len(points)-1):
        faces.extend((i*sides+j,i*sides+(j+1)%sides,(i+1)*sides+(j+1)%sides,(i+1)*sides+j) for j in range(sides))
    b.geom(mat,vertices,faces)

def cord(points,radius=.009):b.tube(points,radius,rope,5)

def cut(p):
    # Same rest cut as build_art.sail_cut. Border cord stays outside the cloth
    # and its fixed endpoints coincide with the original yard attachment zone.
    x,y,z=p
    u=max(0,min(1,(x+1.25)/2.5));v=max(0,min(1,(y+1.15)/2.30))
    e=math.sin(math.pi*u)*math.sin(math.pi*v)
    return (x*(1-.13*math.sin(math.pi*v)),
            y+math.sin(math.pi*u)*(.13*(1-v)**4-.045*v**6),
            z-.21*e)

def lash_x(x,y,z,radius,turns=3):
    b.tube([(x+(i/(turns*8)-.5)*.045,y+radius*math.cos(i*math.tau/8),z+radius*math.sin(i*math.tau/8)) for i in range(turns*8+1)],.0045,rope,4)

def block(x,y,z,r=.031):
    # Two cheeks flank a recessed sheave, with an open iron-free axle eye.
    for zz in [z-.014,z+.014]:
        b.ring((x,y,zz),r,.009,wood,10)
    b.ring((x,y,z),r*.78,.008,brass,10)
    b.beam((x,y,z-.026),(x,y,z+.026),.008,edge,6)
    b.ring((x,y+r+.014,z),.012,.0045,brass,8)

# An octagonal lower mast transitions into the smaller topmast, with a real
# recessed joint, iron-coloured timber shadow band and turned truck.
b.lathe((0,0,.13),[(-1.49,.066),(-1.43,.069),(-1.36,.058),(-.80,.057),(.62,.051),(1.18,.047),(1.29,.046),(1.32,.058),(1.37,.058),(1.40,.034),(1.65,.030)],wood,8)
for y,r in [(-1.41,.071),(-.42,.062),(.52,.057),(1.20,.052),(1.33,.061),(1.52,.036)]:
    b.lathe((0,0,.13),[(y-.018,r),(y-.012,r+.005),(y+.012,r+.005),(y+.018,r)],brass,8)
b.lathe((0,1.62,.13),[(0,.032),(.017,.057),(.033,.064),(.044,.057),(.060,.035),(.085,.041),(.111,.029),(.140,.012)],edge,8)

# Canted cross-trees and curved knees give the masthead a readable construction
# without creating an unattached platform or overlapping the existing pennant.
for zz in [-.072,.174]:
    sweep([(-.24,1.365,zz),(0,1.39,zz),(.24,1.365,zz)],[.019,.031,.019],wood,6)
for sign in [-1,1]:
    b.tube([(sign*.045,1.22,.13),(sign*.09,1.29,.12),(sign*.18,1.36,.09)],.016,wood,6)
    b.beam((sign*.20,1.365,-.075),(sign*.20,1.365,.188),.013,brass,6)

# Vergues are swept tapered spars. Broad centres, narrower tips, shaped
# ferrules and real seizings replace the previous uniform straight cylinders.
for y in [-1.22,1.23]:
    points=[(-1.368,y-.018,.006),(-1.15,y-.010,0),(-.55,y+.008,.006),(0,y+.012,.013),(.55,y+.008,.006),(1.15,y-.010,0),(1.368,y-.018,.006)]
    sweep(points,[.024,.035,.047,.061,.047,.035,.024],wood,8)
    for sign in [-1,1]:
        sweep([(sign*1.285,y-.014,.004),(sign*1.32,y-.016,.005)],[.033,.031],brass,8)
        lash_x(sign*.68,y+.006,.005,.049)
        lash_x(sign*1.21,y-.012,0,.039)
        for x in [sign*.14,sign*1.295]:
            block(x,y-.085,.012,.025 if abs(x)>1 else .023)
    # Parrel and truss sit around the actual mast/yard crossing.
    b.tube([(-.10,y+.013,.008),(-.084,y+.010,.156),(0,y+.009,.196),(.084,y+.010,.156),(.10,y+.013,.008)],.009,rope,5)
    for x in [-.047,0,.047]:b.ring((x,y+.005,.193),.010,.004,brass,8)

# Topping lifts, outer boltropes and short clew tackles remain attached to the
# same mast and sail corners. These do not substitute for deck-owned shrouds.
for sign in [-1,1]:
    cord([(0,1.60,.13),(sign*.46,1.43,.09),(sign*.93,1.295,.03),(sign*1.30,1.215,.008)],.010)
    cord([cut((sign*(1.285+.005*i/20),-1.18+2.37*i/20,.011+.003*i/20)) for i in range(21)],.008)
    for y in [-1.12,1.115]:
        b.ring(cut((sign*1.15,y,-.027)),.025,.008,brass,8)
    block(sign*1.28,-1.095,.012,.027)
    for dx in [-.016,.016]:
        cord([(sign*1.28+dx,-1.09,.021),(sign*1.295+dx,-1.30,.022)],.005)
    cord([(sign*1.295,-1.30,.022),(sign*1.17,-1.34,.042),(sign*1.04,-1.227,.045)],.007)
    # Halyard return follows the mast, with a small cleat at the mast heel.
    cord([(sign*.055,1.34,.08),(sign*.089,.15,.09),(sign*.093,-1.32,.10)],.007)
    b.tube([(sign*.10,-1.30,.115),(sign*.12,-1.31,.12),(sign*.115,-1.37,.12)],.011,brass,6)

objects=b.make_objects('r44 authored spar and tackle')
bpy.context.scene.render.threads_mode='FIXED';bpy.context.scene.render.threads=4
for image in bpy.data.images:
    if image.source=='FILE' and image.filepath:image.pack()
bpy.ops.wm.save_as_mainfile(filepath=str(HERE/'sail-rig.blend'),compress=True)
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0]
bpy.ops.export_scene.gltf(filepath=str(ROOT/'.dream-loop/sail-export/static-rig.glb'),export_format='GLB',use_selection=True,export_yup=True,export_apply=False,export_animations=False,export_cameras=False,export_lights=False,export_tangents=True)
report={'triangles':sum(len(o.data.polygons) for o in objects),'meshes':len(objects),'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'helper_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'cpu_threads':4,'blend_sha256':hashlib.sha256((HERE/'sail-rig.blend').read_bytes()).hexdigest(),'rig_glb_sha256':hashlib.sha256((ROOT/'.dream-loop/sail-export/static-rig.glb').read_bytes()).hexdigest()}
(HERE/'static-manifest.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report),flush=True)
