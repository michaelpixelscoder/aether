"""Derive standard sRGB KTX2 mips from untouched ImageGen RGBA8 PNG pixels.

Level zero is exact. Every following level averages linear premultiplied RGB and
linear alpha over its footprint, then unpremultiplies and encodes sRGB. No alpha
coverage correction, authored-pixel edit, image resizing at level zero or GPU.
"""
import os
for thread_var in ('OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS','NUMEXPR_NUM_THREADS'):
    os.environ[thread_var]='4'
import hashlib,json,struct,zlib,time
from pathlib import Path
import numpy as np
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
SOURCE=AUTHOR/'source/world-flow-current-native.png'
OUT=ROOT/'assets/textures/world-flow-current-r64.ktx2'

def read_png(path):
    raw=path.read_bytes();assert raw[:8]==b'\x89PNG\r\n\x1a\n'
    cursor=8;payload=b'';dims=None
    while cursor<len(raw):
        size=struct.unpack_from('>I',raw,cursor)[0];kind=raw[cursor+4:cursor+8];chunk=raw[cursor+8:cursor+8+size]
        assert zlib.crc32(kind+chunk)&0xffffffff==struct.unpack_from('>I',raw,cursor+8+size)[0]
        if kind==b'IHDR':
            width,height,depth,color,compression,filtering,interlace=struct.unpack('>IIBBBBB',chunk)
            assert [depth,color,compression,filtering,interlace]==[8,6,0,0,0]
            dims=(height,width)
        if kind==b'IDAT':payload+=chunk
        cursor+=size+12
        if kind==b'IEND':break
    assert cursor==len(raw) and dims
    height,width=dims;scan=zlib.decompress(payload);stride=width*4
    assert len(scan)==height*(stride+1)
    decoded=np.empty((height,stride),dtype=np.uint8);previous=bytearray(stride)
    for y in range(height):
        at=y*(stride+1);mode=scan[at];row=bytearray(scan[at+1:at+1+stride]);assert mode<=4
        for i in range(stride):
            left=row[i-4] if i>=4 else 0;up=previous[i];corner=previous[i-4] if i>=4 else 0
            if mode==1:value=left
            elif mode==2:value=up
            elif mode==3:value=(left+up)//2
            elif mode==4:
                p=left+up-corner;pa=abs(p-left);pb=abs(p-up);pc=abs(p-corner)
                value=left if pa<=pb and pa<=pc else up if pb<=pc else corner
            else:value=0
            row[i]=(row[i]+value)&255
        decoded[y]=np.frombuffer(row,dtype=np.uint8);previous=row
    return decoded.reshape((height,width,4))

