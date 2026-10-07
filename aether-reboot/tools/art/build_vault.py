"""Regional open-bottom cavern shell, Y-up metres; independent of island assets.

Blender --background --python tools/art/build_vault.py
Two 500 m axial entries, two >=400 m skylights, no floor. All colliders are
inside visible structural rock; no collider can seal an opening.
"""
import bpy, math, random, json, hashlib, types, sys
from pathlib import Path
from mathutils import Vector

HERE=Path(__file__).resolve();ROOT=HERE.parents[2];BASE=HERE.with_name('build_world.py')
b=types.ModuleType('vault_parts');b.__file__=str(BASE)
exec(compile(BASE.read_text(encoding='utf8'),str(BASE),'exec'),b.__dict__)
b.__file__=str(HERE)
OUT=ROOT/'assets/world';RNG=random.Random(103941);ROOF={};WALL={};UPLANDS={}

def rect_distance(x,z,half,cx=0,cz=0):
    return math.hypot(max(0,abs(x-cx)-half),max(0,abs(z-cz)-half))

def outside_skylights(x,z,half,clearance=200):
    return all(rect_distance(x,z,half,cx,0)>=clearance for cx in [-700,700])

def palette():
    b.MATERIALS.clear();b.COLORS.clear()
    texture='assets/textures/dressed-limestone-r11.png'
    b.ROCK=b.material('01 | Vault weathered limestone',b.rock_factor((.21,.23,.24)),b.ROCK_ALBEDO,roughness=.94,normal_texture=b.ROCK_NORMAL,roughness_texture=b.ROCK_ROUGHNESS)
    b.ROCK_DARK=b.material('02 | Vault deep fracture',b.rock_factor((.135,.15,.16)),b.ROCK_ALBEDO,roughness=.98,normal_texture=b.ROCK_NORMAL,roughness_texture=b.ROCK_ROUGHNESS)
    b.STONE=b.material('03 | Vault silver strata',(.29,.30,.29),texture,roughness=.91)
    b.LEAF=b.material('04 | Vault muted lichen',(.075,.102,.081),roughness=1)
    b.LEAF_LIGHT=b.material('05 | Vault mineral lichen',(.13,.155,.132),roughness=.96)
    # Shared primitive/export helpers also accept these architectural aliases.
    b.TRIM=b.STONE;b.WOOD=b.ROCK

def quad(mat,pts):b.geom(mat,pts,[(0,1,2,3)])

def shell(cells,step,roof=False):
    half=step/2
    for (ix,iz),(lo,hi) in cells.items():
        x=(ix+.5)*step;z=(iz+.5)*step
        quad(b.ROCK,[(x-half,hi,z-half),(x-half,hi,z+half),(x+half,hi,z+half),(x+half,hi,z-half)])
        quad(b.ROCK if roof else b.ROCK_DARK,[(x-half,lo,z-half),(x+half,lo,z-half),(x+half,lo,z+half),(x-half,lo,z+half)])
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            other=cells.get((ix+dx,iz+dz))
            spans=[(lo,hi)] if other is None else [(lo,min(hi,other[0])),(max(lo,other[1]),hi)]
            for bottom,top in spans:
                if top-bottom<.01:continue
                if dx:
                    pts=[(x+dx*half,bottom,z-half),(x+dx*half,top,z-half),(x+dx*half,top,z+half),(x+dx*half,bottom,z+half)]
                    if dx<0:pts.reverse()
                else:
                    pts=[(x-half,bottom,z+dz*half),(x+half,bottom,z+dz*half),(x+half,top,z+dz*half),(x-half,top,z+dz*half)]
                    if dz<0:pts.reverse()
                quad(b.STONE if (ix+iz)%5==0 else b.ROCK_DARK,pts)
                if not b.LOD and not roof:
                    # Offset shale shelves and fractured ends make every wall
                    # a stratified massif rather than a single broad panel.
                    for row in range(-5,4):
                        local=random.Random(ix*9359+iz*7727+row*373)
                        if local.random()<.28:continue
                        y=row*112+local.uniform(-14,14)
                        along=local.uniform(-12,12);depth=local.uniform(7,24)
                        px=x+dx*(half+depth*.22)+dz*along;pz=z+dz*(half+depth*.22)+dx*along
                        width=local.uniform(30,61);height=local.uniform(48,90)
                        if abs(px)-width/2<250 and abs(pz)>1200:continue
                        b.box((px,y,pz),(depth,height,width) if dx else (width,height,depth),b.STONE if row%4==0 else b.ROCK)
                        if row in [0,2] and (ix+iz)%4==0:
                            b.box((px+dx*depth*.55,y+height*.35,pz+dz*depth*.55),(4,18,width*.72) if dx else (width*.72,18,4),b.LEAF)
        if roof and not b.LOD and (ix*3+iz)%4==0:
            # Ceiling facets remain within the requested Y350..450 envelope.
            local=random.Random(ix*93+iz*937)
            sy=min(25,lo-350)
            if sy>1 and outside_skylights(x,z,half):
                b.box((x,lo-sy*.35,z),(step*.76,sy,step*.68),b.STONE if (ix+iz)%3==0 else b.ROCK_DARK)

