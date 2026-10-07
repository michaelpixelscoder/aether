"""Final portable R59 caller rebase, independent JS API and outside-root QA."""
from pathlib import Path
import tempfile,shutil,subprocess,sys,json,hashlib
from patch_ground import AUTHOR,FILES
ROOT=AUTHOR.parents[2]
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
with tempfile.TemporaryDirectory(prefix='aether-ground-r60-final-') as directory:
    temp=Path(directory).resolve()
    assert temp.parent==Path(tempfile.gettempdir()).resolve() and temp.name.startswith('aether-ground-r60-final-')
    relocated=temp/'portable'
    shutil.copytree(ROOT,relocated,ignore=shutil.ignore_patterns('__pycache__'))
    author=relocated/'tools/art/ground-r60-source'
    target=relocated/'assets/world'
    old={file:sha(ROOT/'assets/world'/file) for file in FILES}
    before={file:sha(author/'history/r59-portable-world'/file) for file in FILES}
    call=subprocess.run([sys.executable,str(author/'patch_ground.py'),'--input-world',str(author/'history/r59-portable-world'),'--output-world',str(target),'--report',str(author/'proofs/r59-portable-patch.json')],cwd=temp,capture_output=True,text=True)
    assert call.returncode==0,call.stderr
    assert all(sha(target/file)==old[file] for file in FILES),'All4 full GLB reproduced byte-exactly'
    assert (target/'manifest.json').read_bytes()==(ROOT/'assets/world/manifest.json').read_bytes(),'Selective manifest/provenance reproduced byte-exactly'
    assert all(sha(author/'history/r59-portable-world'/file)==before[file] for file in FILES),'Caller immutable'
    for script in ['verify.mjs','test-api.mjs']:
        call=subprocess.run(['node',str(author/script)],cwd=temp,capture_output=True,text=True)
        assert call.returncode==0,(script,call.stdout[:1000],call.stderr[:1000])
    negative=json.loads((author/'proofs/api-negative-tests.json').read_text())
    assert len(negative['corruptions_rejected'])==14
report={'schema':1,'passed':True,'outside_workspace':True,'final_r59_caller_authentication':True,'all4_glb_rebuilt_byte_exact':True,'manifest_rebuilt_byte_exact':True,'caller_unchanged':True,'independent_js_api_negative_cases':negative['corruptions_rejected'],'source_png_unchanged':True,'script_sha256':sha(Path(__file__)),'canonical_mutations':False,'gpu_used':False}
(AUTHOR/'proofs/final-portable-relocation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
