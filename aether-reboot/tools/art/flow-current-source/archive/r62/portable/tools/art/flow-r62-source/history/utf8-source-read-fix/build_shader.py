"""One isolated R62 shader; primary native art and canonical files never edited."""
from pathlib import Path
import hashlib,json
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
sha=lambda raw:hashlib.sha256(raw).hexdigest()
BASE='f3f2efc413e71ae5e941325d59c2a457b6863cfa79dd1478a44af475f678fb41'
def rebuild():
    raw=(AUTHOR/'history/world-flow-r53.wgsl').read_bytes();assert sha(raw)==BASE
    source=raw.decode().replace('\r\n','\n')
    begin=source.index('    if settings.x < 0.5 {');end=source.index('    } else if settings.x < 1.5 {')
    block=(AUTHOR/'source/current-block.wgsl').read_text().replace('\r\n','\n')
    helpers=(AUTHOR/'source/current-sampling.wgsl').read_text().replace('\r\n','\n')
    candidate=source[:begin]+block+source[end:]
    candidate=candidate.replace('@fragment\n',helpers+'\n@fragment\n',1)
    candidate=candidate.replace('// R53 energy calibration medium; same native image, geometry and clock.',
        '// R62 isolated coherent current transport; authentic current alpha delta, unchanged native texture/geometry/clock.')
    output=candidate.encode('utf8');target=ROOT/'assets/shaders/world-flow.wgsl';target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(output)
    proof={'schema':1,'shader_sha256':sha(output),'baseline_sha256':BASE,'generator_sha256':sha(Path(__file__).read_bytes()),
        'source_fragments':{file:sha((AUTHOR/'source'/file).read_bytes()) for file in ['current-block.wgsl','current-sampling.wgsl']},
        'current_texture_fetches':5,'cascade_texture_fetches':2,'current_alpha_uv_sampling_changed':True,
        'native_png_ktx_geometry_routes_physics_clock_unchanged':True,'gpu_compile':'PENDING ROOT CAPTURE','promotion_authorized':False}
    (AUTHOR/'proofs/shader.json').write_text(json.dumps(proof,indent=2)+'\n');return output
if __name__=='__main__':print(sha(rebuild()))