def merge_colliders(cells,step):
    remaining=set(cells)
    while remaining:
        ix,iz=min(remaining,key=lambda p:(p[1],p[0]));height=cells[ix,iz]
        width=1
        while (ix+width,iz) in remaining and cells[ix+width,iz]==height:width+=1
        depth=1
        while all((xx,iz+depth) in remaining and cells[xx,iz+depth]==height for xx in range(ix,ix+width)):depth+=1
        for xx in range(ix,ix+width):
            for zz in range(iz,iz+depth):remaining.remove((xx,zz))
        lo,hi=height
        b.collision(((ix+width/2)*step,(lo+hi)/2,(iz+depth/2)*step),(width*step,hi-lo,depth*step))

def build_roof():
    global ROOF
    step=100 if b.LOD else 50;ROOF={}
    for ix in range(-33 if not b.LOD else -17,33 if not b.LOD else 17):
        for iz in range(-33 if not b.LOD else -17,33 if not b.LOD else 17):
            x=(ix+.5)*step;z=(iz+.5)*step;r=math.hypot(x,z)
            if r>1630 or not outside_skylights(x,z,step/2):continue
            a=math.atan2(z,x)
            def heights(px,pz):
                n=.48+.19*math.sin(px*.004+pz*.003)+.13*math.cos(px*.010-pz*.007)+.09*math.sin(px*.027+pz*.019)
                lower=350+min(1,max(0,n))*42
                upper=420+min(1,max(0,.55+.25*math.sin(px*.007-pz*.005)+.12*math.cos(px*.024+pz*.013)))*30
                return lower,upper
            samples=[heights(x+dx*step/2,z+dz*step/2) for dx,dz in [(-1,-1),(-1,1),(1,1),(1,-1)]]
            # The collider is inscribed between the highest underside corner
            # and the lowest top corner, hence cannot protrude from the rock.
            ROOF[ix,iz]=(math.ceil(max(p[0] for p in samples)/10)*10,math.floor(min(p[1] for p in samples)/10)*10)
    for (ix,iz),(lo,hi) in ROOF.items():
        x=(ix+.5)*step;z=(iz+.5)*step;half=step/2
        corners=[(x+dx*half,z+dz*half) for dx,dz in [(-1,-1),(-1,1),(1,1),(1,-1)]]
        upper=[(px,heights(px,pz)[1],pz) for px,pz in corners]
        lower=[(px,heights(px,pz)[0],pz) for px,pz in corners]
        quad(b.ROCK,upper)
        if b.LOD:quad(b.ROCK,list(reversed(lower)))
        else:
            local=random.Random(ix*7919+iz*4177)
            center=(x+local.uniform(-half*.28,half*.28),max(350,heights(x,z)[0]-local.uniform(1,11)),z+local.uniform(-half*.28,half*.28))
            points=lower+[center]
            mat=b.ROCK_DARK if math.sin(x*.006+z*.003)>.5 else b.ROCK
            b.geom(mat,points,[(4,1,0),(4,2,1),(4,3,2),(4,0,3)])
        for edge,(dx,dz) in enumerate([(-1,0),(0,1),(1,0),(0,-1)]):
            if (ix+dx,iz+dz) in ROOF:continue
            nxt=(edge+1)%4
            quad(b.STONE,[lower[edge],lower[nxt],upper[nxt],upper[edge]])
    if not b.LOD:merge_colliders(ROOF,step)

