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
