"""Three Dawn satellite compositions. The original nine assets are never rebuilt.

Blender --background --python tools/art/build_world_variants.py -- --preview
Blender --background --python tools/art/build_world_variants.py -- --publish
The rotor pivots around local +Z in the exported Y-up GLB.
"""
import bpy, math, random, json, hashlib, sys, types
from pathlib import Path
from mathutils import Vector

HERE=Path(__file__).resolve();ROOT=HERE.parents[2]
BASE=HERE.with_name('build_world.py')
b=types.ModuleType('aether_world_parts');b.__file__=str(BASE)
# Import definitions without executing the nine-island main entry point.
source=BASE.read_text(encoding='utf8')
exec(compile(source,str(BASE),'exec'),b.__dict__)
b.__file__=str(HERE)
PUBLISH='--publish' in sys.argv
OUT=ROOT/'assets/world' if PUBLISH else ROOT/'tools/art/world-preview/variants'
OUT.mkdir(parents=True,exist_ok=True);b.OUT=OUT
KINDS=['dawn-watch','dawn-garden','dawn-ruin'];KIND='';CELLS={};ROUTES=[]
HEIGHTS={ 'dawn-watch':[0,4,9,15,23], 'dawn-garden':[0,3,7,12,18], 'dawn-ruin':[0,3,7,12,18] }
RNG=random.Random(49281)

def height_at(x,z):
    heights=HEIGHTS[KIND]
    index=0 if z>42 else 1 if z>22 else 2 if z>2 else 3 if z>-18 else 4
    height=heights[index]
    # Low, wide channels are carved under the exact staircase volumes.
    if abs(x)<9:
        for i,front in enumerate([50,30,10,-10]):
            if front-17<z<front+3:height=min(height,heights[i])
    return height

def coastline(x,z):
    if abs(x)<=9 and 54<z<=72:return True
    if KIND=='dawn-watch':
        a=math.atan2(z+7,(x+9)*1.10)
        radius=math.hypot((x+9)*1.10,z+7)
        return radius<62+9*math.sin(a*3+.4)+5*math.cos(a*5-1)
    a=math.atan2((z+2)*1.12,x-3);radius=math.hypot((z+2)*1.12,x-3)
    return radius<65+7*math.sin(a*4+.7)+6*math.cos(a*3)

def surface(points,mat):
    b.geom(mat,points,[(0,1,2,3)])

