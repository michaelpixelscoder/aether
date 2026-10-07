"""CPU R66 radiance-only delta quantification. No texture or shader output."""
from r65_math import *
def main():
 v,u=np.meshgrid((np.arange(512)+.5)/512,(np.arange(1024)+.5)/1024,indexing='ij');nv=.42*v+.29
 old_profile=.12+1.88*(1-smooth(.16,.43,np.abs(v-.5)))
 # Fixed domain boundaries discovered from the exact R64 native row-mass valleys.
 splits=[.29,.379585326953748,.5813397129186603,.71]
 rows=[]
 for mip in range(9):
  tex=sample(albedo[mip],u,nv);threshold=.035-.015*smooth(2,6,np.asarray(mip));mass=classify(tex,threshold)
  old=bounded(mass,14*old_profile);new=bounded(mass,np.full(v.shape,14.))
  # Thermal core uses unchanged middle, coreMass conversion and gain factor.
  middle=1-smooth(.16,.43,np.abs(v-.5));core=middle
  peak=np.max(mass,axis=-1);core_mass=mass*(1-core[...,None]*.85)+np.array([.52,.92,1])*peak[...,None]*core[...,None]*.85
  core_old=core_mass*(14*old_profile*(1+2*core)/(1+peak*14*old_profile*(1+2*core)/6))[...,None]
  core_new=core_mass*(14*(1+2*core)/(1+peak*14*(1+2*core)/6))[...,None]
  energy=tex[...,:3]@np.array([.12,.52,.36]);ridge=np.maximum(energy-.03,0)+energy*.1
  coverage=(.023+.005*smooth(.025,.1,energy)+.62*smooth(.05,.18,ridge))*tex[...,3]
  body=(tex[...,:3]*.22+np.array([.002,.012,.04]))*(.023+.005*smooth(.025,.1,energy))[...,None]*tex[...,3,None]
  def metrics(sel):
   old_l=luma(old)[sel];new_l=luma(new)[sel];old_total=luma(old+body)[sel];new_total=luma(new+body)[sel]
   return {'native_associated_mass_rgb_before_and_after':mass[sel].mean(axis=0).tolist(),'native_mass_delta_max_abs':0.,'coverage_mean_before_and_after':float(coverage[sel].mean()),'coverage_delta_max_abs':0.,'crest_radiance_mean_rgb_before':old[sel].mean(axis=0).tolist(),'crest_radiance_mean_rgb_after':new[sel].mean(axis=0).tolist(),'crest_radiance_luma_before':float(old_l.mean()),'crest_radiance_luma_after':float(new_l.mean()),'crest_radiance_luma_ratio':float(new_l.mean()/old_l.mean()),'body_plus_crest_luma_ratio':float(new_total.mean()/old_total.mean()),'peak_radiance_before':float(old[sel].max()),'peak_radiance_after':float(new[sel].max()),'thermal_full_core_luma_ratio':float(luma(core_new)[sel].mean()/luma(core_old)[sel].mean())}
  assert np.isfinite(new).all() and new.min()>=0 and new.max()<=6
  # At every native fragment, profile-change gain is constrained to.5..8.334x;
  # monotonic bounded rolloff can only reduce this expansion/contraction.
  ratio=np.divide(new,old,out=np.ones_like(new),where=old>0)
  assert ratio.min()>=.5-1e-10 and ratio.max()<=1/.12+1e-10
  rows.append({'mip':mip,'threshold_unchanged':float(threshold),'whole_crop':metrics(np.ones(v.shape,dtype=bool)),'bands':[dict(native_v_span=[lo,hi],**metrics((nv>=lo)&(nv<hi))) for lo,hi in zip(splits[:-1],splits[1:])],'per_fragment_radiance_ratio_minmax':[float(ratio.min()),float(ratio.max())]})
 dark=np.zeros((256,4));dark[:,:3]=np.linspace(0,.025,256)[:,None];dark[:,3]=1
 dark_mass=classify(dark,.035);assert np.max(classify(np.array([0.,0.,0.,1.]),.035))==0
 report={'schema':1,'passed':True,'probe_sha256':sha(Path(__file__).read_bytes()),'r64_texture_sha256':sha(raw),'mips':rows,'changed_parameter_only':'optical_profile=.12+1.88*middle -> optical_profile=1.0','coverage_and_native_mass_identical':True,'fetch_budget_identical':5,'zero_rgb_crest_mass_stays_zero':True,'weak_linear_rgb_up_to_point025_new_peak_crest_radiance':float(bounded(dark_mass,np.full(256,14.)).max()),'no_gpu':True,'no_performance_claim':True,'limits':['Fixed mip bilinear single-native samples; actual temporal/seam mixture and anisotropic GPU filtering not measured','Mean response over uniform1024x512UV samples, not screen-area integration','Existing body included with fixed grazing.5; thermal fullcore diagnostic at U>=18.25 only','Core blue term unchanged and excluded from thermal ratio; thermal gain/profile change still changes final core radiance','Associated HDR radiance before compositor/exposure/tonemap, not perceived scene contrast','Central network decreases; no promised final visual improvement']}
 (AUTHOR/'proofs/radiance-delta.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
 for r in rows:print(r['mip'],r['whole_crop']['crest_radiance_luma_ratio'],[b['crest_radiance_luma_ratio'] for b in r['bands']])
if __name__=='__main__':main()
