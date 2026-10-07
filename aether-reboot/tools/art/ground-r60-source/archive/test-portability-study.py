"""Relocate R60 and rebuild independently over real frozen R56 and R59 inputs."""
from pathlib import Path
import tempfile,shutil,subprocess,sys,json,hashlib,copy
from patch_ground import AUTHOR,FILES,parse,pack,SOURCE_SHA

ROOT=AUTHOR.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
with tempfile.TemporaryDirectory(prefix='aether-ground-r60-') as temporary:
    temp=Path(temporary).resolve()
    assert temp.parent==Path(tempfile.gettempdir()).resolve() and temp.name.startswith('aether-ground-r60-')
    relocated=temp/'package'
    shutil.copytree(ROOT,relocated,ignore=shutil.ignore_patterns('__pycache__'))
    author=relocated/'tools/art/ground-r60-source'
    results=[]
    for revision in ['r56','r59']:
        baseline=author/f'history/{revision}-world'
        destination=temp/f'{revision}-rebuilt'
        proof=temp/f'{revision}-rebuilt.json'
        call=subprocess.run([sys.executable,str(author/'patch_ground.py'),'--input-world',str(baseline),'--output-world',str(destination),'--report',str(proof)],cwd=temp,capture_output=True,text=True)
        assert call.returncode==0,call.stderr
        verify=lambda:subprocess.run(['node',str(author/'verify.mjs'),'--input-world',str(baseline),'--output-world',str(destination),'--report',str(proof)],cwd=temp,capture_output=True,text=True)
        assert verify().returncode==0
        historical=json.loads((AUTHOR/f'proofs/{revision}-patch.json').read_text())
        for item in historical['files']:assert sha(destination/item['file'])==item['output_sha256']
        results.append({'revision':revision,'real_archived_input':True,'all4_glbs_rebuilt_byte_exact':True,'independent_node_delta_gate_pass':True})
        if revision!='r59':continue
        target=destination/'dawn.glb'
        def modify_json(change):
            doc,binary=parse(target.read_bytes());change(doc);return pack(doc,binary)
        def change_geometry(data):
            doc,binary=parse(data);binary=bytes([binary[0]^1])+binary[1:];return pack(doc,binary)
        def other_material(doc):doc['materials'][0]['doubleSided']=not doc['materials'][0].get('doubleSided',False)
        def target_roughness(doc):doc['materials'][11]['pbrMetallicRoughness']['roughnessFactor']=.5
        def target_factor(doc):doc['materials'][11]['pbrMetallicRoughness']['baseColorFactor']=[1.,.7,.2,1.]
        def uv_accessor(doc):doc['accessors'][57]['count']-=1
        def image_prefix(doc):doc['images'][0]['name']='overwritten'
        def manifest_change(data):
            doc=json.loads(data);doc['dawn']['generator_sha256']='corrupted';return json.dumps(doc).encode()
        tests=[
            ('other11 materials',target,lambda b:modify_json(other_material)),
            ('roughness12',target,lambda b:modify_json(target_roughness)),
            ('yellow factor',target,lambda b:modify_json(target_factor)),
            ('geometry BIN',target,change_geometry),
            ('UV accessor',target,lambda b:modify_json(uv_accessor)),
            ('existing images',target,lambda b:modify_json(image_prefix)),
            ('historical manifest',destination/'manifest.json',manifest_change),
            ('runtime PNG',destination/f'textures/{SOURCE_SHA}.png',lambda b:b[:-1]+bytes([b[-1]^1])),
            ('native author PNG',author/'source/ground-r60-native.png',lambda b:b[:-1]+bytes([b[-1]^1])),
            ('patch provenance',author/'patch_ground.py',lambda b:b+b'\n')]
        rejected=[]
        for name,file,change in tests:
            original=file.read_bytes();corrupt=change(original);assert corrupt!=original
            file.write_bytes(corrupt)
            check=verify();assert check.returncode!=0,name
            rejected.append(name);file.write_bytes(original)
        assert verify().returncode==0
report={'schema':1,'passed':True,'outside_workspace':True,'baselines':results,'corruptions_rejected':rejected,
    'script_sha256':sha(Path(__file__)),'native_gpu_compile':False,'canonical_mutations':False}
(AUTHOR/'proofs/relocation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
