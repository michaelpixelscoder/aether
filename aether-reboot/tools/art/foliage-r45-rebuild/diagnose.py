"""Read-only four-thread diagnosis of one newly observed tangent threshold."""
import bpy,numpy as np,json,hashlib,sys
from pathlib import Path
DIR=Path(__file__).resolve().parent;ROOT=DIR.parents[2]
BEFORE=Path(sys.argv[sys.argv.index('--before-root')+1]) if '--before-root' in sys.argv else ROOT.parents[1]
report={'blender':bpy.app.version_string,'pairs':[],'repeats':[]};samples={}
for label,folder in [('before',BEFORE/'tools/art/world-source'),('after',ROOT/'tools/art/world-source')]:
    bpy.ops.wm.open_mainfile(filepath=str(folder/'dawn-garden.blend'))
    mesh=bpy.data.objects['08 | Deep foliage'].data;mesh.calc_tangents(uvmap=mesh.uv_layers.active.name)
    arrays={}
    for name,length,collection,attribute,dtype in [
        ('positions',len(mesh.vertices)*3,mesh.vertices,'co',np.float32),
        ('indices',len(mesh.loops),mesh.loops,'vertex_index',np.int32),
        ('uv',len(mesh.loops)*2,mesh.uv_layers.active.uv,'vector',np.float32),
        ('normal',len(mesh.loops)*3,mesh.corner_normals,'vector',np.float32),
        ('tangent',len(mesh.loops)*3,mesh.loops,'tangent',np.float32)]:
        data=np.empty(length,dtype=dtype);collection.foreach_get(attribute,data);arrays[name]=data
    samples[label]=arrays
for name,a in samples['before'].items():
    b=samples['after'][name];assert a.shape==b.shape
    report['pairs'].append({'attribute':name,'components':len(a),'different':int(np.count_nonzero(a!=b)),'max_abs':float(np.max(np.abs(a.astype(np.float64)-b.astype(np.float64)))),'before_sha256':hashlib.sha256(a).hexdigest(),'after_sha256':hashlib.sha256(b).hexdigest()})
    if name!='tangent':assert np.array_equal(a,b),name
for repeat in range(12):
    copy=mesh.copy();copy.calc_tangents(uvmap=copy.uv_layers.active.name)
    t=np.empty(len(copy.loops)*3,dtype=np.float32);copy.loops.foreach_get('tangent',t)
    n=np.empty(len(copy.loops)*3,dtype=np.float32);copy.corner_normals.foreach_get('vector',n)
    assert np.array_equal(n,samples['after']['normal'])
    # Every near-threshold negative-X tangent is reported, so loop/corner
    # ordering can be independently linked to the explicit GLB corner proof.
    ix=np.flatnonzero(np.abs(t+.15315)<.00000015)
    report['repeats'].append({'raw_sha256':hashlib.sha256(t).hexdigest(),'normal_sha256':hashlib.sha256(n).hexdigest(),'max_abs_from_first':float(np.max(np.abs(t-samples['after']['tangent']))),'threshold_samples':[{'loop':int(i//3),'component':int(i%3),'raw':float(t[i]),'round4':float(np.round(t[i],4))} for i in ix]})
    bpy.data.meshes.remove(copy)
(DIR/'diagnosis.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print('R45 raw mesh inputs exact; repeated tangent threshold recorded',flush=True)
