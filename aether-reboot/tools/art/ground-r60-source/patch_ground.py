"""R60 PNG-only glTF material12 delta; preserve caller's GLB BIN/geometry/art."""
from pathlib import Path
import argparse,copy,struct,json,hashlib

AUTHOR=Path(__file__).resolve().parent
NATIVE=AUTHOR/'source/ground-r60-native.png'
SOURCE_SHA='087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365'
FILES=['dawn.glb','dawn-lod.glb','dawn-watch.glb','dawn-watch-lod.glb']
UV_SCALE=2.1/(4*.11)
FACTOR=[1.,1.,1.,1.]
IMAGE_URI=f'textures/{SOURCE_SHA}.png'

def sha(data):return hashlib.sha256(data).hexdigest()

def parse(raw):
    assert len(raw)>=28 and struct.unpack_from('<III',raw)==(0x46546c67,2,len(raw))
    size,kind=struct.unpack_from('<II',raw,12)
    assert kind==0x4e4f534a
    doc=json.loads(raw[20:20+size])
    at=20+size;bin_size,bin_kind=struct.unpack_from('<II',raw,at)
    assert bin_kind==0x004e4942 and at+8+bin_size==len(raw)
    assert len(doc['buffers'])==1 and 'uri' not in doc['buffers'][0]
    return doc,raw[at+8:]

def pack(doc,binary):
    encoded=json.dumps(doc,separators=(',',':'),ensure_ascii=False).encode()
    encoded+=b' '*((-len(encoded))%4)
    assert len(binary)%4==0
    return struct.pack('<IIIII',0x46546c67,2,28+len(encoded)+len(binary),len(encoded),0x4e4f534a)+encoded+struct.pack('<II',len(binary),0x004e4942)+binary

def target_material(doc):
    indices=[i for i,m in enumerate(doc['materials']) if m.get('name','').startswith('12 | Surface biome')]
    assert len(indices)==1,indices
    return indices[0]

def patch(raw):
    original,binary=parse(raw)
    doc=copy.deepcopy(original)
    index=target_material(doc)
    material=doc['materials'][index]
    pbr=material['pbrMetallicRoughness']
    previous_info=pbr['baseColorTexture']
    previous_texture=doc['textures'][previous_info['index']]
    assert 'extensions' not in previous_texture,'Unsupported basis texture; refuse rather than overwrite caller extensions'
    assert not any(image.get('uri')==IMAGE_URI for image in doc['images']),'Already patched; refuse stacking duplicate textures'
    image_index=len(doc['images'])
    doc['images'].append({'mimeType':'image/png','name':'R60 natural grass moss native','uri':IMAGE_URI})
    texture_index=len(doc['textures'])
    texture=copy.deepcopy(previous_texture)
    texture.update(source=image_index,name='R60 grass moss original PNG')
    doc['textures'].append(texture)
    info=copy.deepcopy(previous_info)
    info['index']=texture_index
    extensions=info.setdefault('extensions',{})
    assert 'KHR_texture_transform' not in extensions,'Unexpected caller transform; requires explicit migration'
    extensions['KHR_texture_transform']={'scale':[UV_SCALE,UV_SCALE]}
    pbr['baseColorTexture']=info
    pbr['baseColorFactor']=FACTOR
    used=doc.setdefault('extensionsUsed',[])
    if 'KHR_texture_transform' not in used:used.append('KHR_texture_transform')
    candidate=pack(doc,binary)
    check_delta(raw,candidate)
    return candidate

