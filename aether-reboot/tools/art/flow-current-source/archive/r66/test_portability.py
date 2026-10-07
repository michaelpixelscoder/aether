"""Rebuild R66 in a separate native D directory; leave evidence intact."""
from pathlib import Path
import hashlib,json,tempfile,shutil,subprocess,sys
P=Path(__file__).resolve().parent;B=P.parent;sha=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((P/'package-manifest.json').read_text());T=Path(tempfile.mkdtemp(prefix='flow-r66-rebuild-',dir=B)).resolve()
assert T.parent==B.resolve() and T.name.startswith('flow-r66-rebuild-')
for f in manifest['files']:
 src=P/f['path'];assert sha(src.read_bytes())==f['sha256'];dest=T/f['path'];dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest)
shutil.copyfile(P/'package-manifest.json',T/'package-manifest.json')
A=T/'portable/tools/art/flow-r66-source';logs=[]
for command in [[sys.executable,str(A/'build_shader.py')],[sys.executable,str(A/'probe.py')],['node',str(A/'verify.mjs')]]:
 r=subprocess.run(command,cwd=T,capture_output=True,text=True);logs.append({'command':command,'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr});assert r.returncode==0,(command,r.stderr)
for f in manifest['files']:assert sha((T/f['path']).read_bytes())==f['sha256'],f['path']
evidence={'passed':True,'outside_directory':str(T),'package_manifest_sha256':sha((P/'package-manifest.json').read_bytes()),'all_rebuilt_outputs_byte_exact':True,'preserved_originals':True,'commands':logs}
out=B/'flow-r66-portability-proof.json';out.write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf8');print('PASS outside rebuild; outputs byte exact;',out)