def terrain():
    global CELLS
    CELLS={}
    for ix in range(-12,13):
        for iz in range(-12,13):
            x,z=ix*6,iz*6
            if not coastline(x,z):continue
            h=height_at(x,z)
            if KIND=='dawn-watch':
                d=12+78*math.exp(-((x+24)**2/1250+(z+16)**2/1900))+6*math.sin(x*.07+z*.11)
            else:d=14+41*math.exp(-((x-15)**2+(z+10)**2)/2200)+7*math.cos(x*.09-z*.04)
            d=round(max(9,d)/3)*3
            CELLS[ix,iz]=(h,d)
            b.collision((x,(h-d)/2,z),(6,h+d,6))
            surface([(x-3,h,z-3),(x-3,h,z+3),(x+3,h,z+3),(x+3,h,z-3)],b.SOIL)
            surface([(x-3,-d,z-3),(x+3,-d,z-3),(x+3,-d,z+3),(x-3,-d,z+3)],b.ROCK_DARK)
    for (ix,iz),(h,d) in CELLS.items():
        x,z=ix*6,iz*6
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            adjacent=CELLS.get((ix+dx,iz+dz))
            spans=[(-d,h)] if adjacent is None else [(-d,min(h,-adjacent[1])),(max(-d,adjacent[0]),h)]
            for lo,hi in spans:
                if hi-lo<.01:continue
                if dx:
                    pts=[(x+dx*3,lo,z-3),(x+dx*3,hi,z-3),(x+dx*3,hi,z+3),(x+dx*3,lo,z+3)]
                    if dx<0:pts.reverse()
                else:
                    pts=[(x-3,lo,z+dz*3),(x+3,lo,z+dz*3),(x+3,hi,z+dz*3),(x-3,hi,z+dz*3)]
                    if dz<0:pts.reverse()
                surface(pts,b.ROCK if (ix+iz)%3 else b.ROCK_DARK)
                if not b.LOD:
                    for row in range(math.ceil(lo/4),math.floor(hi/4)):
                        if (ix+iz+row)%3==0:continue
                        yy=row*4+1.5
                        for tangent in [-1.6,1.6]:
                            bump=.7+RNG.random()*1.1
                            if KIND=='dawn-ruin':
                                # Connected exposed strata, with chipped front
                                # faces instead of another layer of cubical lumps.
                                along=(dz,dx);normal=(dx,dz)
                                outline=[(tangent-1.48,yy-1.45),(tangent+1.48,yy-1.25),(tangent+1.26,yy+1.42),(tangent-1.35,yy+1.50)]
                                b.relief_stone((x+dx*3,0,z+dz*3),along,normal,outline,bump*.45,b.ROCK if row%3 else b.ROCK_DARK,ix*917+iz*311+row*71+int(tangent*3))
                            else:
                                b.box((x+dx*(3+bump*.25)+dz*tangent,yy,z+dz*(3+bump*.25)+dx*tangent),(bump,2.9,3.2) if dx else (3.2,2.9,bump),b.ROCK if row%3 else b.ROCK_DARK)
                # Persistent cliff greenery has real volume even in the LOD.
                if adjacent is None and hi>=0 and (ix+iz)%3==0:
                    vine(x+dx*3.3,h,z+dz*3.3,min(42,d*.8),seed=ix*47+iz*97)
                    if not b.LOD:
                        vine(x+dx*3.8+dz*1.7,h-.5,z+dz*3.8+dx*1.7,min(35,d*.6),seed=ix*49+iz*71)
                        for k in range(3):b.box((x+dx*3.7+dz*(k-1)*1.6,h-k*.8,z+dz*3.7+dx*(k-1)*1.6),(3.4,1.9,2.5),b.LEAF_LIGHT if k==1 else b.LEAF)
                if adjacent is not None and hi>0 and not b.LOD:
                    b.box((x+dx*3.1,hi-.35,z+dz*3.1),(1.0,.7,5.7) if dx else (5.7,.7,1.0),b.LEAF)
    # Narrow displaced, tapering stone roots attached to the continuous body.
    for j,(x,z) in enumerate([(-42,-19),(-25,-46),(36,-22),(48,7),(-39,28)]):
        if not coastline(x,z):continue
        cell=CELLS.get((round(x/6),round(z/6)))
        if not cell:continue
        start=-cell[1]+6
        for i in range(5):
            w=12-i*2
            b.box((x+i*.8*math.sin(j),start-i*9,z+i*.8*math.cos(j)),(w,10,w*.8),b.ROCK_DARK if i%2 else b.ROCK,0,True)
    b.box((0,0,80),(14,1,18),b.WOOD,.04,True)
    for z in range(72,89,2):b.box((0,.53,z),(13.8,.10,1.8),b.WOOD,.02)
    for x in [-6.5,6.5]:
        for z in [73,80,87]:b.cyl((x,-1,z),.42,5,b.WOOD,8)
        b.lantern(x,0,75,3)
    for z in [68,62,56]:b.box((0,.10,z),(10,.2,5.8),b.STONE,.025)

def vine(x,y,z,length,seed):
    local=random.Random(seed)
    if b.LOD:
        for i in range(2):
            yy=y-i*length/2-length/4
            b.box((x+math.sin(i+seed)*.6,yy,z),(2.0,length/2+1,1.7),b.LEAF if i%2 else b.LEAF_LIGHT)
        return
    for i in range(max(3,int(length/1.2))):
        x+=local.uniform(-.35,.35);z+=local.uniform(-.3,.3)
        spread=1.2+(1-i/max(1,length/1.2))*.7
        b.box((x,y-i*1.2,z),(spread*local.uniform(.7,1.4),1.65,spread),b.LEAF if i%4 else b.LEAF_LIGHT)
        if i%4==0:b.box((x+.85,y-i*1.2-.5,z+.4),(.9,1.3,.75),b.LEAF)

def tree(x,z,h=12,r=5):
    b.tree(x,height_at(x,z),z,h,r)

def stairs_and_paths():
    heights=HEIGHTS[KIND]
    for i,front in enumerate([50,30,10,-10]):
        rise=heights[i+1]-heights[i]
        b.stairs(0,heights[i],front,10,rise,14)
        # Keep the landing in the coarse mesh and lift it 4 cm off coincident
        # turf faces. The old thin box vanished at LOD and exposed a dark trench.
        thickness=.85 if b.LOD else .5
        b.box((0,heights[i+1]-.5*thickness+.04,front-18),(10,thickness,8),b.STONE,0,True)
        ROUTES.append({'name':f'terrace {i+1}','a':[0,heights[i],front],'b':[0,heights[i+1],front-14]})
        for x in [-6.3,6.3]:b.lantern(x,heights[i+1],front-19,3)
    for x in [-8,8]:
        for z in [34,14,-6,-26]:
            y=HEIGHTS[KIND][[34,14,-6,-26].index(z)+1]
            b.box((x,y-.15,z),(3,.3,5),b.STONE,.04)

