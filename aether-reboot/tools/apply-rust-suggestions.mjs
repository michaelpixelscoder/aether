// Apply only rustc/Clippy MachineApplicable source suggestions inside this workspace.
// This helper makes no version-control calls.
import fs from 'node:fs';import path from 'node:path';import {spawnSync} from 'node:child_process';
const root=process.cwd();
for(let round=0;round<8;round++){
 const result=spawnSync('cargo',['clippy','--workspace','--all-targets','--locked','--message-format=json','--','-D','warnings'],{encoding:'utf8',maxBuffer:32*1024*1024});
 if(result.status===0){console.log('Clippy passed.');process.exit(0);}
 const changes=new Map();let count=0;
 for(const line of result.stdout.split('\n')){let item;try{item=JSON.parse(line);}catch{continue;}if(item.reason!=='compiler-message')continue;
  console.log(item.message.rendered||item.message.message);
  for(const child of item.message.children||[]){const spans=child.spans||[];
   if(!spans.length||spans.some(s=>s.suggestion_applicability!=='MachineApplicable'||typeof s.suggested_replacement!=='string'))continue;
   for(const span of spans){const file=path.resolve(root,span.file_name);if(!file.startsWith(path.join(root,'crates')+path.sep))throw Error('Suggestion outside crates: '+file);
    const list=changes.get(file)||[];if(list.some(s=>s.byte_start<span.byte_end&&span.byte_start<s.byte_end))continue;list.push(span);changes.set(file,list);count++;
   }
  }
 }
 if(!count){console.error(result.stderr);process.exit(result.status||1);}
 for(const [file,spans] of changes){let bytes=fs.readFileSync(file);for(const s of spans.sort((a,b)=>b.byte_start-a.byte_start))bytes=Buffer.concat([bytes.subarray(0,s.byte_start),Buffer.from(s.suggested_replacement),bytes.subarray(s.byte_end)]);fs.writeFileSync(file,bytes);}
 console.log(`Applied ${count} safe suggestions in round ${round+1}.`);
}
process.exit(1);
