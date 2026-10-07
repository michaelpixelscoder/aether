"""CPU-only R59 closed macrofracture study; canonical inputs remain read-only.

The baseline authoring functions are frozen. Only their two decorative rock
emitters are substituted. Authoritative roots, soil, colliders and all other
material batches are reconstructed unchanged before a GLB payload comparison.
"""
import bpy,math,random,json,hashlib,types,struct,sys
from pathlib import Path
from mathutils import Vector

HERE=Path(__file__).resolve().parent;BASE=HERE/'baseline';OUT=HERE/'output'
def module(name,path):
    result=types.ModuleType(name);result.__file__=str(path)
    exec(compile(path.read_text(encoding='utf8'),str(path),'exec'),result.__dict__)
    return result
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
records=[]

def install(b):
    metrics=[]
    def sculpt(cells,spacing,shift):
        half=spacing/2+(.02 if spacing==8 else 0)
        total=0;plates=0;closed=0;groups={};largest=0;seams=[]
        # The closed substrate stays continuous behind every real recessed
        # joint. Its envelope is the same recessed R46 envelope; soil caps and
        # roots are never passed through this replacement emitter.
        for (ix,iz),(h,d) in cells.items():
            x,z=ix*spacing,iz*spacing;top=h+shift-(.035 if spacing==6 else 0);bottom=-d+shift
            def inset(dx,dz):
                near=cells.get((ix+dx,iz+dz))
                return .65 if near is None or near[0]<h else 0
            xmin=x-half+inset(-1,0);xmax=x+half-inset(1,0)
            zmin=z-half+inset(0,-1);zmax=z+half-inset(0,1)
            b.box(((xmin+xmax)/2,(top+bottom)/2,(zmin+zmax)/2),(xmax-xmin,top-bottom,zmax-zmin),b.ROCK)
            total+=12;closed+=1
            for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
                near=cells.get((ix+dx,iz+dz));along=(dz,-dx)
                spans=[(bottom,top)] if near is None else [(bottom,min(top,-near[1]+shift)),(max(bottom,near[0]+shift),top)]
                plane=round(x*dx+z*dz+half,6);u=x*along[0]+z*along[1]
                for lo,hi in spans:
                    if hi-lo<1.6:continue
                    walk=near is not None and lo>=near[0]+shift-.001
                    groups.setdefault((dx,dz,plane,near is None,walk),[]).append((u-spacing/2,u+spacing/2,lo,hi))
        for (dx,dz,plane,external,walk),rects in sorted(groups.items()):
            along=(dz,-dx);normal=(dx,dz);origin=(dx*plane,dz*plane)
            # Site coordinates span the entire wall plane and every terrace
            # level, not fixed-height slabs or per-cell random columns. A
            # shared site field carries oblique seams through all canvases.
            umin=min(r[0] for r in rects);umax=max(r[1] for r in rects)
            ymin=min(r[2] for r in rects);ymax=max(r[3] for r in rects)
            seed=round(plane*911+dx*479+dz*139+59103)
            local=random.Random(seed);sites=[]
            # Three unequal fracture families. No global horizontal bedding
            # bands are authored, and no RNG draw enters the game authoring.
            for lane in range(max(1,math.ceil((umax-umin)/11.5))):
                u=umin+(lane+.5)*(umax-umin)/max(1,math.ceil((umax-umin)/11.5))
                y=ymin-local.uniform(0,12)
                while y<ymax+18:
                    sites.append((u+local.uniform(-4.8,4.8),y+local.uniform(-5,5),local.uniform(-.28,.28),local.uniform(.8,2.1)))
                    y+=local.uniform(16,29)
            if not sites:sites=[((umin+umax)/2,(ymin+ymax)/2,.1,1.2)]
            # Each exposed cell is only a clip canvas; bevels are attached to
            # the true geological joint, never to its cell boundary.
            for left,right,lo,hi in rects:
                canvas=[(left,lo),(right,lo),(right,hi),(left,hi)]
                for i,(su,sy,tilt,depth) in enumerate(sites):
                    poly=canvas[:];constraints=[];stretch=.55
                    for j,(tu,ty,_,_) in enumerate(sites):
                        if i==j:continue
                        a=2*(tu-su);c=tu*tu-su*su
                        aa=2*stretch*stretch*(ty-sy);cc=stretch*stretch*(ty*ty-sy*sy)
                        poly=b._clip_rock_polygon(poly,a,aa,c+cc)
                        constraints.append((a,aa,c+cc))
                        if len(poly)<3:break
                    if len(poly)<3:continue
                    area=abs(sum(p[0]*q[1]-p[1]*q[0] for p,q in zip(poly,poly[1:]+poly[:1])))*.5
                    if area<.30:continue
                    # Inset only the true fracture boundaries by .25–.60 m;
                    # clipping on a cell/top/bottom boundary has no false rim.
                    gap=.29+.15*((i+seed)%3)
                    front=poly[:]
                    for a,aa,c in constraints:
                        front=b._clip_rock_polygon(front,a,aa,c-gap*math.hypot(a,aa))
                        if len(front)<3:break
                    if len(front)<3:continue
                    cx=sum(p[0] for p in front)/len(front);cy=sum(p[1] for p in front)/len(front)
                    # Close each plate as an actual solid shell. Its plane
                    # tilts across broad faces; a broken shoulder ridge is a
                    # deliberate geometric relief, rather than texture noise.
                    def point(u,y,d):return (origin[0]+along[0]*u+normal[0]*d,y,origin[1]+along[1]*u+normal[1]*d)
                    if walk:
                        dd=lambda u,y:-.28+.05*math.sin((u-su)*.18+sy)
                    else:
                        dd=lambda u,y:max(.38,min(2.35,depth+tilt*(u-su)*.11+(y-sy)*.018))
                    # The back follows the same inset boundary. Geological
                    # gaps show the continuous closed core at -.65m.
                    n=len(front);vertices=[point(u,y,-.60) for u,y in front]
                    vertices += [point(u,y,dd(u,y)) for u,y in front]
                    # A large off-center crown breaks the smooth slab face.
                    ridge=(cx+(su-cx)*.15,cy+(sy-cy)*.12)
                    vertices.append(point(*ridge,dd(*ridge)+(0 if walk else .35)))
                    faces=[tuple(reversed(range(n)))]
                    faces += [(n+k,n+(k+1)%n,2*n) for k in range(n)]
                    faces += [(k,(k+1)%n,n+(k+1)%n,n+k) for k in range(n)]
                    b._emit_closed_rock(vertices,faces)
                    amount=sum(len(f)-2 for f in faces);total+=amount;plates+=1
                    largest=max(largest,max(p[1] for p in front)-min(p[1] for p in front))
                    seams.append(gap*2)
        metrics.append({'closed_substrate_cells':closed,'macroplates':plates,'rock_triangles':total,'max_plate_vertical_span_m':largest,'true_joint_width_range_m':[min(seams),max(seams)] if seams else [],'wall_site_field_shared_across_cells':True,'walk_faces_recess_only':True})
        return total,plates
    def no_small_stacked_crowns(cells,spacing):
        # The old 6–9 small stones per exposed cell make the horizontal rows
        # read like rounded courses. Their removal funds macrofractures.
        return 0,0,0,0
    b._continuous_cliff_skin=sculpt;b._cliff_crown_outcrops=no_small_stacked_crowns
    return metrics