def flower_bed(x,y,z,w,d):
    b.box((x,y+.3,z),(w,.6,d),b.STONE,.045)
    b.box((x,y+.7,z),(w-.6,.5,d-.6),b.LEAF)
    if b.LOD:return
    for i in range(int(w*d/2.5)):
        xx=x+RNG.uniform(-w*.43,w*.43);zz=z+RNG.uniform(-d*.42,d*.42)
        b.box((xx,y+1.05,zz),(.4,.7,.4),b.LEAF_LIGHT)
        if i%3==0:b.box((xx,y+1.48,zz),(.45,.22,.45),b.GOLD if i%2 else b.CRYSTAL)

def basin(x,z,w,d,height=64):
    y=height_at(x,z)
    b.box((x,y-.45,z),(w,.7,d),b.STONE,.06,True)
    for dx in [-w/2,w/2]:b.box((x+dx,y+.4,z),(1,.9,d),b.TRIM,.04,True)
    b.box((x,y+.4,z-d/2),(w,.9,1),b.TRIM,.04,True)
    b.LANDMARKS['pools'].append({'position':[x,y+.35,z],'size':[w-.8,.2,d-.5]})
    shore=max(iz*6+3.5 for ix,iz in CELLS if ix==round(x/6))
    front=z+d/2
    if shore>front+1:
        channel=w*.38
        b.box((x,y-.3,(front+shore)/2),(channel+1.2,.7,shore-front),b.STONE,.03,True)
        for side in [-1,1]:b.box((x+side*(channel/2+.3),y+.2,(front+shore)/2),(.6,.5,shore-front),b.TRIM,.02,True)
        for zz in range(math.ceil(front/9)*9,math.floor(shore),9):
            ground=height_at(x,zz)
            if y-ground>1:b.column(x,ground,zz,y-ground-.5,.7)
        b.LANDMARKS['pools'].append({'position':[x,y+.1,(front+shore)/2],'size':[channel,.2,shore-front]})
    else:channel=w-1.2
    b.LANDMARKS['waterfalls'].append({'position':[x,y+.25,shore],'width':channel,'height':height})

def ruins(x,z,width=16):
    y=height_at(x,z)
    b.arch(x,y,z,width,13,2.2,cap=False)
    for side in [-1,1]:
        b.column(x+side*(width*.7),y,z-7,7 if side<0 else 11,1)
        for i in range(3):b.box((x+side*(width*.68+i*1.8),y+1.5-i*.35,z+2+i*.4),(2.4,3-i*.7,2),b.STONE,.08,True)
        vine(x+side*(width*.5+.7),y+10,z+1.2,8,seed=int(x+side*7))

def vegetation(occupied):
    global RNG
    RNG=random.Random(63147+KINDS.index(KIND)*797)
    candidates=[]
    for (ix,iz),(h,d) in CELLS.items():
        x,z=ix*6,iz*6
        if abs(x)<12 or z>58:continue
        if any(abs(x-cx)<w/2+4 and abs(z-cz)<dep/2+4 for cx,cz,w,dep in occupied):continue
        if abs(x)>50 or abs(z)>43 or (ix+iz*3)%4==0:candidates.append((x,z))
    RNG.shuffle(candidates)
    count=26 if KIND=='dawn-ruin' else 40 if KIND=='dawn-watch' else 60
    chosen=[]
    for x,z in candidates:
        if any(math.hypot(x-px,z-pz)<8 for px,pz in chosen):continue
        chosen.append((x,z))
        tree(x,z,RNG.uniform(8,13) if KIND=='dawn-ruin' else RNG.uniform(8,16) if KIND=='dawn-watch' else RNG.uniform(10,19),RNG.uniform(4.0,5.2) if KIND=='dawn-ruin' else RNG.uniform(4.0,6.8))
        if len(chosen)>=count:break
    if not b.LOD:
        for x,z in candidates[:100]:
            y=height_at(x,z)
            for i in range(4):
                b.box((x+RNG.uniform(-2,2),y+.6+RNG.random()*.8,z+RNG.uniform(-2,2)),(RNG.uniform(1,2.8),RNG.uniform(.7,1.5),RNG.uniform(1,2.8)),b.LEAF_LIGHT if i==0 else b.LEAF)

