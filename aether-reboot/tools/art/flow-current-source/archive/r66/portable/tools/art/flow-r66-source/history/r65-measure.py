"""R65 CPU decision evidence. No shader variant, GPU texture or bitmap output."""
from pathlib import Path
import json,hashlib,struct,os
for name in ('OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS'):os.environ[name]='4'
import numpy as np
from source.r64_encoder import area_axis,decode_srgb
AUTHOR=Path(__file__).resolve().parent;ROOT=AUTHOR.parents[2];sha=lambda b:hashlib.sha256(b).hexdigest()
raw=(ROOT/'assets/textures/world-flow-current-r64.ktx2').read_bytes()
assert sha(raw)=='68755a59545f61bc047975e7af6c301bd98164468a6f293a1336e9bdbb5b1f05'
assert sha((AUTHOR/'source/world-flow-current-native.png').read_bytes())=='eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3'
assert sha((AUTHOR/'history/world-flow-r64.wgsl').read_bytes())=='0861bdc4197dab68fb10826a6165fa6ef5e325fa52ad6d42331a6b65e30c5cf5'
width,height=struct.unpack_from('<2I',raw,20);count=struct.unpack_from('<I',raw,40)[0];albedo=[]
for i in range(count):
 off,n,_=struct.unpack_from('<3Q',raw,80+i*24);w=max(1,width//2**i);h=max(1,height//2**i)
 tex=np.frombuffer(raw[off:off+n],dtype=np.uint8).reshape(h,w,4).astype(np.float64)/255
 tex[...,:3]=decode_srgb(tex[...,:3]);albedo.append(tex)
def smooth(a,b,x):
 f=np.clip((x-a)/(b-a),0,1);return f*f*(3-2*f)
def classify(tex,threshold):
 energy=tex[...,:3]@np.array([.12,.52,.36]);ridge=np.maximum(energy-.03,0)+energy*.1
 strong=smooth(threshold,threshold*3.5,ridge)
 hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
 cyan=hue*.2+np.array([.012,.48,1])*.8
 ink=smooth(.18,.32,tex[...,0]/np.maximum(tex[...,1],.0005));violet=ink*smooth(.035,.095,ridge)*.85
 hue=cyan*(1-violet[...,None])+np.array([.35,.025,1])*violet[...,None]
 return hue*ridge[...,None]*(.006+.994*strong[...,None])*tex[...,3,None]
def filter_next(data):return area_axis(area_axis(data,1),0)
def sample(tex,u,v):
 h,w,_=tex.shape;x=(u-np.floor(u))*w-.5;y=(v-np.floor(v))*h-.5
 ix=np.floor(x).astype(int);iy=np.floor(y).astype(int);fx=(x-ix)[...,None];fy=(y-iy)[...,None]
 return (tex[iy%h,ix%w]*(1-fx)+tex[iy%h,(ix+1)%w]*fx)*(1-fy)+(tex[(iy+1)%h,ix%w]*(1-fx)+tex[(iy+1)%h,(ix+1)%w]*fx)*fy
def luma(mass):return mass@np.array([.2126,.7152,.0722])
def bounded(mass,gain):
 peak=np.max(mass,axis=-1);return mass*(gain/(1+peak*gain/6))[...,None]
def main():
 # Preclassification at the actual full-resolution source threshold, once.
 mass=[classify(albedo[0],.035)]
 while mass[-1].shape[:2]!=(1,1):mass.append(filter_next(mass[-1]))
 conservation=[float(np.max(np.abs(m.mean(axis=(0,1))-mass[0].mean(axis=(0,1))))) for m in mass]
 assert max(conservation)<1e-12
 # Include a quantized16F simulation; this does not produce an image/container.
 half=[m.astype(np.float16).astype(np.float64) for m in mass]
 assert all(np.isfinite(m).all() and (m>=0).all() for m in half)
 half_mean_errors=[float(np.max(np.abs(m.mean(axis=(0,1))-reference.mean(axis=(0,1))))) for m,reference in zip(half,mass)]
 v,u=np.meshgrid((np.arange(320)+.5)/320,(np.arange(640)+.5)/640,indexing='ij');native_v=v*.42+.29
 profile=.12+1.88*(1-smooth(.16,.43,np.abs(v-.5)));gain=14*profile
 rows=[]
 for lod in range(9):
  tex=sample(albedo[lod],u,native_v);threshold=.035+(.020-.035)*smooth(2,6,np.asarray(lod))
  classified=classify(tex,threshold);prefiltered=sample(mass[lod],u,native_v);quantized=sample(half[lod],u,native_v)
  coverage=(.023+.005*smooth(.025,.1,tex[...,:3]@np.array([.12,.52,.36]))+.62*smooth(.05,.18,np.maximum(tex[...,:3]@np.array([.12,.52,.36])-.03,0)+(tex[...,:3]@np.array([.12,.52,.36]))*.1))*tex[...,3]
  old_l=luma(classified);ref_l=luma(prefiltered);next_l=luma(quantized)
  old_out=luma(bounded(classified,gain));next_out=luma(bounded(quantized,gain))
  rows.append({'mip':lod,'threshold_after_mip':float(threshold),'classified_after_mip_mean_mass_rgb':classified.mean(axis=(0,1)).tolist(),
   'fullres_classification_then_filter_mean_mass_rgb':prefiltered.mean(axis=(0,1)).tolist(),'prefilter_mass_luma_mean':float(ref_l.mean()),'after_mip_mass_luma_mean':float(old_l.mean()),
   'raw_recovery_ratio':float(ref_l.mean()/old_l.mean()),'R64_retained_fraction_of_native_prefilter':float(old_l.mean()/ref_l.mean()),
   'fp16_recovery_ratio':float(next_l.mean()/old_l.mean()),'fp16_mean_mass_luma_error':float(abs(next_l.mean()-ref_l.mean())),'bounded_world_gain_ratio':float(next_out.mean()/old_out.mean()),
   'unchanged_coverage_mean':float(coverage.mean()),'prefiltered_peak_mass':float(quantized.max()),'new_bound_output_peak':float(bounded(quantized,gain).max())})
  assert bounded(quantized,gain).max()<=6
 # Distinguish true all-dark data from a filtered pixel containing real peaks.
 dark=np.zeros((256,4));dark[:,:3]=np.linspace(0,.025,256)[:,None];dark[:,3]=1
 dark_mass=classify(dark,.035);dark_prefilter=np.mean(dark_mass,axis=0)
 assert float(dark_mass.max())<.00002
 # Direct controlled example: one native crest in a dark64-texel footprint.
 controlled=np.zeros((8,8,4));controlled[...,3]=.4;controlled[0,0]=[.04,.6,.95,.95]
 direct=classify(controlled,.035).mean(axis=(0,1));associated=controlled[...,:3]*controlled[...,3,None];a=controlled[...,3].mean();averaged=np.r_[associated.mean(axis=(0,1))/a,a]
 late=classify(averaged,.020);controlled_ratio=float(luma(direct)/max(float(luma(late)),1e-30))
 report={'schema':1,'passed':True,'native_png_sha256':'eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3','r64_ktx_sha256':sha(raw),'measure_source_sha256':sha(Path(__file__).read_bytes()),
  'crop':[.42,.29],'fullres_threshold':.035,'method':'Classify original linearRGBA texels once at mip0; exact full-area linear associated-crest-mass average. Compare sameUV against classification after actual sRGB/albedo mip samples; coverage not changed.',
  'full_source_mean_mass_rgb':mass[0].mean(axis=(0,1)).tolist(),'per_level_global_mass_conservation_max_abs_error':max(conservation),'fp16_per_level_mean_mass_max_abs_error':max(half_mean_errors),
  'mips':rows,'controlled_one_peak64texel_recovery_ratio':controlled_ratio,'all_dark_footprint_mean_mass_rgb':dark_prefilter.tolist(),
  'proposal':'One conservative linearFP16 RGB crest-mass auxiliary field, native classification-before-filtering; retain original albedo/coverage/body and all other shader parameters. Root decides only after these ratios.',
  'proposed_extra_gpu_texel_bytes':sum(max(1,width//2**i)*max(1,height//2**i)*8 for i in range(count)),
  'proposed_current_fetches':9,'original_current_fetches':5,'cascade_fetches_unchanged':2,
  'no_png_or_ktx_edits':True,'no_gpu_output':True,'no_shader_candidate_yet':True,'no_performance_claim':True,'limits':['CPU fixed mip and uniformUV crop, not native anisotropic footprints','RGB mass conservation before existing world gain/soft-rolloff; final output nonlinear response separately reported','FP16 quantization tested in memory only; actual KTX format support/DFD/binding requires validation','Radiance cannot establish perceived contrast against tonemapped cloud whites','Extra16.8MB/four samples and new binding are real cost; not accepted performance budget']}
 (AUTHOR/'proofs/decision-ratios.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
if __name__=='__main__':main()
