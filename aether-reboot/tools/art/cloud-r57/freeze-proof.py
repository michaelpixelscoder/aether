"""Finalize separate R57 contract/provenance after fresh CPU recomputation."""
from pathlib import Path
import json,hashlib
ROOT=Path(__file__).resolve().parents[3]
P=ROOT
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
write=lambda p,v:p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
r55=json.loads((P/'tools/art/cloud-r55/contract.json').read_text())
oldsource=json.loads((P/'tools/art/cloud-r55/sources.json').read_text())
history=dict(r55['r47_unchanged'])
history.update({('tools/art/cloud-r57/archive/verify-cloud-lighting-r55.mjs.txt' if f=='tools/art/verify-cloud-lighting.mjs' else f):h for f,h in oldsource.items()})
for f in ['tools/art/cloud-r55/contract.json','tools/art/cloud-r55/sources.json','tools/art/cloud-r55/provenance.json',
    'docs/evidence/cloud-r55-ray-comparison.json','docs/evidence/cloud-r55-lighting-fidelity.json','docs/evidence/cloud-r55-negative-tests.json',
    'docs/evidence/cloud-cache-assets.json','tools/art/cloud-r57/archive/clouds-r55.wgsl']:
    history[f]=sha(P/f)
contract={'schema':1,'recipe':'r57-blue-fill-warm-direct','shader_sha256':sha(P/'assets/shaders/clouds.wgsl'),
    'limits':r55['limits'],'historical_unchanged':history,
    'lighting':{'ambient_rgb_gain':[4,4.5,5],'direct_radiance':[3,1.86,.96],'bounce_radiance':[3,1.86,.96],
        'bounce_strength':.35,'phase_g':.45,'bounce_elevation_degrees':42,'shadow_taps_per_lobe':4,'lighting_refresh_steps':[4,2],'daylight_factor':1},
    'proof_policy':'Fresh 24 CPU fields from actual R57 radiance; preserve all R47/R55 original contract/source/ray bytes. Historical display transform and fixed-noise cameras remain controlled.',
    'limits_notes':['Regional historical .035 pointwise failure remains failed.','No native performance, temporal or graphical-fidelity claim.']}
write(P/'tools/art/cloud-r57/contract.json',contract)
files=['tools/art/review_cloud_light_r57_cpu.py','tools/art/compare_cloud_light_r57_rays_cpu.py',
    'tools/art/verify-cloud-r57-lighting.mjs','tools/art/verify-cloud-lighting.mjs','tools/art/cloud-r57/verify-r55-historical.mjs',
    'tools/art/cloud-r57/test-gate.mjs','tools/art/cloud-r57/freeze-proof.py','tools/art/cloud-r57/archive/clouds-r55.wgsl','tools/art/cloud-r57/archive/verify-cloud-lighting-r55.mjs.txt']
files+=sorted(p.relative_to(P).as_posix() for p in (P/'tools/art/cloud-r57/review').glob('*') if p.is_file())
write(P/'tools/art/cloud-r57/sources.json',{f:sha(P/f) for f in files})
rays=json.loads((P/'docs/evidence/cloud-r57-ray-comparison.json').read_text())
assert rays['recipe']=='r57-blue-fill-warm-direct' and len(rays['views'])==8
metrics=[{'bank':v['bank'],'view':v['view'],'refresh':v['lighting_refresh_steps'],
    'opaque_rmse':v['comparisons']['cache_vs_analytic']['opaque_alpha_gt_095']['display_rmse'],
    'edge_alpha_p99':v['comparisons']['cache_vs_analytic']['edge_alpha_005_to_095']['opacity_absolute']['p50_p95_p99_max'][2]} for v in rays['views']]
print(json.dumps({'status':'contract ready; gates pending','wall_seconds':rays['duration_seconds'],'views':metrics}))
