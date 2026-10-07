"""Matched CPU rays compare the exact 3D function and cache; never GPU/paintover."""
import sys,json,hashlib,time
from pathlib import Path
import bpy,numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parent))
import review_cloud_light_r57_cpu as renderer
ROOT=renderer.ROOT;renderer.WIDTH=240;renderer.HEIGHT=150
def distribution(values):
    a=np.asarray(values).reshape(-1)
    return {'count':len(a),'mean':float(np.mean(a)),'p50_p95_p99_max':np.quantile(a,[.5,.95,.99,1]).tolist()}
def compare(actual,reference,categories):
    entry={}
    for region,mask in categories.items():
        if not np.any(mask):continue
        display=actual['display'][mask]-reference['display'][mask];mse=float(np.mean(display**2))
        entry[region]={'linear_absolute':distribution(np.abs(actual['linear'][mask]-reference['linear'][mask])),
                       'display_absolute':distribution(np.abs(display)),
                       'display_rmse':mse**.5,'display_psnr_db':float(-10*np.log10(max(mse,1e-20))),
                       'opacity_absolute':distribution(np.abs(actual['alpha'][mask]-reference['alpha'][mask]))}
    return entry
started=time.perf_counter()
report={'schema':1,'recipe':'r57-blue-fill-warm-direct','generator_sha256':hashlib.sha256((ROOT/'tools/art/build_cloud_cache.py').read_bytes()).hexdigest(),
        'renderer_sha256':hashlib.sha256((ROOT/'tools/art/review_cloud_light_r57_cpu.py').read_bytes()).hexdigest(),
        'comparison_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'shader_sha256':hashlib.sha256((ROOT/'assets/shaders/clouds.wgsl').read_bytes()).hexdigest(),
        'r47_contract_sha256':hashlib.sha256((ROOT/'tools/art/cloud-r47/contract.json').read_bytes()).hexdigest(),
        'historical_ray_comparison_sha256':hashlib.sha256((ROOT/'docs/evidence/cloud-r47-ray-comparison.json').read_bytes()).hexdigest(),
        'config_sha256':hashlib.sha256((ROOT/'assets/atmosphere/cloud-banks.json').read_bytes()).hexdigest(),
        'r55_ray_comparison_sha256':hashlib.sha256((ROOT/'docs/evidence/cloud-r55-ray-comparison.json').read_bytes()).hexdigest(),
        'note':'Fresh CPU recomputation with R57 ambient RGB gains4/4.5/5 and directRGB3/1.86/.96; no R55 pixel rescaling or relabelling. Same R47 four views, dimensions, alpha categories, fixed noise, midpoint rays and display transform. R57 unchanged low sun, offset shadow taps and directional bounce. Historical four-step plus current two-step refresh. No native-scene/performance claim.',
        'dimensions':[renderer.WIDTH,renderer.HEIGHT],'views':[]}
for refresh in [4,2]:
    renderer.LIGHTING_REFRESH=refresh
    for bank_index,bank in enumerate(['hollow','underforge']):
        renderer.BANK_INDEX=bank_index
        for view,origin,target,span in [('oblique',[3250,2200,3600],[0,-90,0],6200),('grazing',[3300,550,3800],[0,-140,0],5600)]:
            renderer.ORIGIN=origin;renderer.TARGET=target;renderer.SPAN=span
            label=bank+'-'+view+'-refresh'+str(renderer.LIGHTING_REFRESH)
            analytic=renderer.inspect(ROOT,label+'-analytic','analytic')
            cache=renderer.inspect(ROOT,label+'-cache','cache')
            mixed=renderer.inspect(ROOT,label+'-cache-density-analytic-normal','cache-density-analytic-normal')
            alpha=analytic['alpha'];categories={'all':np.ones(len(alpha),dtype=bool),'opaque_alpha_gt_095':alpha>.95,'edge_alpha_005_to_095':(alpha>.05)&(alpha<=.95),'empty_alpha_lte_005':alpha<=.05}
            item={'bank':bank,'view':view,'lighting_refresh_steps':renderer.LIGHTING_REFRESH,'comparisons':{},'images':[x['metadata'] for x in [analytic,cache,mixed]]}
            for name,actual,reference in [('cache_vs_analytic',cache,analytic),('density_error_with_analytic_normal',mixed,analytic),('normal_error_at_same_cached_density',cache,mixed)]:
                item['comparisons'][name]=compare(actual,reference,categories)
            # Quantitative image synthesized from radiance difference; red = 0.1.
            delta=np.max(np.abs(cache['display']-analytic['display']),axis=1)
            heat=np.stack((np.clip(delta/.1,0,1),np.clip(delta/.1-1,0,1),np.zeros_like(delta),np.ones_like(delta)),axis=1)
            im=bpy.data.images.new('CPU absolute display error '+label,width=renderer.WIDTH,height=renderer.HEIGHT,alpha=True,float_buffer=True)
            im.pixels.foreach_set(heat.reshape((renderer.HEIGHT,renderer.WIDTH,4))[::-1].flatten());im.filepath_raw=str(renderer.OUT/str('cloud-ray-error-'+label+'.png'));im.file_format='PNG';im.save()
            np.savez_compressed(renderer.OUT/str('cloud-ray-floats-'+label+'.npz'),analytic_linear=analytic['linear'],cache_linear=cache['linear'],mixed_linear=mixed['linear'],analytic_alpha=analytic['alpha'],cache_alpha=cache['alpha'])
            report['views'].append(item)
            print(label,json.dumps({k:v['all']['display_rmse'] for k,v in item['comparisons'].items()}),flush=True)
report['duration_seconds']=time.perf_counter()-started
report['legacy_pointwise_gate']='Still failed. This image diagnostic does not replace or waive the old threshold.'
(ROOT/'docs/evidence/cloud-r57-ray-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print('R57 four views at both refresh cadences freshly recomputed; R47 evidence unchanged.',flush=True)