def watch():
    x,z=-25,-35;y=height_at(x,z)
    b.tower(x,y,z,57,7.4)
    # Two cantilevered observation balconies and carved supports.
    for yy in [y+17,y+35]:
        b.cyl((x,yy,z),10,.8,b.STONE,12)
        b.torus((x,yy+2,z),9.6,.16,b.GOLD,'y',24)
        for i in range(12):
            a=i*math.tau/12;xx=x+math.cos(a)*9.5;zz=z+math.sin(a)*9.5
            b.beam((xx,yy,zz),(xx,yy+2.1,zz),.15,b.GOLD,5)
            b.beam((x+math.cos(a)*7,yy-4,z+math.sin(a)*7),(xx,yy-.1,zz),.38,b.TRIM,5)
    # Astrolabe silhouette distinct from the capital's palace dome.
    b.torus((x,y+68,z),4.4,.28,b.GOLD,'z',24)
    b.torus((x,y+68,z),3.8,.22,b.GOLD,'y',24)
    b.crystal((x,y+65,z),5,1.1)
    b.banner(x+10,y+17,z+7,8)
    occupied=[(x,z,26,28)]
    # Observatory library: open arcade, projecting cornice and a crenellated
    # roof terrace attached to the stepped tower, rather than another cylinder.
    b.house(-43,y,-33,12,14,18,False)
    for xx in [-46,-39]:b.arch(xx,y,-22,4.5,9,2.1,cap=False)
    b.box((-42.5,y+10.5,-22),(18,1.1,5),b.TRIM,0,True)
    if not b.LOD:
        b.balustrade(-49,-36,y+14.9,-23.4)
        b.cornice_blocks(-43,y+13,-33,13,19)
        for xx in [-48,-43,-38]:vine(xx,y+14,-23.2,7,seed=int(xx*29))
    occupied.append((-43,-31,20,24))
    b.bridge_x(-30,-12,y,-19,4.5)
    for x,z,w,h,d in [(-28,29,12,9,11),(-44,8,10,12,12),(29,32,11,10,10),(39,6,12,11,12),(31,-34,13,12,12),(-43,-17,9,8,10)]:
        y=height_at(x,z);b.house(x,y,z,w,h,d);occupied.append((x,z,w,d))
        b.district_details(x,y,z+d/2+4)
        flower_bed(x-w*.55,y,z+d/2+2,3,5)
    ruins(18,-40,11);occupied.append((18,-40,26,18))
    for x,z in [(-36,47),(38,39)]:basin(x,z,12,12,65)
    b.portal_gate()
    vegetation(occupied+[(-36,47,14,16),(38,39,14,16)])

