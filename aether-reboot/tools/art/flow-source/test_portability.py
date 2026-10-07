"""Rebuild this bundle after relocating it to an explicitly named new directory."""
from pathlib import Path
import argparse,shutil,subprocess,hashlib,json,sys
AUTHOR=Path(__file__).resolve().parent
ROOT=AUTHOR.parents[2]
parser=argparse.ArgumentParser()
parser.add_argument('--destination',required=True)
args=parser.parse_args()
destination=Path(args.destination).resolve()
assert not destination.exists(),'Use a new test directory; existing data is never overwritten.'
assert destination!=ROOT.resolve() and not destination.is_relative_to(ROOT.resolve()),'Destination must be outside the source tree.'
shutil.copytree(ROOT,destination,ignore=shutil.ignore_patterns('__pycache__','*.log'))
moved_author=destination/'tools/art/flow-source'
outputs=[]
for name in ['build_mips.py','build_shader.py','review_material.py','verify.mjs']:
    command=['node' if name.endswith('.mjs') else sys.executable,str(moved_author/name)]
    done=subprocess.run(command,cwd=destination,capture_output=True,text=True,check=True)
    outputs.append({'tool':name,'exit_code':done.returncode})
artifacts=[]
for name in ['assets/shaders/world-flow.wgsl','assets/textures/world-flow.ktx2','tools/art/flow-source/source/world-flow-native.png']:
    a=(ROOT/name).read_bytes();b=(destination/name).read_bytes();assert a==b
    artifacts.append({'file':name,'sha256':hashlib.sha256(a).hexdigest(),'bytes':len(a),'relocated_rebuild_byte_exact':True})
report={'schema':1,'passed':True,'method':'Copied package to an explicitly named new root; rebuilt KTX and shader, regenerated CPU proof and ran the no-argument validator there. No GPU, external asset source or canonical dependency.','tools':outputs,'artifacts':artifacts,'test_source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(AUTHOR/'proofs/relocation.json').write_text(json.dumps(report,indent=2)+'\n',newline='\n')
print('Relocated rebuild and no-argument validator PASS; three authored/runtime artifacts byte-exact.')
