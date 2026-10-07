"""Deep masonry window shells. Presentation only; original collision calls kept.

Install onto a loaded build_world module, then compose only Dawn/Watch HD.
The original house still builds roofs, footings, cornices and corner stones;
only its solid visual cuboid and applied window decals are replaced.
"""
import math

def install(b):
    original_house=b.house;metrics=[]
    def triangles():return sum(sum(len(f)-2 for f in faces) for vertices,faces,uvs in b.BATCH.values())
    def house(x,y,z,w=12,h=12,d=10,roof=True):
        if b.LOD or b.CURRENT not in ['dawn','dawn-watch']:return original_house(x,y,z,w,h,d,roof)
        ground=max(b.ground_level(x+dx,z+dz) for dx in [-w/2,0,w/2] for dz in [-d/2,0,d/2]) if y==0 else y
        old_box,old_windows,old_bay=b.box,b.windows,b.facade_bay
        before=triangles();removed=0;body_count=0
        def keep_physics(p,size,mat,bevel=0,solid=False):
            nonlocal body_count
            if solid and mat==b.STONE and tuple(size)==(w,h,d) and abs(p[1]-(ground+h/2))<1e-6:
                b.collision(p,size);body_count+=1;return
            return old_box(p,size,mat,bevel,solid)
        # Measure removed local window geometry without changing RNG or output.
        old_geom=b.geom
        def measure_window(call,*args,**kwargs):
            nonlocal removed
            def count(mat,points,faces,bevel=0):
                nonlocal removed
                removed+=sum(len(f)-2 for f in faces)
            b.geom=count
            try:call(*args,**kwargs)
            finally:b.geom=old_geom
        b.box=keep_physics
        b.windows=lambda *args,**kwargs:measure_window(old_windows,*args,**kwargs)
        b.facade_bay=lambda *args,**kwargs:measure_window(old_bay,*args,**kwargs)
        try:original_house(x,y,z,w,h,d,roof)
        finally:b.box,b.windows,b.facade_bay=old_box,old_windows,old_bay
        assert body_count==1,(x,y,z,w,h,d,body_count)
        fixed=triangles();depth=min(1.2,w*.105,d*.105)
        # Inner core stays inside the exact former solid envelope. Recesses do
        # not create enterable rooms; dark glass closes every opening.
        b.box((x,ground+h/2,z),(w-2*depth,h,d-2*depth),b.STONE)
        face_records=[]
        def facade(cx,cz,length,angle,centres,window_width):
            c,s=math.cos(angle),math.sin(angle)
            def p(u,v,q=0):return (cx+u*c+q*s,ground+v,cz-u*s+q*c)
            def quad(u0,u1,y0,y1,q=0,mat=None):
                if u1-u0<1e-7 or y1-y0<1e-7:return
                b.geom(mat or b.STONE,[p(u0,y0,q),p(u1,y0,q),p(u1,y1,q),p(u0,y1,q)],[(0,1,2,3)])
            def block(u,v,q,size,mat):b.yaw_box(p(u,v,q),size,angle,mat)
            bottom=h*.33;top=h*.79;radius=window_width/2;spring=top-radius
            assert bottom>2.5 and spring>bottom
            left=-length/2
            for index,centre in enumerate(centres):
                lo=centre-radius;hi=centre+radius
                quad(left,lo,0,h);quad(lo,hi,0,bottom)
                arc=[(centre+radius*math.cos(math.pi-i*math.pi/6),spring+radius*math.sin(math.pi-i*math.pi/6)) for i in range(7)]
                for (u0,v0),(u1,v1) in zip(arc,arc[1:]):
                    b.geom(b.STONE,[p(u0,v0),p(u1,v1),p(u1,h),p(u0,h)],[(0,1,2,3)])
                # Reveal is one continuous return from front stone to real
                # glazing behind it. Its geometry catches the low key light.
                perimeter=[(lo,bottom),(hi,bottom),(hi,spring)]+list(reversed(arc))[1:]+[(lo,bottom)]
                for a,bb in zip(perimeter,perimeter[1:]):
                    b.geom(b.STONE,[p(*a),p(*bb),p(*bb,-depth),p(*a,-depth)],[(0,1,2,3)])
                back=[p(lo,bottom,-depth+.015),p(hi,bottom,-depth+.015)]+[p(u,v,-depth+.015) for u,v in reversed(arc)]
                b.geom(b.ROOF,back,[tuple(range(len(back)))])
                # Paired glass lights preserve the previous subdued amber
                # surface area; the whole deep recess is not emissive.
                glass_width=window_width*.215
                for side in [-1,1]:quad(centre+side*window_width*.14-glass_width/2,centre+side*window_width*.14+glass_width/2,bottom+.32,spring+.10,-depth+.034,b.GLOW)
                quad(centre-.055,centre+.055,bottom+.20,spring+.22,-depth+.047,b.GOLD)
                quad(lo+.12,hi-.12,bottom+(spring-bottom)*.53,bottom+(spring-bottom)*.53+.09,-depth+.049,b.GOLD)
                # Six broad radial voussoirs. Inset backs, chipped shoulder
                # bevels and separated radial joints make the crown read as
                # stone rather than an attached bright rectangular outline.
                outer=radius+.53
                for seg in range(6):
                    a=seg*math.pi/6+.012;bb=(seg+1)*math.pi/6-.012
                    outline=[(centre+radius*math.cos(a),spring+radius*math.sin(a)),(centre+outer*math.cos(a),spring+outer*math.sin(a)),(centre+outer*math.cos(bb),spring+outer*math.sin(bb)),(centre+radius*math.cos(bb),spring+radius*math.sin(bb))]
                    front=.19+(.085 if seg in [2,3] else 0)
                    points=[p(u,v,q) for q in [-.025,front] for u,v in outline]
                    b.geom(b.TRIM if seg in [2,3] else b.STONE,points,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)])
                jamb_height=spring-bottom
                for side in [-1,1]:
                    block(centre+side*(radius+.22),bottom+jamb_height/2,.105,(.42,jamb_height,.26),b.STONE)
                block(centre,bottom-.12,.13,(window_width+1.0,.24,.52),b.TRIM)
                left=hi
                face_records.append({'centre':[cx+centre*c,ground+(bottom+top)/2,cz-centre*s],'normal':[s,0,c],'width':window_width,'bottom':ground+bottom,'top':ground+top,'recess_m':depth,'max_projection_m':.39})
            quad(left,length/2,0,h)
        facade(x,z+d/2,w,0,[-w*.28,0,w*.28],min(2.8,w*.16))
        facade(x,z-d/2,w,math.pi,[-w*.28,0,w*.28],min(2.8,w*.16))
        facade(x+w/2,z,d,math.pi/2,[-d*.25,d*.25],min(2.8,d*.18))
        facade(x-w/2,z,d,-math.pi/2,[-d*.25,d*.25],min(2.8,d*.18))
        # Top/bottom are unchanged solid silhouettes; the roof remains authored
        # by original_house and hides the original top cap as before.
        b.geom(b.STONE,[(x-w/2,ground+h,z-d/2),(x-w/2,ground+h,z+d/2),(x+w/2,ground+h,z+d/2),(x+w/2,ground+h,z-d/2)],[(0,1,2,3)])
        added=triangles()-fixed
        metrics.append({'centre':[x,ground,z],'dimensions':[w,h,d],'old_window_and_body_raw_triangles':removed+12,'new_shell_raw_triangles':added,'raw_net_triangles':added-removed-12,'windows':face_records,'core_dimensions':[w-2*depth,h,d-2*depth]})
    b.house=house
    return metrics
