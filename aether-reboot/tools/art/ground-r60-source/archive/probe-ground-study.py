"""Independent PNG albedo/color/UV analysis. No bitmap outputs or edits."""
import os
for key in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']:os.environ[key]='4'
import numpy as np
from pathlib import Path
import json,struct,zlib,hashlib
from patch_ground import AUTHOR,parse,target_material,SOURCE_SHA,UV_SCALE,FILES

def read_png(path):
    raw=path.read_bytes();assert raw[:8]==b'\x89PNG\r\n\x1a\n'
    cursor=8;data=[];size=None
    while cursor<len(raw):
        n=struct.unpack_from('>I',raw,cursor)[0];kind=raw[cursor+4:cursor+8];payload=raw[cursor+8:cursor+8+n]
        assert zlib.crc32(kind+payload)&0xffffffff==struct.unpack_from('>I',raw,cursor+8+n)[0]
        if kind==b'IHDR':
            w,h,depth,color,compression,filtering,interlace=struct.unpack('>IIBBBBB',payload)
            assert depth==8 and color in [2,6] and compression==filtering==interlace==0
            size=(w,h,3 if color==2 else 4)
        if kind==b'IDAT':data.append(payload)
        cursor+=n+12
    assert cursor==len(raw) and size
    w,h,channels=size;scan=zlib.decompress(b''.join(data));stride=w*channels
    assert len(scan)==h*(stride+1)
    decoded=np.empty((h,stride),dtype=np.uint8);previous=np.zeros(stride,dtype=np.uint8)
    for y in range(h):
        at=y*(stride+1);mode=scan[at];assert mode<=4
        row=bytearray(scan[at+1:at+1+stride])
        for x in range(stride):
            left=row[x-channels] if x>=channels else 0;up=int(previous[x]);corner=int(previous[x-channels]) if x>=channels else 0
            if mode==0:add=0
            elif mode==1:add=left
            elif mode==2:add=up
            elif mode==3:add=(left+up)//2
            else:
                predictor=left+up-corner;a=abs(predictor-left);b=abs(predictor-up);c=abs(predictor-corner)
                add=left if a<=b and a<=c else up if b<=c else corner
            row[x]=(row[x]+add)&255
        decoded[y]=np.frombuffer(row,dtype=np.uint8);previous=decoded[y]
    return decoded.reshape(h,w,channels)

def linear(rgb):
    value=rgb.astype(np.float64)/255
    return np.where(value<=.04045,value/12.92,((value+.055)/1.055)**2.4)

def albedo(pixels):
    rgb=pixels[...,:3];decoded=linear(rgb)
    luminance=decoded@np.array([.2126,.7152,.0722])
    flat=rgb.reshape(-1,3)
    return {'dimensions':[pixels.shape[1],pixels.shape[0]],'channels':pixels.shape[2],
        'srgb_mean_8bit':rgb.mean(axis=(0,1)).tolist(),'linear_mean_rgb':decoded.mean(axis=(0,1)).tolist(),
        'linear_luma_mean':float(luminance.mean()),'linear_luma_p05_p50_p95':np.quantile(luminance,[.05,.5,.95]).tolist(),
        'opaque_alpha':bool(pixels.shape[2]==3 or (pixels[...,3]==255).all()),
        'mean_absolute_horizontal_edge_difference_8bit':float(np.abs(pixels[:,0,:3].astype(np.float64)-pixels[:,-1,:3]).mean()),
        'mean_absolute_vertical_edge_difference_8bit':float(np.abs(pixels[0,:,:3].astype(np.float64)-pixels[-1,:,:3]).mean())}

def accessor(doc,binary,index):
    a=doc['accessors'][index];v=doc['bufferViews'][a['bufferView']]
    dtype={5121:'u1',5123:'<u2',5125:'<u4',5126:'<f4'}[a['componentType']]
    components={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[a['type']]
    width=np.dtype(dtype).itemsize*components;stride=v.get('byteStride',width)
    start=v.get('byteOffset',0)+a.get('byteOffset',0)
    values=np.ndarray((a['count'],components),dtype=dtype,buffer=binary,offset=start,strides=(stride,np.dtype(dtype).itemsize)).astype(np.float64)
    if a.get('normalized'):values/=255 if a['componentType']==5121 else 65535
    return values

native=read_png(AUTHOR/'source/ground-r60-native.png')
old=read_png(AUTHOR/'history/grass-r56.png')
assert hashlib.sha256((AUTHOR/'source/ground-r60-native.png').read_bytes()).hexdigest()==SOURCE_SHA
assert native.shape[:2]==(1254,1254)
ground=[]
for file in FILES:
    doc,binary=parse((AUTHOR/'history/r56-world'/file).read_bytes())
    target=target_material(doc)
    primitives=[p for m in doc['meshes'] for p in m['primitives'] if p.get('material')==target]
    metric=[];vertex=[];triangles=0
    for primitive in primitives:
        attrs=primitive['attributes'];p=accessor(doc,binary,attrs['POSITION']);uv=accessor(doc,binary,attrs['TEXCOORD_0']);normals=accessor(doc,binary,attrs['NORMAL'])
        indices=accessor(doc,binary,primitive['indices']).astype(np.int64).reshape(-1,3)
        triangles+=len(indices)
        if 'COLOR_0' in attrs:vertex.append({'accessor':attrs['COLOR_0'],'mean':accessor(doc,binary,attrs['COLOR_0']).mean(axis=0).tolist()})
        for corners in indices:
            if np.mean(normals[corners,1])<.95:continue
            xz=p[corners][:,[0,2]];tex=uv[corners]
            basis=xz[1:]-xz[0]
            if abs(np.linalg.det(basis))<1e-9:continue
            mapping=np.linalg.solve(basis,tex[1:]-tex[0])
            metric.extend(np.linalg.svd(mapping,compute_uv=False).tolist())
    assert len(metric)>0
    assert np.max(np.abs(np.array(metric)-.11))<1e-5
    material=doc['materials'][target]
    ground.append({'file':file,'material_index':target,'primitive_count':len(primitives),'triangles':triangles,
        'actual_base_color_factor':material['pbrMetallicRoughness'].get('baseColorFactor',[1,1,1,1]),
        'actual_vertex_color_0':vertex if vertex else 'ABSENT from every material12 primitive',
        'measured_uv_singular_values_min_max':[min(metric),max(metric)],'local_tile_m':1/.11,
        'world_tile_m_primary_scale2_1':2.1/.11,'new_world_tile_m_primary':2.1/(.11*UV_SCALE),
        'new_world_tile_m_instance_scale1_5':1.5/(.11*UV_SCALE),
        'roughness_unchanged':material['pbrMetallicRoughness'].get('roughnessFactor')})
report={'schema':1,'passed':True,'pixels_modified':False,'gpu_used':False,'source_png_sha256':SOURCE_SHA,
    'script_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'old_texture':albedo(old),'native_texture':albedo(native),
    'source_palette_turf_linear':[.095,.19,.032],'actual_glb_materials':ground,
    'neutral_factor':[1,1,1,1],'factor_reason':'Actual source pixel values are albedo; glTF linear factor1 preserves them. Factor>1 invalid per Khronos; no yellow palette multiplier or global exposure added.',
    'limits':['PNG prototype has one mip level; distant shimmer/performance need native check','seam metrics are diagnostics, not perfect tileability proof','same GLB material reused at other instance scales has proportional physical tile size','natural albedo is darker than old mosaic; final8K/key lighting review needed']}
(AUTHOR/'proofs/albedo-uv.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
