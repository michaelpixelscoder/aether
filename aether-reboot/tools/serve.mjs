import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
const root=path.resolve(process.argv[2]||'target/bevy_web/web/aether_game');
const port=Number(process.argv[3]||4173);
const types={'.html':'text/html; charset=utf-8','.js':'text/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png','.glb':'model/gltf-binary','.ttf':'font/ttf','.wav':'audio/wav'};
http.createServer((req,res)=>{
  let pathname;
  try{pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);}catch{res.writeHead(400).end();return;}
  const file=path.resolve(root,'.'+(pathname.endsWith('/')?pathname+'index.html':pathname));
  if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  fs.stat(file,(error,stat)=>{
    if(error||!stat.isFile()){res.writeHead(404).end('Fichier absent');return;}
    const accepted=req.headers['accept-encoding']||'';
    const encoding=accepted.includes('br')&&fs.existsSync(file+'.br')?'br':accepted.includes('gzip')&&fs.existsSync(file+'.gz')?'gzip':null;
    const served=file+(encoding==='br'?'.br':encoding==='gzip'?'.gz':'');
    const headers={'Content-Type':types[path.extname(file)]||'application/octet-stream','Cache-Control':/\/r\/\d+\.\d+\.\d+-[a-f0-9]{12}\//.test(pathname)?'public, max-age=31536000, immutable':'no-store','X-Content-Type-Options':'nosniff','Vary':'Accept-Encoding','Content-Length':fs.statSync(served).size};
    if(encoding)headers['Content-Encoding']=encoding;
    res.writeHead(200,headers);
    fs.createReadStream(served).pipe(res);
  });
}).listen(port,'127.0.0.1',()=>console.log(`Aether : http://127.0.0.1:${port} (${root})`));
