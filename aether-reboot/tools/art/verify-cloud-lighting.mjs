// Current cloud gate: preserve R55 history and independently validate R57 radiance.
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {verify as historical} from './cloud-r57/verify-r55-historical.mjs';
import {verify as current} from './verify-cloud-r57-lighting.mjs';
const defaultRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
export function verify(root=defaultRoot,{write=true}={}) {
 historical(root,{write:false});
 return current(root,{write});
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))verify();
