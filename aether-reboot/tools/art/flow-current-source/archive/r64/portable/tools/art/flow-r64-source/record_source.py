"""Pin copied original art, prompt and actual native dimensions; no bitmap edit."""
from pathlib import Path
import hashlib,json,struct
AUTHOR=Path(__file__).resolve().parent;sha=lambda raw:hashlib.sha256(raw).hexdigest()
source=AUTHOR/'source/world-flow-current-native.png';raw=source.read_bytes()
expected='eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3'
assert sha(raw)==expected;assert raw[:8]==b'\x89PNG\r\n\x1a\n'
w,h,depth,color,compression,filtering,interlace=struct.unpack_from('>IIBBBBB',raw,16)
assert (depth,color,compression,filtering,interlace)==(8,6,0,0,0),'Preserve strict original RGBA8 contract'
prompt=(AUTHOR/'source/prompt.txt').read_bytes()
assert prompt.endswith(b'\n'),'Root reported exactly one added final LF in saved prompt'
invocation_prompt=prompt[:-1]
(AUTHOR/'source/prompt-invocation.txt').write_bytes(invocation_prompt)
metadata={'schema':1,'file':'tools/art/flow-r64-source/source/world-flow-current-native.png','sha256':expected,'bytes':len(raw),'width':w,'height':h,'channels':4,'bit_depth':8,
 'prompt_file':'tools/art/flow-r64-source/source/prompt.txt','prompt_sha256':sha(prompt),'prompt_bytes':len(prompt),'tool':'image_gen.imagegen','built_in':True,
 'invocation_prompt_file':'tools/art/flow-r64-source/source/prompt-invocation.txt','invocation_prompt_sha256':sha(invocation_prompt),'invocation_prompt_bytes':len(invocation_prompt),'saved_prompt_difference':'Root saved the exact in-memory tool prompt plus one final LF; only that LF removed in separate invocation text',
 'generation_parameters':{'transparent_background':True},'omitted_parameters':['referenced_image_paths','num_last_images_to_include'],
 'original_generated_location':'C:/Users/Damien/.codex/generated_images/01a0f81f-371e-7440-ba13-362c122d50e5/exec-cd3181e2-4abb-4bec-8cf7-324b919ffef8.png',
 'generated_at_approx_utc':'2026-10-03 10:35 UTC','tool_duration_seconds_root_reported':53.7,'metadata_authority':'Exact invocation reported by root; no inferred image-edit parameters',
 'native_pixels_untouched':True,'edit_or_bitmap_remap':False,
 'source_from_root':'C:/Users/Damien/Documents/Perso/Game/Aether-git/aether-reboot/.dream-loop/flow-r64-original/world-flow-current-native.png','external_path_role':'provenance only; all builds read the portable source'}
(AUTHOR/'source/native-r64.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8');print(json.dumps(metadata))