def garden():
    # The mill rises out of a terraced garden, not a copy of the palace.
    x,z=24,-30;y=height_at(x,z)
    b.tower(x,y,z,35,6,crown=False)
    b.dome(x,y+34.5,z,7.6,7)
    for yy in [y+9,y+22]:
        b.cyl((x,yy,z),8.2,.6,b.WOOD,12)
        b.torus((x,yy+1.8,z),7.8,.13,b.GOLD,'y',24)
        for i in range(12):
            a=i*math.tau/12
            b.beam((x+math.cos(a)*7.8,yy,z+math.sin(a)*7.8),(x+math.cos(a)*7.8,yy+1.8,z+math.sin(a)*7.8),.14,b.GOLD,5)
    # A real axle remains static; the separately named rotor carries four sails.
    b.beam((x,y+31,z+4),(x,y+31,z+9.5),.9,b.WOOD,12)
    b.house(25,height_at(25,-8),-8,14,10,13)
    b.house(-31,height_at(-31,25),25,12,9,10)
    b.house(-43,height_at(-43,-22),-22,11,11,12)
    # An open garden loggia and two long arbor frames produce fine structural
    # silhouettes among the separated crowns; the axial staircase stays open.
    gy=height_at(-43,25)
    for zz in [18,30]:
        for xx in [-48,-39]:b.column(xx,gy,zz,6.5,.55)
        b.box((-43.5,gy+6.7,zz),(12,.8,1.4),b.TRIM)
    for xx in [-48,-39]:b.box((xx,gy+6.7,24),(1.3,.8,14),b.WOOD)
    if not b.LOD:
        for zz in range(18,32,2):b.box((-43.5,gy+7.1,zz),(12,.28,.30),b.WOOD)
        for zz in [18,24,30]:
            b.box((-47,gy+7.3,zz),(3.2,.6,2.2),b.LEAF)
            vine(-48,gy+7.4,zz,5,seed=700+zz)
    ruins(-30,-45,16);ruins(40,21,11)
    for x,z,w,d in [(-33,5,18,12),(28,43,14,12),(-24,49,12,11)]:basin(x,z,w,d,65 if z>30 else 80)
    for x,z in [(-21,35),(20,19),(-22,-10),(44,-10),(-35,-34),(21,-50)]:
        y=height_at(x,z);flower_bed(x,y,z,9,5)
        if not b.LOD:b.district_details(x,y,z+5)
    # Arched root bundles penetrate ledges and hang well below the stone belly.
    for i,(x,z) in enumerate([(-47,9),(-29,37),(46,17),(51,-16),(-31,-45),(24,-52)]):
        y=height_at(x,z)
        segments=5 if b.LOD else 12
        points=[]
        for j in range(segments+1):
            t=j/segments
            points.append((x*(1-.32*t)+math.sin(t*5+i)*7*t,y+3-(y+87+i)*t,z*(1-.36*t)+math.sin(t*6-i)*5*t))
        for j in range(segments):
            t=j/segments
            b.beam(points[j],points[j+1],max(.3,3.6*(1-t)**1.4),b.WOOD,4 if b.LOD else 7,r2=max(.16,3.6*(1-(j+1)/segments)**1.4))
            if not b.LOD and j in [4,7,9]:
                px,py,pz=points[j]
                sign=1 if j%2 else -1
                middle=(px+sign*5,py-7,pz+3)
                end=(px+sign*8,py-15,pz+7)
                b.beam(points[j],middle,.9,b.WOOD,5,r2=.5)
                b.beam(middle,end,.5,b.WOOD,5,r2=.12)
        vine(x*1.05,y-4,z*1.05,44,seed=930+i)
    occupied=[(24,-30,23,25),(25,-8,18,16),(-31,25,16,14),(-43,-22,15,16),(-43.5,24,15,17),(-30,-45,30,18),(40,21,23,18),(-33,5,20,14),(28,43,16,14),(-24,49,14,13)]
    vegetation(occupied)
    b.LANDMARKS['portal']=None

def rotor():
    if KIND!='dawn-garden':return []
    old=b.BATCH;b.BATCH={}
    hub=Vector((24,HEIGHTS[KIND][-1]+31,-20.2))
    b.beam(tuple(hub+Vector((0,0,-.65))),tuple(hub+Vector((0,0,.65))),1.45,b.GOLD,12)
    # Wood lattice, individual battens, brass perimeter and blue cloth strips.
    for i in range(4):
        angle=i*math.tau/4+math.radians(16)
        radial=Vector((math.sin(angle),math.cos(angle),0));tangent=Vector((math.cos(angle),-math.sin(angle),0))
        pt=lambda r,t,zz=0:tuple(hub+radial*r+tangent*t+Vector((0,0,zz)))
        b.beam(pt(0,0),pt(22,0),.42,b.WOOD,6,r2=.24)
        b.beam(pt(5,5.2),pt(22,5.2),.23,b.WOOD,6)
        for r in [5,8,11,14,17,20,22]:b.beam(pt(r,-.2),pt(r,5.4),.18,b.WOOD,5)
        strips=3 if b.LOD else 8
        for j in range(strips):
            r0=6+j*15/strips;r1=r0+15/strips-.12
            points=[pt(r0,.5,.10),pt(r1,.5,.10),pt(r1,4.75,.10),pt(r0,4.75,.10)]
            b.geom(b.ROOF,points,[(0,1,2,3),(3,2,1,0)])
        if not b.LOD:
            b.beam(pt(5,5.2,.16),pt(22,5.2,.16),.07,b.GOLD,4)
            b.beam(pt(21.8,0,.16),pt(21.8,5.2,.16),.07,b.GOLD,4)
            for r in [9,15,21]:b.beam(pt(r,.5,.14),pt(r,4.8,.14),.07,b.GOLD,4)
    objs=b.make_objects();bpy.ops.object.select_all(action='DESELECT')
    for obj in objs:obj.select_set(True)
    bpy.context.view_layer.objects.active=objs[0]
    if len(objs)>1:bpy.ops.object.join()
    obj=objs[0];obj.name='WindmillRotor';obj.data.name='WindmillRotorGeometry'
    pivot=b.vec(hub)
    for vertex in obj.data.vertices:vertex.co-=pivot
    obj.location=pivot
    obj['aether_animation_axis']='local Z';obj['aether_angular_speed']=.24
    b.BATCH=old
    return [obj]