for kind in ['dawn','dawn-watch']:
    for candidate in [False,True]:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        if kind=='dawn':b=module('r59_base',BASE/'tools/art/build_world.py');v=None
        else:v=module('r59_variant',BASE/'tools/art/build_world_variants.py');b=v.b
        b.clear();b.CURRENT=kind;b.LOD=False;b.BATCH={};b.COLLISIONS=[];b.LANDMARKS={'waterfalls':[],'pools':[],'crystals':[],'portal':None}
        b.OUT=OUT;b.SOURCE=HERE/'source';b.SOURCE.mkdir(exist_ok=True)
        metric=install(b) if candidate else []
        if v:v.KIND=kind;v.RNG=random.Random(49281);b.RNG=v.RNG;v.ROUTES=[];v.compose()
        else:b.RNG=random.Random(74019);b.compose(kind)
        collisions=json.loads((BASE/'assets/world/collisions.json').read_text())[kind]
        marks=json.loads((BASE/'assets/world/landmarks.json').read_text())[kind]
        assert b.COLLISIONS==collisions,'collision authority changed'
        assert b.LANDMARKS==marks,'landmark authority changed'
        # ROCK_DARK and every unrelated primitive are excluded from the
        # generated payload; the assembler always takes them from the exact
        # R56 baseline bytes, so R54 architecture/R56 terraces cannot regress.
        b.BATCH={key:value for key,value in b.BATCH.items() if key[0]==b.ROCK}
        objects=b.make_objects()
        for img in bpy.data.images:
            if img.source=='FILE' and img.filepath:img.pack()
        name=kind+('-candidate' if candidate else '-regenerated-baseline')
        bpy.ops.wm.save_as_mainfile(filepath=str(HERE/'source'/(name+'.blend')),compress=True)
        result=b.write_glb(OUT/(name+'.glb'),objects,kind)
        records.append({'kind':kind,'candidate':candidate,'payload':result,'metrics':metric,'source_sha256':sha(HERE/'source'/(name+'.blend')),'collisions_exact':True,'landmarks_exact':True,'walking_routes':v.ROUTES if v else None,'legacy_cliff_metrics':b.CLIFF_METRICS})
        print('R59_CPU',json.dumps(records[-1]),flush=True)
(HERE/'build-evidence.json').write_text(json.dumps({'schema':1,'author_sha256':sha(Path(__file__)),'records':records},indent=2)+'\n')
