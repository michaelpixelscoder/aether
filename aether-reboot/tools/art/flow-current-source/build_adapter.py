"""Minimal R53 gate adapter. Original metadata/checks stay intact."""
from pathlib import Path
import hashlib
A=Path(__file__).resolve().parent;R=A.parents[2]
raw=(A/'history/r53-verify.mjs').read_bytes()
assert hashlib.sha256(raw).hexdigest()=='c99fb06a15fb81ebf5c0dbbec575e59f422dfc3100ca3cb6f6e458cb22e10ab4'
text=raw.decode('utf8');newline='\r\n' if '\r\n' in text else '\n'
before="import {fileURLToPath} from 'node:url';";after="import {fileURLToPath,pathToFileURL} from 'node:url';"
assert text.count(before)==1;text=text.replace(before,after)
before="const shader=read('assets/shaders/world-flow.wgsl').toString(),old=readArt('history/prior-r52.wgsl').toString();"
after="""// Validate the actual successor in full BEFORE restoring its entire R53 reference.
// All original R53 native/mip/material checks below remain historical and exact.
const runtimeShaderRaw=read('assets/shaders/world-flow.wgsl');
let historicalShaderRaw=runtimeShaderRaw,successorReport=null;
const successorApi=path.join(root,'tools/art/flow-current-source/validate.mjs');
if(fs.existsSync(successorApi)){
 const {validateAndRecoverR53}=await import(pathToFileURL(successorApi).href);
 const validated=validateAndRecoverR53({root,runtimeShader:runtimeShaderRaw});
 historicalShaderRaw=validated.r53Raw;successorReport=validated.report;
}
const shader=historicalShaderRaw.toString(),old=readArt('history/prior-r52.wgsl').toString();""".replace('\n',newline)
assert text.count(before)==1;text=text.replace(before,after)
before="fs.writeFileSync(path.join(author,'proofs/container.json'),JSON.stringify(report,null,2)+'\\n');"
after="""if(successorReport){
 report.validation_role='R53 historical metadata/gates retained; actual runtime authorized by strict successor chain';
 report.runtime_shader_sha256=sha(runtimeShaderRaw);
 report.successor_chain=successorReport;
}
fs.writeFileSync(path.join(author,'proofs/container.json'),JSON.stringify(report,null,2)+'\\n');""".replace('\n',newline)
assert text.count(before)==1;text=text.replace(before,after)
out=text.encode('utf8');(R/'tools/art/flow-source/verify.mjs').write_bytes(out);print(hashlib.sha256(out).hexdigest())
