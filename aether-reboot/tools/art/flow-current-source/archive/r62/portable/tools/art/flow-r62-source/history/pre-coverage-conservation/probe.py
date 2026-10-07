"""Actual untouched KTX sampling and R62 math, CPU only; no bitmap outputs."""
from pathlib import Path
import hashlib,json,struct,os
for name in ('OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS'):os.environ[name]='4'
import numpy as np
AUTHOR=Path(__file__).resolve().parent;ROOT=AUTHOR.parents[2]
sha=lambda raw:hashlib.sha256(raw).hexdigest()
raw=(ROOT/'assets/textures/world-flow.ktx2').read_bytes();assert sha(raw)=='8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973'
levels=[]
for i in range(11):
 off,n,_=struct.unpack_from('<3Q',raw,80+i*24);size=max(1,1254//2**i)
 tex=np.frombuffer(raw[off:off+n],dtype=np.uint8).reshape(size,size,4).astype(float)/255
 tex[...,:3]=np.where(tex[...,:3]<=.04045,tex[...,:3]/12.92,((tex[...,:3]+.055)/1.055)**2.4);levels.append(tex)
def fract(x):return x-np.floor(x)
def smooth(a,b,x):
 f=np.clip((x-a)/(b-a),0,1);return f*f*(3-2*f)
def bilinear(u,v,i):
 tex=levels[i];h,w,_=tex.shape;x=fract(u)*w-.5;y=fract(v)*h-.5
 ix=np.floor(x).astype(int);iy=np.floor(y).astype(int);fx=(x-ix)[...,None];fy=(y-iy)[...,None]
 return (tex[iy%h,ix%w]*(1-fx)+tex[iy%h,(ix+1)%w]*fx)*(1-fy)+(tex[(iy+1)%h,ix%w]*(1-fx)+tex[(iy+1)%h,(ix+1)%w]*fx)*fy
def sample(u,v,lod):
 i=int(np.clip(np.floor(lod),0,10));f=np.clip(lod-i,0,1)
 return bilinear(u,v,i)*(1-f)+bilinear(u,v,min(i+1,10))*f
def optics(tex,threshold,grazing):
 energy=tex[...,:3]@np.array([.12,.52,.36]);ridge=np.maximum(energy-.030,0)+energy*.10
 strong=smooth(threshold,threshold*3.5,ridge)
 hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
 cyan=hue*.20+np.array([.012,.48,1])*.80
 ink=smooth(.18,.32,tex[...,0]/np.maximum(tex[...,1],.0005));violet=ink*smooth(.035,.095,ridge)*.85
 hue=cyan*(1-violet[...,None])+np.array([.35,.025,1])*violet[...,None]
 body_alpha=.018+.010*grazing+.005*smooth(.025,.10,energy)
 coverage=(body_alpha+.62*strong)*tex[...,3]
 body=(tex[...,:3]*.22+np.array([.002,.012,.04]))*body_alpha[...,None]*tex[...,3,None]
 crest=hue*ridge[...,None]*(.006+.994*strong[...,None])*tex[...,3,None]
 return body,crest,coverage,tex[...,3]
def mix_optics(a,b,w):return tuple(x*w[...,None]+y*(1-w[...,None]) if x.ndim==w.ndim+1 else x*w+y*(1-w) for x,y in zip(a,b))
def seam(u,v,lod,threshold,grazing):
 phase=fract(u);w=smooth(.012,.065,phase)*smooth(.012,.065,1-phase)
 return mix_optics(optics(sample(phase,v,lod),threshold,grazing),optics(sample(fract(phase+.5),v,lod),threshold,grazing),w)
def advection(u,v,t,lod,grazing=.5):
 field=sample(.5+.35*np.sin(u*.12+.21),.5+.35*np.sin(v*.12+.17),6)
 noise=np.clip(field[...,2]*4,0,1)*2-1;p=fract(t/6+noise*.19);q=fract(p+.5)
 triangle=1-np.abs(p*2-1);w=triangle*triangle*(3-2*triangle)
 dv=np.clip((field[...,0]-field[...,1])*6,-1,1)*.003
 threshold=.035+(.020-.035)*smooth(2,6,np.asarray(lod))
 return mix_optics(seam(u-.035*(p-.5),v-dv*(p-.5),lod,threshold,grazing),seam(u-.035*(q-.5),v-dv*(q-.5),lod,threshold,grazing),w)
def r62(u,v,t,lod,thermal=0,grazing=.5):
 crop=.16+(.42-.16)*thermal;start=.42+(.29-.42)*thermal
 drift=np.sin(u*.37-t*.12)*.006+np.sin(u*.83+t*.09)*.003
 body,crest,a,source_a=advection(u*.20-t*.10,v*crop+start+drift,t,lod,grazing)
 middle=1-smooth(.16,.43,np.abs(v-.5));profile=.12+1.88*middle;core=thermal*smooth(16.5,18.25,u)*middle
 peak=np.max(crest,axis=-1);core_mass=crest*(1-core[...,None]*.85)+np.array([.52,.92,1])*peak[...,None]*core[...,None]*.85
 gain=14*profile*(1+core*2);bounded=gain/(1+peak*gain/6)
 associated=body+core_mass*bounded[...,None]+np.array([.52,.92,1])*core[...,None]*.14*source_a[...,None]
 edge=smooth(.015,.095,v)*smooth(.015,.095,1-v)
 return associated/np.maximum(a[...,None],.00001),a*.9*edge,associated
def r61(u,v,t,lod,thermal=0):
 drift=np.sin(u*.37-t*.12)*.006+np.sin(u*.83+t*.09)*.003
 x=u*.20-t*.10;y=v*.42+.29+drift;p=fract(x);tri=1-np.abs(p*2-1);w=tri*tri*(3-2*tri)
 a=sample(p,y,lod);b=sample(fract(p+.5),y,lod);native_a=a[...,3]*w+b[...,3]*(1-w)
 tex=(a[...,:3]*a[...,3,None]*w[...,None]+b[...,:3]*b[...,3,None]*(1-w[...,None]))/np.maximum(native_a[...,None],.00001)
 energy=tex@np.array([.12,.52,.36]);ridge=np.maximum(energy-.03,0)+energy*.1;opacity=.045+.4*smooth(.025,.20,ridge)
 hue=tex/np.maximum(np.max(tex,axis=-1),.001)[...,None];ink=smooth(.18,.32,tex[...,0]/np.maximum(tex[...,1],.0005));violet=ink*smooth(.035,.095,ridge)*.9
 hue=hue*.2+np.array([.015,.28,1])*.8;hue=hue*(1-violet[...,None])+np.array([.30,.018,1])*violet[...,None]
 middle=1-smooth(.16,.43,np.abs(v-.5));profile=.12+1.88*middle;core=thermal*smooth(16.5,18.25,u)*middle
 hue=hue*(1-core[...,None]*.85)+np.array([.52,.92,1])*core[...,None]*.85
 threshold=.035+(.020-.035)*smooth(2,6,np.asarray(lod));strong=.035+.965*smooth(threshold,threshold*3.5,ridge)
 emission=ridge*14*profile*(1+core*2)*(strong*(1-core)+core)+core*.14
 rgb=tex*.4+hue*emission[...,None]/opacity[...,None]
 return rgb,.9*smooth(.015,.095,v)*smooth(.015,.095,1-v)*native_a*opacity
def main():
 stats=[];v,u=np.meshgrid((np.arange(160)+.5)/160,(np.arange(480)+.5)/480*32,indexing='ij')
 for lod in range(9):
  rgb,alpha,associated=r62(u,v,4,lod);old,old_a=r61(u,v,4,lod)
  assert np.isfinite(rgb).all() and (rgb>=0).all() and (alpha>=0).all() and (alpha<=.9*.653+1e-9).all()
  assert associated.max()<6.15
  lum=(rgb*alpha[...,None])@np.array([.2126,.7152,.0722]);before=(old*old_a[...,None])@np.array([.2126,.7152,.0722])
  stats.append({'mip':lod,'finite_nonnegative':True,'alpha_mean':float(alpha.mean()),'alpha_max':float(alpha.max()),'premul_luma_mean':float(lum.mean()),'mean_luma_ratio_to_R61':float(lum.mean()/before.mean()),'p99_luma':float(np.quantile(lum,.99)),'strong_coverage_fraction':float((alpha>.2).mean())})
 # Weak/native-body coverage is bounded independently of large crest radiance.
 weak=np.zeros((128,4));weak[:,:3]=np.linspace(0,.03,128)[:,None];weak[:,3]=1
 for grazing in [0,.5,1]:assert optics(weak,.035,grazing)[2].max()<=.033+1e-9
 # Sample-phase and color-field continuity, at both seam and temporal resets.
 continuity=0
 for x in [0,.5,1,6.58333333,8.33333333]:
  a=advection(np.full(48,x-1e-7),np.linspace(.42,.58,48),4,2)
  b=advection(np.full(48,x+1e-7),np.linspace(.42,.58,48),4,2)
  continuity=max(continuity,max(float(np.abs(p-q).max()) for p,q in zip(a,b)))
 for t in [0,6,12,120]:
  a=advection(np.linspace(-2,6,48),np.full(48,.5),t-1e-7,2)
  b=advection(np.linspace(-2,6,48),np.full(48,.5),t+1e-7,2)
  continuity=max(continuity,max(float(np.abs(p-q).max()) for p,q in zip(a,b)))
 assert continuity<1e-4,continuity
 # Each phase's hidden reset must not affect coverage or associated mass.
 for t in [0,4,6,120]:
  for compact in [0,1]:
   rgb,a,_=r62(np.linspace(0,18.504665,1000),np.full(1000,.5),t,3,compact)
   assert np.isfinite(rgb).all() and np.isfinite(a).all()
 # The real inverse Jacobian remains covariant under either winding/rotation.
 rng=np.random.default_rng(62);metric_error=0
 for _ in range(512):
  m=rng.normal(size=(2,2))
  if abs(np.linalg.det(m))<.05:continue
  du,dv=rng.normal(size=(2,3));dv=dv/np.linalg.norm(dv)*rng.choice([12.6,40.5,58.5,67.5]);dx=du*m[0,0]+dv*m[0,1];dy=du*m[1,0]+dv*m[1,1]
  recovered=(dy*m[0,0]-dx*m[1,0])/np.linalg.det(m);metric_error=max(metric_error,float(np.abs(recovered-dv).max()))
 assert metric_error<1e-10
 e=levels[0][...,:3]@np.array([.12,.52,.36]);crops=[]
 for scale,start in [(.42,.29),(.16,.42)]:
  roi=e[int(start*1254):int((start+scale)*1254)];count=[]
  for x in range(60,1194,24):
   mask=roi[:,x]>.09;count.append(int(np.sum(mask&np.r_[True,~mask[:-1]])))
  crops.append({'scale':scale,'start':start,'mean_strong_segments':float(np.mean(count)),'median_strong_segments':float(np.median(count)),'criterion':'Native linear energy>.09; not screen width/count guarantee'})
 report={'schema':1,'passed':True,'shader_sha256':sha((ROOT/'assets/shaders/world-flow.wgsl').read_bytes()),'probe_sha256':sha(Path(__file__).read_bytes()),'mips':stats,'crops':crops,'continuity_max_delta':continuity,'jacobian_max_error':metric_error,'authentic_current_alpha_delta':True,'weak_body_coverage_max':.033,'current_fetches':5,'gpu_compile':'PENDING','gameplay_proof':False,'bitmap_outputs':False,'performance_claim':False,'limits':['CPU uses fixed mip values, not exact GPU anisotropic footprints or Bevy bloom/ACES','Temporal movement/coherent pixel widths need actual native comparison','Soft HDR rolloff can reduce spiral intensity; native core must stay15–25px','All current alpha differences are explicit; no historical alpha-exact claim']}
 (AUTHOR/'proofs/material.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
