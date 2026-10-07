"""R64 minimal native-asset UV adaptation onto frozen R62; no knob grid."""
from pathlib import Path
import json,hashlib
AUTHOR=Path(__file__).resolve().parent;ROOT=AUTHOR.parents[2];sha=lambda raw:hashlib.sha256(raw).hexdigest()
def rebuild():
 raw=(AUTHOR/'history/world-flow-r62.wgsl').read_bytes();assert sha(raw)=='f5f7517a5123232cb2b6ee9f02527a9753840a3ebec152f72557289cd4869cc1'
 source=raw.decode('utf-8')
 replacements={
 '// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.':'// R64 fresh coherent current art; separate texture handle, unchanged R62 optics and historical waterfalls/pools.',
 '        // Broad paths:~7 strong native crest groups per transverse section,\n        // versus~20 in R61; the compact thermal keeps its R61 crop scale.':'        // The new native asset has three coherent unequal bands.\n        // Both broad paths and the thermal sample the full central42% crop.',
 'let crop=mix(.16,.42,thermal);':'let crop=.42;',
 'let crop_start=mix(.42,.29,thermal);':'let crop_start=.29;',
 '// R62: true authored color samples are classified BEFORE either blend.':'// R64 uses unchanged R62 optical sampling with the fresh current-only image.'}
 for before,after in replacements.items():assert source.count(before)==1;source=source.replace(before,after)
 output=source.encode('utf-8');(ROOT/'assets/shaders/world-flow.wgsl').write_bytes(output)
 proof={'schema':1,'shader_sha256':sha(output),'generator_sha256':sha(Path(__file__).read_bytes()),'baseline_r62_sha256':sha(raw),'changes':'New art crop.42+.29 both widths; labels only. R62 color/radiance/coverage/sampling/clock unchanged.',
 'bindings':{'current_only_texture':'assets/textures/world-flow-current-r64.ktx2','waterfall_pool_texture':'assets/textures/world-flow.ktx2','current_only_handle_must_be_wired_by_root':True},'cascade_pool_branches_and_original_sampler_exact':True,'gpu_compile':'PENDING root current-only override'}
 (AUTHOR/'proofs/shader.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8');print(json.dumps(proof));return output
if __name__=='__main__':rebuild()
