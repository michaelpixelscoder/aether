"""CPU-only inspection of authored liquid through the R53 WGSL math.
Actual KTX mip pixels, sRGB decode, bilinear/trilinear repeat, straight alpha.
Swatches are material probes, not engine screenshots or art/performance scores.
"""
import os
for thread_var in ('OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS','NUMEXPR_NUM_THREADS'):
    os.environ[thread_var]='4'
import numpy as np,json,hashlib,zlib,struct
from pathlib import Path
from build_mips import read_png,decode_srgb
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
FILE=AUTHOR/'source/world-flow-native.png'
native=read_png(FILE)
raw=(ROOT/'assets/textures/world-flow.ktx2').read_bytes()
levels=[]
for i in range(11):
    off,n,_=struct.unpack_from('<3Q',raw,80+i*24)
    size=max(1,1254//2**i)
    a=np.frombuffer(raw[off:off+n],dtype=np.uint8).reshape(size,size,4).astype(np.float64)/255
    a[...,:3]=decode_srgb(a[...,:3]);levels.append(a)

def fract(x):return x-np.floor(x)
def smooth(a,b,x):
    f=np.clip((x-a)/(b-a),0,1);return f*f*(3-2*f)
def bilinear(u,v,level):
    rgba=levels[level];height,width,_=rgba.shape
    p=fract(u)*width-.5;q=fract(v)*height-.5
    x=np.floor(p).astype(np.int32);y=np.floor(q).astype(np.int32);fx=p-x;fy=q-y
    a=rgba[y%height,x%width];b=rgba[y%height,(x+1)%width]
    c=rgba[(y+1)%height,x%width];d=rgba[(y+1)%height,(x+1)%width]
    return (a*(1-fx[...,None])+b*fx[...,None])*(1-fy[...,None])+(c*(1-fx[...,None])+d*fx[...,None])*fy[...,None]
def sample(u,v,lod):
    lower=int(np.clip(np.floor(lod),0,10));upper=min(10,lower+1);f=np.clip(lod-lower,0,1)
    return bilinear(u,v,lower)*(1-f)+bilinear(u,v,upper)*f
def liquid(u,v,lod=0):
    phase=fract(u);triangle=1-np.abs(phase*2-1);weight=triangle*triangle*(3-2*triangle)
    a=sample(phase,v,lod);b=sample(fract(phase+.5),v,lod)
    alpha=a[...,3]*weight+b[...,3]*(1-weight)
    premult=a[...,:3]*a[...,3,None]*weight[...,None]+b[...,:3]*b[...,3,None]*(1-weight[...,None])
    return np.concatenate((premult/np.maximum(alpha[...,None],.00001),alpha[...,None]),axis=-1)
def material(u,v,t,lod,fall=False):
    if fall:
        sway=np.sin(v*7-t*.30)*.004+np.sin(v*15-t*.45)*.002
        tex=liquid(v*1.10-t*.20,u*.24+.37+sway,lod)
        ridge=np.maximum(tex[...,1]-tex[...,2]*.28,tex[...,0]*.85);crest=smooth(.008,.16,ridge)
        opacity=.055+.78*crest
        hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
        rgb=tex[...,:3]*.65+hue*ridge[...,None]*4.5/opacity[...,None]
        edge=smooth(.015,.075,u)*smooth(.015,.075,1-u)
        end=1-smooth(.68+tex[...,1]*.12,.93+tex[...,1]*.05,v)
        alpha=.8*edge*tex[...,3]*smooth(0,.025,v)*end*opacity
    else:
        drift=np.sin(u*.37-t*.12)*.006+np.sin(u*.83+t*.09)*.003
        tex=liquid(u*.20-t*.10,v*.82+.09+drift,lod)
        energy=tex[...,:3]@np.array([.08,.62,.30]);body=smooth(.018,.10,energy)
        ridge=np.maximum(tex[...,1]-tex[...,2]*.28,tex[...,0]*.85);crest=smooth(.008,.16,ridge)
        opacity=.36+.10*body+.46*crest
        hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
        rgb=tex[...,:3]*.70+hue*ridge[...,None]*4.0/opacity[...,None]
        edge=smooth(.015,.095,v)*smooth(.015,.095,1-v)
        alpha=.9*edge*tex[...,3]*opacity
    return rgb,alpha,crest

def save(name,rgb):
    pixels=np.round(np.clip(rgb,0,1)*255).astype(np.uint8);height,width,_=pixels.shape
    def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
    scan=b''.join(b'\0'+row.tobytes() for row in pixels)
    data=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(scan,6))+chunk(b'IEND',b'')
    output=AUTHOR/'review'/name
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_bytes(data)
def display(rgb):
    return np.power(np.clip(rgb/(1+rgb),0,1),1/2.2)

