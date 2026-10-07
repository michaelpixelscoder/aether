"""CPU energy/mip probes for the two native-capture calibration variants."""
from liquid_probe import np,ROOT,liquid,smooth,save,display,material as rejected_material
from pathlib import Path
import hashlib,json
def material(u,v,t,lod,fall=False,strong=False):
    if fall:
        sway=np.sin(v*7-t*.30)*.004+np.sin(v*15-t*.45)*.002
        tex=liquid(v*1.10-t*.20,u*.24+.37+sway,lod)
    else:
        drift=np.sin(u*.37-t*.12)*.006+np.sin(u*.83+t*.09)*.003
        tex=liquid(u*.20-t*.10,v*.82+.09+drift,lod)
    energy=tex[...,:3]@np.array([.12,.52,.36])
    ridge=np.maximum(energy-.030,0)+energy*.10
    crest=smooth(.025,.20,ridge)
    opacity=.012+.62*crest if fall else .045+.40*crest
    hue=tex[...,:3]/np.maximum(np.max(tex[...,:3],axis=-1),.001)[...,None]
    gain=(20 if strong else 12) if fall else (24 if strong else 14)
    rgb=tex[...,:3]*(.35 if fall else .40)+hue*ridge[...,None]*gain/opacity[...,None]
    if fall:
        edge=smooth(.015,.075,u)*smooth(.015,.075,1-u)
        end=1-smooth(.68+tex[...,1]*.12,.93+tex[...,1]*.05,v)
        alpha=.8*edge*tex[...,3]*smooth(0,.025,v)*end*opacity
    else:
        edge=smooth(.015,.095,v)*smooth(.015,.095,1-v)
        alpha=.9*edge*tex[...,3]*opacity
    return rgb,alpha,crest,energy,ridge

def moments(rgb,alpha,crest):
    premul=rgb*alpha[...,None];luma=premul@np.array([.2126,.7152,.0722])
    return {'alpha_mean':float(alpha.mean()),'alpha_max':float(alpha.max()),'premultiplied_rgb_mean':np.mean(premul,axis=(0,1)).tolist(),'premultiplied_luma_mean':float(luma.mean()),'premultiplied_luma_p95':float(np.quantile(luma,.95)),'premultiplied_luma_p99':float(np.quantile(luma,.99)),'strong_crest_fraction':float(np.mean(crest>.5))}

