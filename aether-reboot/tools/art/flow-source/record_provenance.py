"""Record the portable final source/proof chain; no external files required."""
from pathlib import Path
import hashlib,json
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
def item(path):
    return {'file':path.relative_to(ROOT).as_posix(),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size}
report={'schema':1,'status':'Medium-violet retained after parent native review. No further shader changes.',
'tool':'built-in image_gen.imagegen','fallback_cli':False,'native_pixels_untouched':True,
'original_generated_location':'C:/Users/Damien/.codex/generated_images/01a0fcd3-904f-78d3-9e49-e5b56eb81088/exec-f9d8b43e-030a-47da-a377-4e97ba47b719.png',
'original_location_role':'Provenance only; no tool or runtime reads this external path.',
'native':item(AUTHOR/'source/world-flow-native.png'),'prompt':item(AUTHOR/'source/world-flow-prompt.txt'),
'runtime_shader':item(ROOT/'assets/shaders/world-flow.wgsl'),'derived_runtime_texture':item(ROOT/'assets/textures/world-flow.ktx2'),
'sources':[item(AUTHOR/name) for name in ['build_shader.py','build_mips.py','liquid_probe.py','energy_probe.py','review_material.py','verify.mjs','test_portability.py','record_provenance.py']],
'proofs':[item(AUTHOR/'proofs'/name) for name in ['shader.json','mip.json','material.json','container.json','relocation.json']],
'history':[item(AUTHOR/'history'/name) for name in ['rejected-r53.wgsl','reviewed-medium.wgsl','rejected-strong.wgsl','prior-r52.wgsl']],
'native_reviews':[{'result':result,**item(AUTHOR/'review'/name)} for name,result in [('world-r53-material-sail.png','Rejected: dark strip and missing crests'),('world-r53-medium.png','Retained energy base'),('world-r53-strong.png','Rejected: excessive whitening'),('world-r53-violet.png','Retained final narrow crest violet')]],
'gpu_bytes':8384072,'native_rgba_level0_exact':True,'mip_levels':11,'format':'Rgba8UnormSrgb Vk43 straight alpha',
'changes':'Original finer native liquid; linear alpha-weighted mip derivation; two-fetch periodic advection; positive blue-inclusive ridge energy; violet restricted to authored current crest portions. Geometry/alpha/clock/cascade preserved relative to reviewed medium.',
'no_canonical_mutation_by_author':True,'no_engine_gpu_actions_by_author':True}
(AUTHOR/'source/provenance.json').write_text(json.dumps(report,indent=2)+'\n',newline='\n')
print('Final portable provenance recorded.')
