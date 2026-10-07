"""Reconstruct the reviewed final shader from the pinned medium source.
The only allowed patch changes current crest hue. Alpha/UV/clock stay exact.
"""
import hashlib,json
from pathlib import Path
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
source=AUTHOR/'history/reviewed-medium.wgsl'
raw=source.read_bytes()
assert hashlib.sha256(raw).hexdigest()=='3d9432e720abcae6ab54c3c0ebe7fa9081d2394d178f30ee879502ffa0c85d61'
needle=b'        rgb=tex.rgb*.40+hue*ridge*14.0/opacity;'
block='''        // The native red/green ratio selects a few authored crest portions.
        // A ridge-only gate leaves the water body and all alpha unchanged.
        let violet_ink=smoothstep(.18,.32,tex.r/max(tex.g,.0005));
        let violet_weight=violet_ink*smoothstep(.035,.095,ridge)*.95;
        let crest_hue=mix(hue,vec3<f32>(.58,.12,1.0),violet_weight);
        rgb=tex.rgb*.40+crest_hue*ridge*14.0/opacity;'''.replace('\n','\r\n').encode()
assert raw.count(needle)==1
shader=raw.replace(needle,block)
assert hashlib.sha256(shader).hexdigest()=='f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41'
output=ROOT/'assets/shaders/world-flow.wgsl';output.parent.mkdir(parents=True,exist_ok=True)
output.write_bytes(shader)
report={'schema':1,'shader_sha256':hashlib.sha256(shader).hexdigest(),'medium_source_sha256':hashlib.sha256(raw).hexdigest(),'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'patch_scope':'Current hue only; alpha, body, UVs, cascade, gain, pool, clock and haze exact to reviewed medium. Explicit CRLF author bytes preserve cross-platform shader hash.'}
(AUTHOR/'proofs').mkdir(parents=True,exist_ok=True)
(AUTHOR/'proofs/shader.json').write_text(json.dumps(report,indent=2)+'\n',newline='\n')
print('Final shader rebuilt byte-exact: '+report['shader_sha256'])