def check_delta(before,after):
    original,binary=parse(before);doc,next_binary=parse(after)
    assert binary==next_binary,'Entire BIN must remain byte-exact'
    index=target_material(original)
    assert index==target_material(doc)
    assert len(original['materials'])==len(doc['materials'])
    for i,(a,b) in enumerate(zip(original['materials'],doc['materials'])):
        if i!=index:assert a==b,('non12 material mutated',i)
    stripped=copy.deepcopy(doc)
    stripped['materials'][index]=copy.deepcopy(original['materials'][index])
    assert len(doc['images'])==len(original['images'])+1
    assert doc['images'][:-1]==original['images']
    assert doc['images'][-1]=={'mimeType':'image/png','name':'R60 natural grass moss native','uri':IMAGE_URI}
    stripped['images']=copy.deepcopy(original['images'])
    assert len(doc['textures'])==len(original['textures'])+1
    assert doc['textures'][:-1]==original['textures']
    stripped['textures']=copy.deepcopy(original['textures'])
    expected_extensions=copy.deepcopy(original.get('extensionsUsed',[]))
    if 'KHR_texture_transform' not in expected_extensions:expected_extensions.append('KHR_texture_transform')
    assert doc['extensionsUsed']==expected_extensions
    if 'extensionsUsed' in original:stripped['extensionsUsed']=original['extensionsUsed']
    else:stripped.pop('extensionsUsed')
    assert stripped==original,'Every other JSON field, including extras/history/nodes/UV references, remains exact'
    material=copy.deepcopy(doc['materials'][index])
    pbr=material['pbrMetallicRoughness']
    assert pbr['baseColorFactor']==FACTOR
    info=pbr['baseColorTexture']
    assert info['index']==len(original['textures'])
    assert info['extensions']['KHR_texture_transform']=={'scale':[UV_SCALE,UV_SCALE]}
    pbr['baseColorTexture']=copy.deepcopy(original['materials'][index]['pbrMetallicRoughness']['baseColorTexture'])
    if 'baseColorFactor' in original['materials'][index]['pbrMetallicRoughness']:
        pbr['baseColorFactor']=original['materials'][index]['pbrMetallicRoughness']['baseColorFactor']
    else:pbr.pop('baseColorFactor')
    assert material==original['materials'][index],'Only target albedo binding/factor/transform may change'
    return index

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--input-world',type=Path,required=True)
    parser.add_argument('--output-world',type=Path,required=True)
    parser.add_argument('--report',type=Path)
    args=parser.parse_args()
    assert args.input_world.resolve()!=args.output_world.resolve(),'Never patch caller inputs in place'
    assert sha(NATIVE.read_bytes())==SOURCE_SHA
    args.output_world.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((args.input_world/'manifest.json').read_text(encoding='utf-8-sig'))
    original_manifest=copy.deepcopy(manifest)
    entries=[]
    for filename in FILES:
        raw=(args.input_world/filename).read_bytes()
        candidate=patch(raw)
        (args.output_world/filename).write_bytes(candidate)
        kind=filename.removesuffix('.glb').removesuffix('-lod')
        level='lod' if filename.endswith('-lod.glb') else 'high'
        manifest[kind][level]['bytes']=len(candidate)
        manifest[kind][level]['sha256']=sha(candidate)
        entries.append({'file':filename,'input_sha256':sha(raw),'output_sha256':sha(candidate),'bytes':len(candidate),'bin_byte_exact':True,'other_materials_and_all_meshes_exact':True})
    api_sha='6ff191df9ff34a8d5c5283ce111cb5db1c95088ba78cbb1827abc465130f6c8c'
    assert sha((AUTHOR/'validate-ground-delta.mjs').read_bytes())==api_sha
    for kind in ['dawn','dawn-watch']:
        assert 'ground_r60' not in original_manifest[kind],'Already versioned ground provenance'
        high=next(e for e in entries if e['file']==kind+'.glb')
        lod=next(e for e in entries if e['file']==kind+'-lod.glb')
        manifest[kind]['ground_r60']={'source_png_sha256':SOURCE_SHA,'runtime_texture_uri':IMAGE_URI,
            'native_dimensions':[1254,1254],'base_color_factor':FACTOR,'uv_transform_scale':[UV_SCALE,UV_SCALE],
            'primary_instance_scale':2.1,'primary_physical_tile_m':4.0,
            'input_high_sha256':high['input_sha256'],'input_lod_sha256':lod['input_sha256'],
            'author':'tools/art/ground-r60-source/patch_ground.py','author_sha256':sha(Path(__file__).read_bytes()),
            'delta_validator':'tools/art/ground-r60-source/validate-ground-delta.mjs','delta_validator_sha256':api_sha,
            'scope':'Material12 albedo binding/factor/texture transform only; BIN/geometry/UV/all unrelated material/images/history JSON exact'}
    recovered=copy.deepcopy(manifest)
    for kind in ['dawn','dawn-watch']:
        recovered[kind].pop('ground_r60')
        for level in ['high','lod']:
            recovered[kind][level]['bytes']=original_manifest[kind][level]['bytes']
            recovered[kind][level]['sha256']=original_manifest[kind][level]['sha256']
    assert recovered==original_manifest,'Only four real file bytes/SHA metadata fields may change'
    (args.output_world/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    texture=args.output_world/IMAGE_URI;texture.parent.mkdir(exist_ok=True)
    texture.write_bytes(NATIVE.read_bytes())
    report={'schema':1,'source_png_sha256':SOURCE_SHA,'patch_source_sha256':sha(Path(__file__).read_bytes()),'uv_transform_scale':UV_SCALE,'base_color_factor':FACTOR,'files':entries,'historical_manifest_metadata_exact':True,'native_gpu_compile':'PENDING','input_baseline_agnostic':True}
    if args.report:args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
