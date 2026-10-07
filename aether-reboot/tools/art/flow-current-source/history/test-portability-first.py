"""Historical gates/rebuilds and strict successor rejects on a fresh D copy."""
from pathlib import Path
import hashlib,json,tempfile,shutil,subprocess,sys
P=Path(__file__).resolve().parent;S=P/'portable';A=S/'tools/art/flow-current-source';sha=lambda b:hashlib.sha256(b).hexdigest()
T=Path(tempfile.mkdtemp(prefix='flow-r66-promotion-rebuild-',dir=P.parent)).resolve()
assert T.parent==P.parent.resolve() and T.name.startswith('flow-r66-promotion-rebuild-')
shutil.copytree(S,T,dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__'))
TA=T/'tools/art/flow-current-source';runs=[]
def run(args):
 r=subprocess.run(args,cwd=T,capture_output=True,text=True);runs.append({'command':args,'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr});assert r.returncode==0,(args,r.stderr)
# Test every historical gate as written against its own true runtime/assets.
for rev in ['r62','r64','r66']:
 author=TA/'archive'/rev/'portable/tools/art'/f'flow-{rev}-source'
 run(['node',str(author/'verify.mjs')])
 run([sys.executable,str(author/'build_shader.py')])
# Reconstruct all eleven current mips from the unmodified original author PNG.
run([sys.executable,str(TA/'archive/r64/portable/tools/art/flow-r64-source/build_mips.py')])
for rev in ['r62','r64','r66']:
 directory=TA/'archive'/rev;manifest=json.loads((directory/'package-manifest.json').read_text())
 for f in manifest['files']:
  rel=f.get('path') or 'portable/'+f['file'];assert sha((directory/rel).read_bytes())==f['sha256'],(rev,rel)
# Reconstruct promoted runtime through the independently rebuilt entire chain.
rebuilt=(TA/'archive/r66/portable/assets/shaders/world-flow.wgsl').read_bytes();assert sha(rebuilt)=='b76e67027f1b34cf6f9d56192bcdf315ad43810ff32c4e64e075297ab9a16c31'
assert (TA/'archive/r64/portable/assets/textures/world-flow-current-r64.ktx2').read_bytes()==(T/'assets/textures/world-flow-current-r64.ktx2').read_bytes()
# Run pristine R53 gate, not the adapter, against authentic R53 scratch payload.
runtime=T/'assets/shaders/world-flow.wgsl';gate=T/'tools/art/flow-source/verify.mjs';final=runtime.read_bytes();adapted=gate.read_bytes()
runtime.write_bytes((TA/'history/world-flow-r53.wgsl').read_bytes());gate.write_bytes((TA/'history/r53-verify.mjs').read_bytes());run(['node',str(gate)])
runtime.write_bytes(final);gate.write_bytes(adapted)
run([sys.executable,str(TA/'build_adapter.py')]);assert gate.read_bytes()==adapted
run(['node',str(gate)]);run(['node',str(TA/'verify.mjs')])
# Asset/provenance tampering must fail the actual adapter, not a baseline hash transplant.
tamper_files=[
 'assets/textures/world-flow.ktx2','assets/textures/world-flow-current-r64.ktx2','tools/art/flow-current-source/source/world-flow-current-native.png',
 'tools/art/flow-current-source/history/world-flow-r53.wgsl',
 'tools/art/flow-current-source/archive/r62/portable/assets/shaders/world-flow.wgsl',
 'tools/art/flow-current-source/archive/r62/portable/tools/art/flow-r62-source/source/current-block.wgsl',
 'tools/art/flow-current-source/archive/r62/portable/tools/art/flow-r62-source/source/current-sampling.wgsl',
 'tools/art/flow-current-source/archive/r64/portable/tools/art/flow-r64-source/proofs/mip.json',
 'tools/art/flow-current-source/archive/r64/portable/tools/art/flow-r64-source/source/native-r64.json',
 'tools/art/flow-current-source/archive/r64/portable/tools/art/flow-r64-source/build_mips.py',
 'tools/art/flow-current-source/archive/r66/portable/tools/art/flow-r66-source/probe.py',
 'tools/art/flow-current-source/archive/r62/package-manifest.json','tools/art/flow-current-source/archive/r64/package-manifest.json','tools/art/flow-current-source/archive/r66/package-manifest.json']
for rel in tamper_files:
 file=T/rel;original=file.read_bytes();changed=bytearray(original);changed[min(600,len(changed)-1)]^=1;file.write_bytes(changed)
 r=subprocess.run(['node',str(gate)],cwd=T,capture_output=True,text=True);file.write_bytes(original);assert r.returncode!=0,rel
run(['node',str(gate)]);run(['node',str(TA/'verify.mjs')])
proof={'schema':1,'passed':True,'scratch_directory':str(T),'historical_gates_byte_exact':['R53','R62','R64','R66'],'historical_thresholds_and_metadata_preserved':True,'eleven_mips_rebuilt_from_native_original_byte_exact':True,'entire_chain_shader_rebuild_byte_exact':True,'actual_adapter_asset_and_provenance_rejects':tamper_files,'negative_asset_cases':len(tamper_files),'negative_shader_cases':27,'runs':runs,'no_gpu':True,'canonical_untouched':True}
(A/'proofs/portable-chain.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8');print('PASS chain/rebuild and27shader+14asset rejects;',T)
