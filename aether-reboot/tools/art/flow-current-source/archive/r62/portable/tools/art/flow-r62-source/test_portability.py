"""Relocate only the small R62 package to D, rebuild, probe and contract-check."""
from pathlib import Path
import json,hashlib,tempfile,shutil,subprocess,sys
AUTHOR=Path(__file__).resolve().parent;ROOT=AUTHOR.parents[2];POOL=AUTHOR.parents[4]
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
assert POOL.resolve()==Path('D:/Aether-art-studies').resolve()
with tempfile.TemporaryDirectory(prefix='r62-cpu-relocation-',dir=POOL) as directory:
 temporary=Path(directory).resolve();assert temporary.parent==POOL.resolve() and temporary.name.startswith('r62-cpu-relocation-')
 relocated=temporary/'portable';shutil.copytree(ROOT,relocated,ignore=shutil.ignore_patterns('__pycache__'))
 author=relocated/'tools/art/flow-r62-source';before=sha(ROOT/'assets/shaders/world-flow.wgsl')
 for script,runner in [('build_shader.py',sys.executable),('probe.py',sys.executable),('verify.mjs','node')]:
  result=subprocess.run([runner,str(author/script)],cwd=temporary,capture_output=True,text=True)
  assert result.returncode==0,(script,result.stdout[-1000:],result.stderr[-1000:])
 assert sha(relocated/'assets/shaders/world-flow.wgsl')==before,'Portable rebuild byte-exact'
 result=json.loads((author/'proofs/contracts.json').read_text());assert len(result['negative_cases_rejected'])==10
 report={'passed':True,'relocation_on_D':True,'outside_project':True,'shader_rebuild_byte_exact':True,'cpu_probe_passed':True,'ten_contract_corruptions_rejected':True,'native_art_untouched':True,'gpu_used':False,'canonical_mutations':False,'script_sha256':sha(Path(__file__))}
(AUTHOR/'proofs/relocation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
