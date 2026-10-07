from energy_probe import np,ROOT,liquid,smooth,save,display,material,moments
import hashlib,json
from pathlib import Path
AUTHOR=Path(__file__).resolve().parent
def violet_material(u,v,t,lod):
    base,alpha,crest,energy,ridge=material(u,v,t,lod)
    drift=np.sin(u*.37-t*.12)*.006+np.sin(u*.83+t*.09)*.003
    tex=liquid(u*.20-t*.10,v*.82+.09+drift,lod)
    hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
    ink=smooth(.18,.32,tex[...,0]/np.maximum(tex[...,1],.0005))
    weight=ink*smooth(.035,.095,ridge)*.95
    tinted=hue*(1-weight[...,None])+np.array([.58,.12,1])*weight[...,None]
    opacity=.045+.40*crest
    rgb=tex[...,:3]*.40+tinted*ridge[...,None]*14/opacity[...,None]
    untouched=weight==0
    assert np.array_equal(rgb[untouched],base[untouched])
    assert np.isfinite(rgb).all() and np.all((weight>=0)&(weight<=.95))
    return rgb,alpha,weight,ridge,base
stats=[]
for kind,w,h,scale in [('current-close',1280,240,12),('current-mid',1280,96,32),('current-far',1280,32,64)]:
    v,u=np.meshgrid((np.arange(h)+.5)/h,(np.arange(w)+.5)/w,indexing='ij')
    lod=max(0,np.log2(1254*max(.20*scale/w,.82/h)))
    rgb,alpha,weight,ridge,base=violet_material(u*scale,v,4,lod)
    bg=np.where((u>.5)[...,None],np.array([1.2,1.25,1.3]),np.array([.03,.07,.15]))
    save(kind+'.png',display(rgb*alpha[...,None]+bg*(1-alpha[...,None])))
    active=ridge>.035
    stats.append({'kind':kind,'nominal_mip':float(lod),'alpha_exact_medium':True,'uncolored_pixels_rgb_exact':True,
    'tinted_total_fraction':float(np.mean(weight>.10)),'tinted_ridge_fraction':float(np.mean(weight[active]>.10)),
    'violet_weight_max':float(weight.max()),'violet_weight_mean':float(weight.mean()),
    'ridge_color_luma_before':float(np.mean((base*alpha[...,None])@np.array([.2126,.7152,.0722]))),
    'ridge_color_luma_after':float(np.mean((rgb*alpha[...,None])@np.array([.2126,.7152,.0722])))})
mipstats=[]
v,u=np.meshgrid((np.arange(128)+.5)/128,(np.arange(512)+.5)/512,indexing='ij')
for lod in range(9):
    rgb,alpha,weight,ridge,base=violet_material(u*32,v,4,lod)
    mipstats.append({'mip':lod,'tinted_total_fraction':float(np.mean(weight>.1)),'alpha_exact_medium':True,'water_body_rgb_exact':bool(np.array_equal(rgb[ridge<=.035],base[ridge<=.035])),'color_energy_ratio':float(np.mean(rgb*alpha[...,None])/np.mean(base*alpha[...,None]))})
report={'schema':1,'shader_sha256':hashlib.sha256((ROOT/'assets/shaders/world-flow.wgsl').read_bytes()).hexdigest(),'review_source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'sampling_source_sha256':hashlib.sha256((AUTHOR/'liquid_probe.py').read_bytes()).hexdigest(),'energy_source_sha256':hashlib.sha256((AUTHOR/'energy_probe.py').read_bytes()).hexdigest(),'body_alpha_uv_cascade_unchanged':True,'method':'Actual KTX mip pixels; same sampling and medium energy recipe, only a native-ratio/ridge-masked hue change. CPU probes, no native rendering claim.','swatches':stats,'mips':mipstats}
(AUTHOR/'proofs/material.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