def build_uplands():
    """Closed stratified massifs above the unchanged navigable cavern ceiling."""
    global UPLANDS
    step=200 if b.LOD else 100;half=step/2;base_step=100 if b.LOD else 50
    UPLANDS={}
    # Broad unequal ridges, not another thin wavy disk. A low saddle between
    # the two skylights keeps their 400 m openings immediately recognisable.
    peaks=[(-1160,-780,198,430,340),(1120,710,194,370,520),(-190,-1010,168,610,370),(-380,1130,157,560,330),(-1120,390,137,330,580),(1030,-740,172,510,340),(40,230,112,490,530)]
    for ix in range(-17 if not b.LOD else -9,17 if not b.LOD else 9):
        for iz in range(-17 if not b.LOD else -9,17 if not b.LOD else 9):
            x=(ix+.5)*step;z=(iz+.5)*step
            # Every upper block sits entirely on an existing roof cell; there
            # is no overhang into a skylight or an unsupported floating panel.
            scale=round(step/base_step)
            if not all((ix*scale+dx,iz*scale+dz) in ROOF for dx in range(scale) for dz in range(scale)):continue
            if not outside_skylights(x,z,half):continue
            ridge=max(peak*math.exp(-((x-cx)/wx)**2-((z-cz)/wz)**2) for cx,cz,peak,wx,wz in peaks)
            cut=10*math.sin(x*.013+z*.005)+9*math.cos(z*.016-x*.004)
            hi=min(650,450+20*max(1,math.floor((ridge+cut+12)/20)))
            UPLANDS[ix,iz]=(418,hi)
    for (ix,iz),(lo,hi) in UPLANDS.items():
        x=(ix+.5)*step;z=(iz+.5)*step
        moss=hi<570 and math.sin(x*.003-z*.007)+math.cos(x*.008+z*.004)>1.0
        top=(b.LEAF_LIGHT if (ix+iz)%4==0 else b.LEAF) if moss else b.ROCK if (ix*3+iz)%5 else b.STONE
        quad(top,[(x-half,hi,z-half),(x-half,hi,z+half),(x+half,hi,z+half),(x+half,hi,z-half)])
        quad(b.ROCK_DARK,[(x-half,lo,z-half),(x+half,lo,z-half),(x+half,lo,z+half),(x-half,lo,z+half)])
        for dx,dz in [(-1,0),(1,0),(0,-1),(0,1)]:
            other=UPLANDS.get((ix+dx,iz+dz))
            bottom=other[1] if other else lo
            if bottom>=hi:continue
            # Horizontal strata have actual section changes and shadowed ledges.
            levels=[bottom,hi] if b.LOD else [bottom]+[yy for yy in range(440,651,25) if bottom<yy<hi]+[hi]
            for j,(aa,bb) in enumerate(zip(levels,levels[1:])):
                if dx:
                    pts=[(x+dx*half,aa,z-half),(x+dx*half,bb,z-half),(x+dx*half,bb,z+half),(x+dx*half,aa,z+half)]
                    if dx<0:pts.reverse()
                else:
                    pts=[(x-half,aa,z+dz*half),(x+half,aa,z+dz*half),(x+half,bb,z+dz*half),(x-half,bb,z+dz*half)]
                    if dz<0:pts.reverse()
                mat=b.STONE if int(aa/25)%5==0 else b.ROCK_DARK if int(aa/25)%3==0 else b.ROCK
                quad(mat,pts)
                if not b.LOD and bb-aa>=18 and (ix+iz+j)%3==0:
                    depth=8;along=12*math.sin(ix*7+iz*3+j)
                    px=x+dx*(half+depth*.18)+dz*along;pz=z+dz*(half+depth*.18)+dx*along
                    size=(depth,min(12,bb-aa),step*.63) if dx else (step*.63,min(12,bb-aa),depth)
                    if outside_skylights(px,pz,max(size[0],size[2])/2):
                        b.box((px,aa+(bb-aa)*.65,pz),size,mat,0,True)
    if not b.LOD:merge_colliders(UPLANDS,step)
    # Unequal, offset edge crags produce a broken outer skyline. Their lower
    # courses overlap the roof; every protruding ledge has matching collision.
    for i,(x,z,width,levels) in enumerate([(-1520,-330,190,3),(-1370,-820,240,4),(-880,-1330,170,3),(-230,-1500,230,4),(470,-1450,190,2),(1380,-680,220,4),(1490,170,190,3),(1230,990,250,4),(520,1400,210,3),(-520,1430,240,4),(-1280,920,200,3)]):
        for j in range(levels):
            size=width*(1-j*.19)
            px=x+math.sin(i*2+j)*j*11;pz=z+math.cos(i*3+j*.8)*j*9
            b.box((px,430+(j+.5)*55,pz),(size,55,size*.79),b.ROCK_DARK if j==0 else b.STONE if j%3==1 else b.ROCK,0,True)

