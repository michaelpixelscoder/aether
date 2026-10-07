"""Small isolated D relocation; exact native KTX/shader rebuild + strict tests."""
from pathlib import Path
import json,hashlib,tempfile,shutil,subprocess,sys,struct
import numpy as np
import build_mips
AUTHOR=Path(__file__).resolve().parent;ROOT=AUTHOR.parents[2];POOL=AUTHOR.parents[4]
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
assert POOL.resolve()==Path('D:/Aether-art-studies').resolve()
rectangles=[]
with tempfile.TemporaryDirectory(prefix='r64-cpu-relocation-',dir=POOL) as directory:
 temporary=Path(directory).resolve();assert temporary.parent==POOL.resolve() and temporary.name.startswith('r64-cpu-relocation-')
 relocated=temporary/'portable';shutil.copytree(ROOT,relocated,ignore=shutil.ignore_patterns('__pycache__'))
 author=relocated/'tools/art/flow-r64-source';before={name:sha(ROOT/name) for name in ['assets/textures/world-flow-current-r64.ktx2','assets/shaders/world-flow.wgsl']}
 for script,runner in [('record_source.py',sys.executable),('build_mips.py',sys.executable),('build_shader.py',sys.executable),('verify.mjs','node')]:
  result=subprocess.run([runner,str(author/script)],cwd=temporary,capture_output=True,text=True)
  assert result.returncode==0,(script,result.stdout[-500:],result.stderr[-1000:])
 assert all(sha(relocated/file)==value for file,value in before.items()),'New KTX and shader byte-exact portable reconstruction'
 assert sha(relocated/'assets/textures/world-flow.ktx2')=='8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973','Original cascade/pool asset unchanged'
 contracts=json.loads((author/'proofs/contracts.json').read_text());assert len(contracts['negative_cases_rejected'])==20
 # Generic encoder tests use only in-memory synthetic pixels; original source untouched.
 original_out=build_mips.OUT;build_mips.OUT=temporary/'rectangular-test.ktx2'
 try:
  for height,width in [(65,97),(64,128),(7,1),(1,7)]:
   original=np.broadcast_to(np.array([73,141,229,193],dtype=np.uint8),(height,width,4)).copy();levels=[original]
   while levels[-1].shape[:2]!=(1,1):levels.append(build_mips.downsample(levels[-1]))
   offsets=build_mips.write_ktx(levels);raw=build_mips.OUT.read_bytes()
   expected_count=max(width,height).bit_length();assert len(levels)==expected_count
   assert struct.unpack_from('<9I',raw,12)==(43,1,width,height,0,0,1,expected_count,0)
   for i,pixels in enumerate(levels):
    assert pixels.shape[:2]==(max(1,height//2**i),max(1,width//2**i));assert (pixels==original[0,0]).all()
    off,n,unpacked=struct.unpack_from('<3Q',raw,80+i*24);assert off==offsets[i] and off%8==0 and n==unpacked==pixels.nbytes
    assert raw[off:off+n]==pixels.tobytes()
   rectangles.append({'native_dimensions':[width,height],'mips':expected_count,'constant_rgba_exact':True,'index_offsets_payload_dimensions_exact':True})
  # A final odd edge must contribute its full area on either axis, never discarded.
  for axis in [0,1]:
   pixels=np.zeros((1,3,4) if axis==1 else (3,1,4),dtype=np.uint8)
   pixels[-1,-1]=[255,0,0,255];assert np.array_equal(build_mips.downsample(pixels)[0,0],[255,0,0,85])
 finally:build_mips.OUT=original_out
 report={'passed':True,'outside_project_D':True,'native_png_source_exact':True,'new_ktx_and_shader_byte_exact':True,'original_waterfall_pool_ktx_exact':True,'twenty_negative_contract_cases':contracts['negative_cases_rejected'],
  'rectangular_and_single_axis_tests':rectangles,'odd_last_edge_contributes_both_axes':True,'historical_four_encoder_tests':build_mips.tests(),'gpu_used':False,'canonical_mutations':False,'script_sha256':sha(Path(__file__))}
(AUTHOR/'proofs/relocation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