def ruin_pier(x,y,z,width,depth,height,seed):
    """One coherent, eroded gate pier with an irregular broken crown."""
    b.box((x,y+height/2,z),(width,height,depth),b.STONE,0,True)
    # Dressed courses are separate geometry with shallow chipped arrises.
    for side in [-1,1]:
        b.masonry_skin((x-width/2,y,z+side*(depth/2+.025)),(1,0),(0,side),width,height-.2,height-.2,seed+side)
        b.masonry_skin((x+side*(width/2+.025),y,z-depth/2),(0,1),(side,0),depth,height-.2,height-.2,seed+13+side)
    for yy in [y+.5,y+height*.36,y+height*.70]:
        b.box((x,yy,z),(width+.9,.7,depth+.75),b.TRIM,.045)
    # Outer buttresses taper into the same mass; collision matches each tier.
    for side in [-1,1]:
        for j in range(3):
            h=height*(.56-j*.14)
            b.box((x+side*(width/2+.45),y+h/2,z+depth/2+.65+j*.65),(1.3,h,1.5),b.STONE,0,True)
    for j,(dx,hh) in enumerate([(-.32,2.8),(.02,.9),(.32,4.5)]):
        b.box((x+width*dx,y+height+hh/2,z-.45+j*.35),(width*.29,hh,depth*.76),b.STONE,0,True)
    if not b.LOD:
        for side in [-1,1]:
            b.facade_bay(x,y+height*.41,z+side*(depth/2+.08),width*.34,height*.19,0 if side==1 else math.pi,False)
        vine(x-width*.35,y+height+1,z+depth/2+.4,height*.46,seed)

def ruin_wall(x0,x1,y,z,heights,depth=4.0):
    # Bonded sections form a substantial wall, while the interrupted top
    # exposes the ruin's profile. Gaps are intentional openings, not decals.
    step=(x1-x0)/len(heights)
    for j,h in enumerate(heights):
        xx=x0+(j+.5)*step
        b.box((xx,y+h/2,z),(step+.02,h,depth),b.STONE,0,True)
        for side in [-1,1]:
            b.masonry_skin((xx-step/2,y,z+side*(depth/2+.018)),(1,0),(0,side),step,h-.08,h-.08,40101+j*977+side)
        b.box((xx-.18,y+h+.15,z-.1),(step*.75,.3,depth*.86),b.ROCK,.025)
        if j%3==0:vine(xx,y+h,z+depth/2+.25,min(9,h*.7),seed=40500+j)

def ruin_arch(x,y,z,width,height,depth):
    b.arch(x,y,z,width,height,depth,cap=False)
    # Each overhead voussoir also owns an inscribed physical volume; these
    # stay outside the real opening rather than filling it with a box.
    r=width/2;spring=y+height-r
    for i in range(12):
        a=(i+.5)*math.pi/12
        b.collision((x+math.cos(a)*(r+.775),spring+math.sin(a)*(r+.775),z),(.82,.82,depth-.08))

def ruin_circle(x,y,z):
    # A horizontal portal in a walkable court, as in the board's foreground.
    # The flush tessellated ring does not introduce a curb across access paths.
    pieces=32 if not b.LOD else 20
    for i in range(pieces):
        aa=i*math.tau/pieces+.006;bb=(i+1)*math.tau/pieces-.006
        for inner,outer in [(10.8,12.2),(12.35,14.0)]:
            pts=[(x+math.cos(a)*r,y+yy,z+math.sin(a)*r) for yy in [-.16,.11] for r,a in [(inner,aa),(inner,bb),(outer,bb),(outer,aa)]]
            b.geom(b.TRIM if i%5==0 else b.STONE,pts,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)])
    b.torus((x,y+.18,z),10.55,.46,b.GLOW,'y',48 if not b.LOD else 24,6)
    b.torus((x,y+.20,z),8.15,.16,b.GLOW,'y',48,5)
    b.torus((x,y+.23,z),3.2,.18,b.GLOW,'y',32,5)
    if not b.LOD:
        for i in range(12):
            a=i*math.tau/12
            # Broken radials and inset glyphs read as carved ritual marks.
            for lo,hi in [(3.4,4.3),(6.2,7.8),(11.5,12.9)]:
                b.beam((x+math.cos(a)*lo,y+.25,z+math.sin(a)*lo),(x+math.cos(a)*hi,y+.25,z+math.sin(a)*hi),.075,b.GLOW,4)
            b.torus((x+math.cos(a)*13.2,y+.20,z+math.sin(a)*13.2),.32,.055,b.GOLD,'y',8,4)
        for i in range(44):
            aa=i*.30;bb=(i+1)*.30;ra=1.1+i*.12;rb=1.1+(i+1)*.12
            b.beam((x+math.cos(aa)*ra,y+.27,z+math.sin(aa)*ra),(x+math.cos(bb)*rb,y+.27,z+math.sin(bb)*rb),.055,b.GLOW,4)