def build_walls():
    global WALL
    step=50;WALL={}
    for ix in range(-33,33):
        for iz in range(-33,33):
            x=(ix+.5)*step;z=(iz+.5)*step
            if abs(x)-25<250:continue
            closest=rect_distance(x,z,25);farthest=math.hypot(abs(x)+25,abs(z)+25)
            if closest<1500 or farthest>1650:continue
            WALL[ix,iz]=(-600,400)
    voxels={}
    layers=4 if b.LOD else 10
    layer_height=1000/layers
    for level in range(layers):
        floor=-600+level*layer_height
        layer={}
        for (ix,iz) in WALL:
            x=(ix+.5)*step;z=(iz+.5)*step;a=math.atan2(z,x)
            # Keep >=128 m of annular stone before 50 m voxel erosion. A
            # thinner layer can leave diagonal pinholes between grid cells.
            inner=1500+max(0,6+7*math.sin(a*5+level*.81)+4*math.cos(a*11-level*.47))
            outer=1650-max(0,2+3*math.cos(a*7-level*.57))
            if rect_distance(x,z,25)<inner or math.hypot(abs(x)+25,abs(z)+25)>outer:continue
            voxels[ix,level,iz]=True;layer[ix,iz]=(floor,floor+layer_height)
        merge_colliders(layer,step)
    long_faces={}
    for ix,level,iz in voxels:
        x=(ix+.5)*step;z=(iz+.5)*step;y=-600+(level+.5)*layer_height
        for dx,dy,dz in [(-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)]:
            if (ix+dx,level+dy,iz+dz) in voxels:continue
            if b.LOD and not dy:
                long_faces.setdefault((dx,dz,ix if dx else iz,level),set()).add(iz if dx else ix)
                continue
            if dx:
                pts=[(x+dx*25,y-layer_height/2,z-25),(x+dx*25,y+layer_height/2,z-25),(x+dx*25,y+layer_height/2,z+25),(x+dx*25,y-layer_height/2,z+25)]
                if dx<0:pts.reverse()
            elif dz:
                pts=[(x-25,y-layer_height/2,z+dz*25),(x+25,y-layer_height/2,z+dz*25),(x+25,y+layer_height/2,z+dz*25),(x-25,y+layer_height/2,z+dz*25)]
                if dz<0:pts.reverse()
            else:
                pts=[(x-25,y+dy*layer_height/2,z-25),(x-25,y+dy*layer_height/2,z+25),(x+25,y+dy*layer_height/2,z+25),(x+25,y+dy*layer_height/2,z-25)]
                if dy<0:pts.reverse()
            mat=b.STONE if level in [2,6] else b.ROCK_DARK if level in [0,4,8] else b.ROCK
            quad(mat,pts)
            if not dy and not b.LOD:
                local=random.Random(ix*9359+iz*7727+level*373)
                if local.random()>.42:
                    along=local.uniform(-12,12);depth=local.uniform(8,22)
                    px=x+dx*(25+depth*.22)+dz*along;pz=z+dz*(25+depth*.22)+dx*along
                    width=local.uniform(24,49);height=local.uniform(35,76)
                    if abs(px)-width/2<250 and abs(pz)>1200:continue
                    b.box((px,y+local.uniform(-12,12),pz),(depth,height,width) if dx else (width,height,depth),mat)
                    if level in [2,5,8] and (ix+iz)%4==0:
                        b.box((px+dx*depth*.55,y+height*.35,pz+dz*depth*.55),(4,18,width*.72) if dx else (width*.72,18,4),b.LEAF)
    # Merge coplanar runs losslessly. This retains all four offset geological
    # layers in the distant silhouette, inside the 12k triangle budget.
    for (dx,dz,fixed,level),remaining in long_faces.items():
        low=-600+level*layer_height;high=low+layer_height
        mat=b.STONE if level in [2,6] else b.ROCK_DARK if level in [0,4,8] else b.ROCK
        while remaining:
            start=min(remaining);end=start
            while end+1 in remaining:end+=1
            remaining.difference_update(range(start,end+1))
            a=start*step;c=(end+1)*step
            plane=(fixed+.5)*step+(dx or dz)*step/2
            if dx:
                pts=[(plane,low,a),(plane,high,a),(plane,high,c),(plane,low,c)]
                if dx<0:pts.reverse()
            else:
                pts=[(a,low,plane),(c,low,plane),(c,high,plane),(a,high,plane)]
                if dz<0:pts.reverse()
            quad(mat,pts)