def decode_srgb(x):return np.where(x<=.04045,x/12.92,((x+.055)/1.055)**2.4)
def encode_srgb(x):return np.where(x<=.0031308,x*12.92,1.055*np.maximum(x,0)**(1/2.4)-.055)
def area_axis(a,axis):
    a=np.moveaxis(a,axis,0);n=len(a);m=max(1,n//2);scale=n/m
    out=np.empty((m,)+a.shape[1:],dtype=np.float64)
    for i in range(m):
        start,end=i*scale,(i+1)*scale;first=int(np.floor(start));last=min(n,int(np.ceil(end)))
        weights=np.maximum(0,np.minimum(np.arange(first,last)+1,end)-np.maximum(np.arange(first,last),start))/scale
        out[i]=np.sum(a[first:last]*weights.reshape((-1,)+(1,)*(a.ndim-1)),axis=0)
    return np.moveaxis(out,0,axis)
def downsample(pixels):
    rgba=pixels.astype(np.float64)/255
    alpha=rgba[...,3:];linear=decode_srgb(rgba[...,:3])
    associated=np.concatenate((linear*alpha,alpha),axis=-1)
    averaged=area_axis(area_axis(associated,1),0)
    straight=np.divide(averaged[...,:3],averaged[...,3:],out=np.zeros_like(averaged[...,:3]),where=averaged[...,3:]>0)
    out=np.concatenate((encode_srgb(straight),averaged[...,3:]),axis=-1)
    return np.round(np.clip(out,0,1)*255).astype(np.uint8)

def tests():
    constant=np.broadcast_to(np.array([73,141,229,193],dtype=np.uint8),(7,9,4)).copy()
    current=constant
    while current.shape[0]>1 or current.shape[1]>1:
        current=downsample(current);assert np.all(current==constant[0,0])
    poison=np.array([[[255,0,0,255],[0,255,255,0]],[[0,255,255,0],[0,255,255,0]]],dtype=np.uint8)
    assert np.array_equal(downsample(poison)[0,0],[255,0,0,64])
    poison[...,3]=0;assert np.all(downsample(poison)==0)
    bw=np.array([[[0,0,0,255],[255,255,255,255]],[[0,0,0,255],[255,255,255,255]]],dtype=np.uint8)
    assert np.array_equal(downsample(bw)[0,0],[188,188,188,255])
    return ['constant RGBA through odd-sized chain','transparent cyan cannot contaminate opaque red','fully transparent color discarded in derived levels','linear-light black/white mean encodes as sRGB188']

def write_ktx(levels):
    block=struct.pack('<IHH4B4B8B',0,2,88,1,1,2,0,0,0,0,0,4,0,0,0,0,0,0,0)
    # DFD RGBSDA / BT709 / sRGB / STRAIGHT_ALPHA. Alpha sample has LINEAR bit.
    for c,channel in enumerate((0,1,2,31)):
        block+=struct.pack('<HBB4BII',c*8,7,channel,0,0,0,0,0,255)
    dfd=struct.pack('<I',len(block)+4)+block
    metadata=b''
    for key,value in [('KTXorientation','rd'),('KTXwriter','Aether native ImageGen / linear alpha-weighted mip derivation')]:
        pair=key.encode()+b'\0'+value.encode()+b'\0'
        metadata+=struct.pack('<I',len(pair))+pair+b'\0'*(-len(pair)%4)
    dfd_offset=80+24*len(levels);kvd_offset=dfd_offset+len(dfd);start=(kvd_offset+len(metadata)+7)//8*8
    offsets={};payload=bytearray()
    for i in reversed(range(len(levels))):
        payload+=b'\0'*((-(start+len(payload)))%8);offsets[i]=start+len(payload);payload+=levels[i].tobytes()
    height,width,_=levels[0].shape
    header=b'\xABKTX 20\xBB\r\n\x1A\n'+struct.pack('<13I2Q',43,1,width,height,0,0,1,len(levels),0,dfd_offset,len(dfd),kvd_offset,len(metadata),0,0)
    indices=b''.join(struct.pack('<3Q',offsets[i],a.nbytes,a.nbytes) for i,a in enumerate(levels))
    OUT.write_bytes(header+indices+dfd+metadata+b'\0'*(start-kvd_offset-len(metadata))+payload)
    return offsets

def main():
    started=time.perf_counter();passed=tests();native=read_png(SOURCE)
    OUT.parent.mkdir(parents=True,exist_ok=True)
    (AUTHOR/'proofs').mkdir(parents=True,exist_ok=True)
    authentication=json.loads((AUTHOR/'source/native-r64.json').read_text(encoding='utf-8'))
    assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()==authentication['sha256'],'Authenticated new original PNG'
    assert list(native.shape)==[authentication['height'],authentication['width'],4],'Actual native dimensions, no resize'
    height,width,_=native.shape
    assert min(width,height)>=64,'Production native texture, not a synthetic low-resolution substitute'
    levels=[native]
    while levels[-1].shape[0]>1 or levels[-1].shape[1]>1:levels.append(downsample(levels[-1]))
    offsets=write_ktx(levels);raw=OUT.read_bytes()
    assert struct.unpack_from('<9I',raw,12)==(43,1,width,height,0,0,1,len(levels),0)
    items=[]
    for i,a in enumerate(levels):
        offset,length,unpacked=struct.unpack_from('<3Q',raw,80+i*24)
        assert offset==offsets[i] and offset%8==0 and length==unpacked==a.nbytes
        payload=raw[offset:offset+length];assert payload==a.tobytes()
        if i==0:assert payload==native.tobytes()
        items.append({'level':i,'dimensions':[a.shape[1],a.shape[0]],'gpu_bytes':a.nbytes,'sha256':hashlib.sha256(payload).hexdigest(),'offset':offset})
    report={'schema':1,'native_png_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'derived_ktx2_sha256':hashlib.sha256(raw).hexdigest(),
            'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'format':'Rgba8UnormSrgb / VK43','alpha':'straight; linear alpha sample flagged in DFD',
            'method':'Iterated box-area average over complete source footprint: linear RGB weighted by alpha, alpha average, unpremultiply, sRGB encode and round to nearest RGBA8. NPOT floor-halving; no coverage MASK or alpha rescale.',
            'native_dimensions':[width,height],'mip_count':len(levels),'native_level0_decoded_rgba_byte_exact':True,'native_rgba_sha256':hashlib.sha256(native.tobytes()).hexdigest(),
            'gpu_bytes':sum(a.nbytes for a in levels),'file_bytes':len(raw),'levels':items,'tests_passed':passed,'seconds':time.perf_counter()-started}
    (AUTHOR/'proofs/mip.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report),flush=True)
if __name__=='__main__':main()