def ruin():
    # The forecourt and broken gate replace the garden's domestic silhouette.
    # Axial circulation x=-5..5 remains untouched from dock to upper terrace.
    b.box((28,5.05,18),(30,4.10,28),b.STONE,0,True)
    b.masonry_skin((13,3,32.03),(1,0),(0,1),30,4.05,4.05,36101)
    b.masonry_skin((43.03,3,4),(0,1),(1,0),28,4.05,4.05,36103)
    # Real paved connection to the level-seven axial landing.
    b.box((17,6.85,14),(25,.5,7),b.STONE,0,True)
    ROUTES.append({'name':'portal court','a':[0,7,14],'b':[28,7.1,14]})
    if not b.LOD:
        for ix in range(8):
            for iz in range(7):
                px=15+ix*3.7;pz=5.8+iz*3.8
                b.box((px,7.13,pz),(3.58,.045,3.67),b.TRIM if (ix+iz)%11==0 else b.STONE)
    ruin_circle(28,7.20,18)
    b.LANDMARKS['portal']=[28,18,18]
    # The two uneven shafts and high voussoir arch define the hero silhouette.
    ruin_pier(34,12,-10,6.3,8.5,38,37101)
    ruin_pier(50,12,-10,6.0,8.5,48,37201)
    ruin_arch(42,12,-10,9.8,31,6.7)
    b.box((42,47,-10),(19,5.2,7.8),b.STONE,0,True)
    for side in [-1,1]:b.masonry_skin((32.5,44.4,-10+side*3.92),(1,0),(0,side),19,5.2,5.2,37303+side)
    b.box((42,49.7,-10),(20.0,.7,8.5),b.TRIM,0,True)
    ruin_wall(13,30.8,12,-13,[9,13,15,18,18.8],4.5)
    ruin_wall(30.5,55.5,18,-30,[14,18,23,25,20,18,24],4)
    for xx in [34,49]:
        b.column(xx,18,-24,9 if xx==34 else 14,1.0)
    # Broken arcade on the far-left upper terrace completes a coherent court.
    ruin_arch(-28,18,-33,16,17,3.2)
    ruin_wall(-49,-39,18,-34,[9,14,18],3.6)
    ruin_wall(-17,-12,18,-33,[8,4],3.6)
    b.column(-41,18,-24,9,1.05)
    b.column(-16,18,-25,5.8,1.15)
    # A tall crystal pillar at the concept's left foreground; authored crystal
    # anchors are real resources, with inscribed crystal collision volumes.
    cy=height_at(-28,9)
    b.box((-28,cy+.7,9),(12,1.4,12),b.STONE,.06,True)
    b.column(-28,cy+1.4,9,9,3.0)
    b.crystal_cluster(-28,cy+11.2,9,24)
    for xx,zz,hh in [(-40,35,10),(-18,-43,13),(52,21,7)]:
        b.crystal_cluster(xx,height_at(xx,zz),zz,hh)
    for xx,zz in [(18.5,28.5),(37.5,28.5)]:
        b.column(xx,7.1,zz,3.3,.62)
        b.crystal((xx,10.9,zz),2.5,.65)
    # Only low masonry and roots flank the dock approach, never a second gate.
    for side in [-1,1]:
        ruin_wall(side*20-4,side*20+4,0,48,[2.4,3.8,2.9],2.7)
    b.LANDMARKS['waterfalls'].append({'position':[-49,12,-3],'width':3.0,'height':68})
    # Trees frame the relics; they cannot fill the portal court or main axes.
    occupied=[(28,18,38,38),(42,-10,32,26),(43,-30,34,12),(-28,9,22,22),(-28,-33,45,25),(-40,35,13,13),(-18,-43,14,14),(52,21,12,12)]
    vegetation(occupied)
    # Roots travel down the exterior cliff, below the walkable terraces.
    for j,(xx,zz) in enumerate([(-48,17),(-36,43),(45,30),(53,-16)]):
        yy=height_at(xx,zz)
        root=[(xx,yy,zz),(xx*1.04,yy-8,zz*1.04),(xx*.99,yy-20,zz*1.02),(xx*.92,yy-37,zz*.98)]
        for i in range(3):b.beam(root[i],root[i+1],.8-i*.22,b.WOOD,5,r2=.58-i*.22)
        vine(xx,yy-.6,zz,19+j*3,seed=39101+j)