def stalactites():
    count=64 if b.LOD else 160
    for i in range(count):
        local=random.Random(78019+i*937)
        a=i*math.tau/count+.013*math.sin(i*3)
        r=local.uniform(1450,1510);x=math.cos(a)*r;z=math.sin(a)*r
        radius=local.uniform(15,34)
        if abs(x)-radius<255:continue
        length=local.uniform(45,125)
        top=370;segments=2 if b.LOD else 4
        for j in range(segments):
            t=j/segments;tt=(j+1)/segments
            p=(x+math.sin(t*3+i)*8*t,top-length*t,z+math.cos(t*4+i)*6*t)
            q=(x+math.sin(tt*3+i)*8*tt,top-length*tt,z+math.cos(tt*4+i)*6*tt)
            rr=radius*(1-t)**1.6;rr2=max(1,radius*(1-tt)**1.6)
            b.beam(p,q,rr,b.ROCK_DARK if j%3==0 else b.ROCK,4 if b.LOD else 7,r2=rr2)
            if not b.LOD:
                # Conservative inscribed collider stays wholly inside each
                # visible tapered segment, with no invisible square envelope.
                inscribed=max(.7,rr2*.72)
                b.collision(tuple((p[k]+q[k])/2 for k in range(3)),(inscribed*2,abs(p[1]-q[1])*.82,inscribed*2))
        if not b.LOD and i%3==0:
            for j in range(4):
                b.box((x+radius*.68,top-j*11,z+radius*.24),(radius*.45,15,radius*.36),b.LEAF if j%2 else b.LEAF_LIGHT)
    # Monolithic buttress ribs stay outside R1400. They read through the
    # regional haze and break the repetitive round wall contour.
    for i in range(20):
        a=i*math.tau/20+.08;x=math.cos(a)*1540;z=math.sin(a)*1540
        if abs(x)<320:continue
        for j in range(7):
            y=-531+j*134;w=54+12*math.sin(i+j*.9)
            px=x+math.sin(a)*j*2.3;pz=z+math.cos(a)*j*1.6
            b.box((px,y,pz),(w,137,w*.86),b.ROCK_DARK if j%3==0 else b.STONE,0,True)

def validate():
    for c in b.COLLISIONS:
        x,y,z=c['center'];sx,sy,sz=c['size']
        if y-sy/2<300:
            # Interior and entry corridors must be physically empty.
            assert math.hypot(max(0,abs(x)-sx/2),max(0,abs(z)-sz/2))>1400,c
            assert abs(x)-sx/2>=250,c
        if y+sy/2>349:
            for cx in [-700,700]:
                assert math.hypot(max(0,abs(x-cx)-sx/2),max(0,abs(z)-sz/2))>=200,c

