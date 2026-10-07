"""Two-island candidate only. No rendering, no other island regeneration."""
import bpy,math,random,json,hashlib,types,importlib.util,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
EVIDENCE=HERE/'architecture-source/evidence';EVIDENCE.mkdir(parents=True,exist_ok=True)
def module(name,path):
    result=types.ModuleType(name);result.__file__=str(path)
    exec(compile(path.read_text(encoding='utf8'),str(path),'exec'),result.__dict__);return result
patch=module('r54_facades',HERE/'facades_r54.py')
ARCHIVE=HERE/'architecture-source/archive'
baseline_collisions=json.loads((ARCHIVE/'collisions.json').read_text())
baseline_landmarks=json.loads((ARCHIVE/'landmarks.json').read_text())
baseline_manifest=json.loads((ARCHIVE/'manifest.json').read_text())
baseline_routes=json.loads((ARCHIVE/'satellite-walk-routes.json').read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
records=[]
for kind in ['dawn','dawn-watch']:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if kind=='dawn':b=module('r54_base',HERE/'build_world.py');variant=None
    else:variant=module('r54_variant',HERE/'build_world_variants.py');b=variant.b
    metrics=patch.install(b);b.clear();b.CURRENT=kind;b.LOD=False;b.BATCH={};b.COLLISIONS=[];b.LANDMARKS={'waterfalls':[],'pools':[],'crystals':[],'portal':None}
    b.OUT=ROOT/'assets/world';b.SOURCE=ROOT/'tools/art/world-source';b.SOURCE.mkdir(parents=True,exist_ok=True)
    if variant:
        variant.KIND=kind;variant.RNG=random.Random(49281);b.RNG=variant.RNG;variant.ROUTES=[];variant.compose()
    else:b.RNG=random.Random(74019);b.compose(kind)
    assert b.COLLISIONS==baseline_collisions[kind],f'{kind}: changed collision'
    assert b.LANDMARKS==baseline_landmarks[kind],f'{kind}: changed landmarks'
    if variant:assert variant.ROUTES==baseline_routes[kind],f'{kind}: changed walking routes'
    objects=b.make_objects()+([] if variant else b.make_animated_objects())
    for image in bpy.data.images:
        if image.source=='FILE' and image.filepath:image.pack()
    source=b.SOURCE/(kind+'.blend');bpy.ops.wm.save_as_mainfile(filepath=str(source),compress=True)
    output=b.OUT/(kind+'.glb');result=b.write_glb(output,objects,kind)
    old=baseline_manifest[kind]['high']['triangles'];delta=result['triangles']-old
    assert delta<=12000,(kind,delta,result['triangles'],old)
    assert result['materials']==baseline_manifest[kind]['high']['materials']
    # The base generator's unchanged fingerprint stays truthful; explicitly add
    # the actual executable patch and wrapper rather than claiming base alone.
    raw=output.read_bytes();n=struct.unpack_from('<I',raw,12)[0];doc=json.loads(raw[20:20+n])
    doc['asset']['extras']['aether_architecture_r54']={'generator':'tools/art/build_facades_r54.py','generator_sha256':sha(Path(__file__)),'patch':'tools/art/facades_r54.py','patch_sha256':sha(HERE/'facades_r54.py'),'base_sha256':sha(HERE/'build_world.py'),'variant_sha256':sha(HERE/'build_world_variants.py') if variant else None,'scope':'house façades only; collision/landmarks/LOD unchanged'}
    encoded=json.dumps(doc,separators=(',',':')).encode();encoded+=b' '*((-len(encoded))%4)
    body=struct.pack('<II',len(encoded),0x4e4f534a)+encoded+raw[20+n:];output.write_bytes(struct.pack('<III',0x46546c67,2,len(body)+12)+body)
    result.update({'sha256':sha(output),'bytes':output.stat().st_size})
    records.append({'kind':kind,'houses':len(metrics),'before_triangles':old,'after':result,'triangle_delta':delta,'meshes':len(doc['meshes']),'nodes':len(doc['nodes']),'collision_exact':True,'landmarks_exact':True,'source_sha256':sha(source),'details':metrics})
    print('R54_FACADES',json.dumps({k:v for k,v in records[-1].items() if k!='details'}),flush=True)
(EVIDENCE/'facades-build.json').write_text(json.dumps({'schema':1,'candidate_only':True,'islands':records},indent=2)+'\n')
