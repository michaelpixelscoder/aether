"""Continuous cloud fields derived from unchanged original density sculptures."""
import bpy
import gc
import hashlib
import json
import time
from pathlib import Path
import numpy as np
import openvdb
from mathutils import Vector

SOURCE_CACHE={}
SOURCE_RECORDS={}

def union(objects,output,material,voxel=64.):
    started=time.perf_counter()
    for data in {o.data for o in objects}:
        data.grids.load()
    bpy.context.view_layer.update()
    boxes=[]
    for obj in objects:
        corners=np.array([tuple(obj.matrix_world@Vector(p)) for p in obj.bound_box])
        boxes.append((corners.min(axis=0),corners.max(axis=0)))
    lower=np.floor(np.min([b[0] for b in boxes],axis=0)/voxel).astype(np.int32)-3
    upper=np.ceil(np.max([b[1] for b in boxes],axis=0)/voxel).astype(np.int32)+3
    shape=upper-lower+1
    field=np.zeros(tuple(shape),dtype=np.float32)
    target_density=next(n for n in material.node_tree.nodes if n.type=='PRINCIPLED_VOLUME').inputs['Density'].default_value
    for obj,box in zip(objects,boxes):
        path=Path(bpy.path.abspath(obj.data.filepath)).resolve()
        if str(path) not in SOURCE_CACHE:
            grid=openvdb.read(str(path),'density')
            a,b=grid.evalActiveVoxelBoundingBox()
            source=np.empty(tuple(b[i]-a[i]+1 for i in range(3)),dtype=np.float32)
            grid.copyToArray(source,ijk=a)
            assert max(abs(v) for v in grid.transform.worldToIndex((0.,0.,0.)))<1e-6
            SOURCE_CACHE[str(path)]=(source,np.array(a),np.array(grid.transform.voxelSize()))
            SOURCE_RECORDS[str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
            del grid
        source,src_lower,src_voxel=SOURCE_CACHE[str(path)]
        inverse=np.array(obj.matrix_world.inverted(),dtype=np.float64)
        begin=np.maximum(np.floor(box[0]/voxel).astype(np.int32)-1,lower+2)
        end=np.minimum(np.ceil(box[1]/voxel).astype(np.int32)+2,upper-1)
        density=next(n for n in obj.data.materials[0].node_tree.nodes if n.type=='PRINCIPLED_VOLUME').inputs['Density'].default_value/target_density
        ys=np.arange(begin[1],end[1],dtype=np.float32)[None,:,None]*voxel
        zs=np.arange(begin[2],end[2],dtype=np.float32)[None,None,:]*voxel
        for start in range(int(begin[0]),int(end[0]),12):
            stop=min(start+12,int(end[0]))
            xs=np.arange(start,stop,dtype=np.float32)[:,None,None]*voxel
            coords=[((inverse[i,0]*xs+inverse[i,1]*ys+inverse[i,2]*zs+inverse[i,3])/src_voxel[i]-src_lower[i]).astype(np.float32) for i in range(3)]
            floors=[np.floor(c).astype(np.int32) for c in coords]
            fractions=[c-f for c,f in zip(coords,floors)]
            sampled=np.zeros(coords[0].shape,dtype=np.float32)
            for dx in (0,1):
                for dy in (0,1):
                    for dz in (0,1):
                        indices=[floors[i]+delta for i,delta in enumerate((dx,dy,dz))]
                        valid=np.ones(sampled.shape,dtype=bool)
                        for i,index in enumerate(indices):
                            valid&=(index>=0)&(index<source.shape[i])
                        weight=((fractions[0] if dx else 1-fractions[0])*(fractions[1] if dy else 1-fractions[1])*(fractions[2] if dz else 1-fractions[2]))
                        sampled+=source[tuple(np.clip(indices[i],0,source.shape[i]-1) for i in range(3))]*weight*valid
            destination=field[start-lower[0]:stop-lower[0],begin[1]-lower[1]:end[1]-lower[1],begin[2]-lower[2]:end[2]-lower[2]]
            np.maximum(destination,sampled*density,out=destination)
        obj.hide_render=True
    grid=openvdb.FloatGrid()
    grid.name='density'
    grid.gridClass=openvdb.GridClass.FOG_VOLUME
    grid.transform=openvdb.createLinearTransform(voxelSize=voxel)
    grid.copyFromArray(field,ijk=tuple(int(v) for v in lower))
    access=grid.getAccessor()
    for x in (lower[0],upper[0]):
        for y in (lower[1],upper[1]):
            for z in (lower[2],upper[2]):
                access.setValueOn((int(x),int(y),int(z)),0.)
    del access
    grid.saveFloatAsHalf=True
    openvdb.write(str(output),grids=[grid])
    del grid
    stored=openvdb.read(str(output),'density')
    a,b=stored.evalActiveVoxelBoundingBox()
    assert tuple(a)==tuple(lower) and tuple(b)==tuple(upper) and stored.background==0.
    stored.copyToArray(field,ijk=a)
    assert np.isfinite(field).all() and field.min()>=0 and field.max()<=1.
    assert all(np.count_nonzero(face)==0 for face in (field[0],field[-1],field[:,0],field[:,-1],field[:,:,0],field[:,:,-1]))
    data=bpy.data.volumes.new(output.stem)
    data.filepath=bpy.path.relpath(str(output))
    data.materials.append(material)
    result=bpy.data.objects.new(output.stem,data)
    bpy.context.scene.collection.objects.link(result)
    result['continuous_cloud_front']=True
    record={'file':output.name,'sha256':hashlib.sha256(output.read_bytes()).hexdigest(),'bytes':output.stat().st_size,
            'instances_merged':len(objects),'voxel_metres':voxel,'dense_shape':shape.tolist(),'dense_bytes':field.nbytes,
            'density_max':float(field.max()),'stored_zero_halo':True,'stored_lower':list(a),'stored_upper':list(b),
            'seconds':round(time.perf_counter()-started,3)}
    print('CONTINUOUS_FRONT',json.dumps(record),flush=True)
    del stored,field
    gc.collect()
    return record