def review(objects,name,view):
    scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=40;scene.cycles.use_denoising=True
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='CUDA';prefs.get_devices()
        for device in prefs.devices:device.use=device.type=='CUDA'
        if any(d.type=='CUDA' for d in prefs.devices):scene.cycles.device='GPU'
    except Exception:pass
    scene.render.resolution_x=1672;scene.render.resolution_y=941;scene.render.resolution_percentage=100
    scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.20,.29,.43,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.34
    scene.view_settings.view_transform='AgX';created=[]
    for pos,power,color,size in [((-400,100,600),6000000,(.55,.67,1),800),((500,-100,-550),6500000,(1,.59,.30),900),((-1100,100,-800),3000000,(.45,.55,1),600)]:
        data=bpy.data.lights.new('Cave inspection fill','AREA');data.energy=power;data.color=color;data.shape='DISK';data.size=size
        obj=bpy.data.objects.new(data.name,data);bpy.context.collection.objects.link(obj);obj.location=b.vec(pos);obj.rotation_euler=(b.vec((0,300,-300))-obj.location).to_track_quat('-Z','Y').to_euler();created.append(obj)
    data=bpy.data.lights.new('Inspection sun','SUN');data.energy=2;data.angle=.12
    sun=bpy.data.objects.new(data.name,data);bpy.context.collection.objects.link(sun);sun.rotation_euler=(.5,-.35,-.6);created.append(sun)
    camera_data=bpy.data.cameras.new('Vault review');camera=bpy.data.objects.new('Vault review',camera_data);bpy.context.collection.objects.link(camera);created.append(camera)
    camera_data.clip_start=1;camera_data.clip_end=15000
    if view=='interior':
        camera.location=b.vec((130,30,1100));target=b.vec((-300,125,-750));camera_data.type='PERSP';camera_data.lens=21
    elif view=='under':
        camera.location=b.vec((2300,-1250,2350));target=b.vec((0,0,0));camera_data.type='ORTHO';camera_data.ortho_scale=4900
    else:
        camera.location=b.vec((2200,2250,2300));target=b.vec((0,-60,0));camera_data.type='ORTHO';camera_data.ortho_scale=4900
    camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler();scene.camera=camera
    if camera_data.type=='ORTHO':
        rotation=camera.rotation_euler.to_quaternion();inverse=rotation.inverted()
        points=[inverse@(obj.matrix_world@v.co-target) for obj in objects for v in obj.data.vertices]
        minimum=[min(p[j] for p in points) for j in range(2)];maximum=[max(p[j] for p in points) for j in range(2)]
        camera.location+=rotation@Vector(((minimum[0]+maximum[0])/2,(minimum[1]+maximum[1])/2,0))
        camera_data.ortho_scale=max((maximum[0]-minimum[0])*1.08,(maximum[1]-minimum[1])*1672/941*1.08)
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(b.PREVIEW/f'world-art-vault-{name}-{view}.png')
    bpy.ops.render.render(write_still=True)
    for obj in created:bpy.data.objects.remove(obj,do_unlink=True)

def main():
    manifest={};collisions=[]
    for low in [False,True]:
        b.clear();b.LOD=low;b.BATCH={};b.COLLISIONS=[];palette()
        build_roof();build_walls();stalactites();build_uplands();validate()
        objects=b.make_objects()
        if not low:
            collisions=b.COLLISIONS
            bpy.ops.wm.save_as_mainfile(filepath=str(b.SOURCE/'arch-vault.blend'))
        label='lod' if low else 'high'
        result=b.write_glb(OUT/('arch-vault'+('-lod' if low else '')+'.glb'),objects,'arch-vault',low)
        print('VAULT',label,result,'colliders',len(b.COLLISIONS),flush=True)
        assert result['triangles']<=(12000 if low else 100000),result
        manifest[label]=result
        if '--no-render' not in sys.argv:
            b.clear();bpy.ops.import_scene.gltf(filepath=str(OUT/('arch-vault'+('-lod' if low else '')+'.glb')))
            objects=[obj for obj in bpy.context.scene.objects if obj.type=='MESH']
            review(objects,label,'interior')
            if not low:review(objects,label,'under')
            review(objects,label,'top')
    manifest.update({'generator_sha256':hashlib.sha256(HERE.read_bytes()).hexdigest(),'base_generator_sha256':hashlib.sha256(BASE.read_bytes()).hexdigest(),'colliders':len(collisions),'coordinate_system':'Y-up metres','floor':False,'roof_y':[350,450],'outer_massif_y':[418,650],'wall_y':[-600,400],'entries':[{'axis':'Z','minimum_width':500}],'skylights':[{'center':[x,400,0],'minimum_diameter':400} for x in [-700,700]],'navigation_clear_radius':1400,'navigation_clear_below_y':300})
    (OUT/'vault-collisions.json').write_text(json.dumps(collisions,indent=2),encoding='utf8')
    (OUT/'vault-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf8')
    print('VAULT COMPLETE',flush=True)

main()
