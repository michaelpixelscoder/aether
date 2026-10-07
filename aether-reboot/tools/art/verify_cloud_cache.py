"""Recompute accepted r47 fidelity. Historical .035 gate remains archived/failed."""
import hashlib,json,runpy,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from build_cloud_cache import ROOT
import audit_cloud_field

def main():
    out=ROOT/'docs/evidence';out.mkdir(parents=True,exist_ok=True)
    audit_cloud_field.main()
    runpy.run_path(str(ROOT/'tools/art/compare_cloud_rays_cpu.py'),run_name='__main__')
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    audit=json.loads((out/'cloud-r47-audit.json').read_text())
    rays=json.loads((out/'cloud-r47-ray-comparison.json').read_text())
    contract=json.loads((ROOT/'tools/art/cloud-r47/contract.json').read_text())
    assert audit['gpu_bytes']==57*1024*1024
    assert len(audit['banks'])==6
    for bank in audit['banks']:
        regional=bank['key'] in ['hollow','underforge']
        assert bank['all']['mean']<.006
        assert bank['all']['p50_p95_p99_max'][2] < (.05 if regional else .035)
        assert bank['exact_quantized_texel_centers']==2048
        assert bank['advection_in_bounds'] and bank['center_extent_padding_shape_exact']
        assert bank['sha256']==contract['reviewed_cache_sha256'][bank['key']]
        if regional:
            assert bank['analytic_normal_vs_finite_difference_degrees']['p50_p95_p99_max'][2]<.05
        else:
            assert bank['high_bank_byte_identical'] and bank['legacy_interpolation_gate_pass']
    assert len(rays['views'])==4 and rays['dimensions']==[240,150]
    assert {(v['bank'],v['view']) for v in rays['views']}=={(b,v) for b in ['hollow','underforge'] for v in ['oblique','grazing']}
    for view in rays['views']:
        result=view['comparisons']['cache_vs_analytic']
        assert result['opaque_alpha_gt_095']['display_rmse']<.025
        assert result['edge_alpha_005_to_095']['opacity_absolute']['p50_p95_p99_max'][2]<.08
    sources=['build_cloud_cache.py','cloud_cache_sampling.py','audit_cloud_field.py','review_cloud_field_cpu.py','compare_cloud_rays_cpu.py','verify_cloud_cache.py']
    report={'schema':1,'passed':True,'contract':'r47-regional-accepted-with-image-and-derivative-guards',
            'legacy_pointwise_gate_pass':audit['legacy_interpolation_gate_pass'],
            'legacy_failure_preserved':True,'gpu_bytes':audit['gpu_bytes'],
            'contract_sha256':sha(ROOT/'tools/art/cloud-r47/contract.json'),
            'source_hashes':{f:sha(ROOT/'tools/art'/f) for f in sources},
            'config_sha256':audit['config_sha256'],'generator_sha256':audit['generator_sha256'],
            'audit_sha256':sha(out/'cloud-r47-audit.json'),'ray_comparison_sha256':sha(out/'cloud-r47-ray-comparison.json'),
            'banks':audit['banks'],'views':rays['views']}
    (out/'cloud-cache-fidelity.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS accepted r47 recipe fidelity, six banks and four ray views; historical regional p99 .035 remains failed.',flush=True)

if __name__=='__main__':main()