def compose():
    b.palette('dawn');b.ground_level=height_at;b.coastline=coastline;b.TERRAIN={}
    if KIND=='dawn-ruin':
        # Violet mineral and ritual light belong only to the new ruin; the
        # established watch/garden materials are left byte-for-byte authored.
        b.crystal_tint((.85,.55,1))
        for name,color,strength in [(b.GLOW,(.50,.035,1.0),2.1)]:
            mat=b.MATERIALS[name];shader=mat.node_tree.nodes.get('Principled BSDF')
            shader.inputs['Base Color'].default_value=(*color,1)
            shader.inputs['Emission Color'].default_value=(*color,1)
            shader.inputs['Emission Strength'].default_value=strength
            mat.diffuse_color=(*color,1);b.COLORS[name]=color
    terrain();stairs_and_paths()
    if KIND=='dawn-watch':watch()
    elif KIND=='dawn-garden':garden()
    else:ruin()

def main():
    global KIND,RNG,ROUTES
    collisions=json.loads((ROOT/'assets/world/collisions.json').read_text())
    landmarks=json.loads((ROOT/'assets/world/landmarks.json').read_text())
    manifest=json.loads((ROOT/'assets/world/manifest.json').read_text())
    routes={}
    for kind in KINDS:
        KIND=kind;b.CURRENT=kind
        pair={}
        for low in [False,True]:
            b.clear();b.LOD=low;b.BATCH={};b.COLLISIONS=[];b.LANDMARKS={'waterfalls':[],'pools':[],'crystals':[],'portal':None}
            RNG=random.Random(49281+KINDS.index(kind)*197);b.RNG=RNG;ROUTES=[]
            compose();objects=b.make_objects()+rotor()
            triangle_count=sum(len(o.data.polygons) for o in objects)
            if not low:assert triangle_count<=200000,(kind,triangle_count)
            if not low and PUBLISH:bpy.ops.wm.save_as_mainfile(filepath=str(b.SOURCE/(kind+'.blend')))
            pair['lod' if low else 'high']=b.write_glb(OUT/(kind+('-lod' if low else '')+'.glb'),objects,kind,low)
            b.render_export(OUT/(kind+('-lod' if low else '')+'.glb'),kind+('-lod' if low else ''))
            print('VARIANT',kind,'LOD' if low else 'HD',pair['lod' if low else 'high'],flush=True)
            if not low:collisions[kind]=b.COLLISIONS;landmarks[kind]=b.LANDMARKS;routes[kind]=ROUTES
        assert pair['high']['triangles']<=200000,pair
        assert pair['lod']['triangles']<=12000,pair
        assert pair['high']['materials']<=12 and pair['lod']['materials']<=12,pair
        manifest[kind]={**pair,'colliders':len(collisions[kind]),'dock':[0,3,88],'generator_sha256':hashlib.sha256(HERE.read_bytes()).hexdigest(),'base_generator_sha256':hashlib.sha256(BASE.read_bytes()).hexdigest()}
    for name,value in [('collisions.json',collisions),('landmarks.json',landmarks),('manifest.json',manifest)]:
        (OUT/name).write_text(json.dumps(value,indent=2),encoding='utf8')
    (b.SOURCE/'satellite-walk-routes.json' if PUBLISH else OUT/'variant-routes.json').write_text(json.dumps(routes,indent=2))
    print('VARIANTS COMPLETE',OUT,flush=True)


_unfractured_variant_terrain=terrain
def terrain():
    return b._replace_cliff_terrain(_unfractured_variant_terrain,lambda:CELLS,6)

if __name__=='__main__': main()
