"""Measure r47 honestly; report old interpolation gate separately, never waive it."""
import json,hashlib,struct,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parent))
from build_cloud_cache import ROOT,CONFIG,union
from cloud_cache_sampling import sample

def distribution(values):
    return {'count':len(values),'mean':float(np.mean(values)) if len(values) else None,
            'p50_p95_p99_max':np.quantile(values,[.5,.95,.99,1]).tolist() if len(values) else []}

def main():
    started=time.perf_counter();rng=np.random.default_rng(471003)
    config=json.loads(CONFIG.read_text());baseline=json.loads((ROOT/'tools/art/cloud-r47/archive/baseline/assets/atmosphere/cloud-banks.json').read_text())
    before=json.loads((ROOT/'tools/art/cloud-r47/archive/baseline/assets/atmosphere/cloud-cache-manifest.json').read_text());manifest=json.loads((ROOT/'assets/atmosphere/cloud-cache-manifest.json').read_text())
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    assert sha(CONFIG)==manifest['config_sha256'];assert sha(ROOT/manifest['generator'])==manifest['generator_sha256']
    report={'schema':1,'generator_sha256':manifest['generator_sha256'],'config_sha256':manifest['config_sha256'],'banks':[],'gpu_bytes':manifest['gpu_bytes'],'legacy_gate_unchanged':{'mean':.006,'p99':.035}}
    for bank,old,entry,old_entry in zip(config,baseline,manifest['banks'],before['banks'],strict=True):
        for key in ['key','center','extent','padding','shape']:assert bank[key]==old[key]
        raw=(ROOT/entry['file']).read_bytes();assert hashlib.sha256(raw).hexdigest()==entry['sha256']
        nx,ny,nz=bank['resolution'];offset,length,_=struct.unpack_from('<3Q',raw,80)
        cache=np.frombuffer(raw,dtype=np.uint8,count=length,offset=offset).reshape((nz,ny,nx,4))
        if bank['shape'][3]==0:assert bank['resolution']==old['resolution'] and entry['sha256']==old_entry['sha256']
        extent=np.array(bank['extent'],dtype=np.float32);center=np.array(bank['center'],dtype=np.float32)
        size=extent+2*np.array(bank['padding'],dtype=np.float32);dims=np.array([nx,ny,nz])
        p=(rng.random((36000,3),dtype=np.float32)-.5)*extent
        seconds=rng.random(len(p),dtype=np.float32)*14400
        p[:,0]+=np.sin(seconds*.00375)*192;p[:,2]+=np.sin(seconds*.0016145833)*192
        uv=p/size+.5;assert np.all((uv>=0)&(uv<=1))
        ref=union(p+center,bank);stored=sample(cache,uv)
        delta=np.abs(ref[:,0]-stored[:,0]);normal=stored[:,1:]*2-1
        normal/=np.maximum(np.linalg.norm(normal,axis=-1,keepdims=True),1e-20)
        angle=np.degrees(np.arccos(np.clip(np.sum(normal*ref[:,1:],axis=-1),-1,1)))
        ids=rng.integers([0,0,0],dims,size=(2048,3));centers=((ids+.5)/dims*size+center-size*.5).astype(np.float32)
        at_centers=union(centers,bank);encoded=at_centers.copy();encoded[:,1:]=encoded[:,1:]*.5+.5
        encoded=np.round(np.clip(encoded,0,1)*255).astype(np.uint8)
        exact=cache[ids[:,2],ids[:,1],ids[:,0]];assert np.array_equal(encoded,exact),'Actual cache must be exact function values at texel centers'
        visible=(ref[:,0]>.12)&(ref[:,0]<=.65)
        item={'key':bank['key'],'sha256':entry['sha256'],'dimensions':bank['resolution'],'center_extent_padding_shape_exact':True,
              'high_bank_byte_identical':bank['shape'][3]==0,'all':distribution(delta),
              'core_potential_gt_065':distribution(delta[ref[:,0]>.65]),
              'surface_potential_012_to_065':distribution(delta[visible]),
              'crevices_potential_lte_005':distribution(delta[ref[:,0]<=.05]),
              'normal_degrees_visible':distribution(angle[visible]),
              'normal_degrees_core':distribution(angle[ref[:,0]>.65]),
              'exact_quantized_texel_centers':len(ids),'advection_in_bounds':True,
              'legacy_interpolation_gate_pass':bool(np.mean(delta)<.006 and np.quantile(delta,.99)<.035)}
        # Surface normals describe this same scalar function, independently of
        # the cache. Constant clamped centers are excluded from this derivative.
        if bank['shape'][3]>=.5:
            chosen=np.flatnonzero((ref[:,0]>.12)&(ref[:,0]<.85))[:3000]
            q=p[chosen]+center;grad=[]
            for axis in range(3):
                v=np.zeros(3,dtype=np.float32);v[axis]=.25
                grad.append(-(union(q+v,bank)[:,0]-union(q-v,bank)[:,0])/.5)
            grad=np.stack(grad,axis=1);grad/=np.maximum(np.linalg.norm(grad,axis=-1,keepdims=True),1e-20)
            degrees=np.degrees(np.arccos(np.clip(np.sum(grad*ref[chosen,1:],axis=-1),-1,1)))
            item['analytic_normal_vs_finite_difference_degrees']=distribution(degrees)
            # Evaluate either side of the cell ownership boundary; density and
            # outward gradient must not jump when the 3x3 neighbourhood shifts.
            for axis in [0,2]:
                q=(rng.random((2500,3),dtype=np.float32)-.5)*extent+center
                q[:,axis]=np.round(q[:,axis]/240)*240
                d=np.zeros(3,dtype=np.float32);d[axis]=.01
                a,b=union(q-d,bank),union(q+d,bank)
                item['cell_boundary_axis_'+str(axis)]={'density_delta':distribution(np.abs(a[:,0]-b[:,0])),
                  'normal_delta_active':distribution(np.linalg.norm(a[:,1:]-b[:,1:],axis=1)[(a[:,0]>.12)&(a[:,0]<.85)])}
        worst=np.argsort(delta)[-12:][::-1]
        item['worst_samples']=[{'world':(p[i]+center).tolist(),'reference':float(ref[i,0]),'cached':float(stored[i,0]),'absolute_error':float(delta[i]),'normal_degrees':float(angle[i])} for i in worst]
        report['banks'].append(item)
        print(bank['key'],json.dumps({'mean':item['all']['mean'],'p99':item['all']['p50_p95_p99_max'][2],'normal_surface_p95':item['normal_degrees_visible']['p50_p95_p99_max'][1],'old_gate_pass':item['legacy_interpolation_gate_pass']}),flush=True)
    assert manifest['gpu_bytes']==57*1024*1024
    report['duration_seconds']=time.perf_counter()-started
    report['legacy_interpolation_gate_pass']=all(b['legacy_interpolation_gate_pass'] for b in report['banks'])
    (ROOT/'docs/evidence/cloud-r47-audit.json').write_text(json.dumps(report,indent=2)+'\n')

if __name__=='__main__':main()
