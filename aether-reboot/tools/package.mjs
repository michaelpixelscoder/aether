import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import zlib from 'node:zlib';import {spawnSync} from 'node:child_process';
const root=process.cwd(),hash=data=>crypto.createHash('sha256').update(data).digest('hex');
const metadata=spawnSync('cargo',['metadata','--locked','--format-version','1'],{encoding:'utf8',maxBuffer:32*1024*1024});if(metadata.status!==0)throw Error(metadata.stderr);
const workspace=JSON.parse(metadata.stdout),version=workspace.packages.find(p=>p.name==='aether_game'&&!p.source).version;
const saveFormat=Number(fs.readFileSync('crates/aether_core/src/save.rs','utf8').match(/pub const VERSION: u32 = (\d+);/)[1]);
const walk=dir=>fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(path.join(dir,e.name)):[path.join(dir,e.name)]).sort();
const relative=file=>path.relative(root,file).replaceAll('\\','/');
const toolSources=fs.readdirSync('tools').filter(n=>/\.(mjs|ps1|json)$/.test(n)).map(n=>path.join('tools',n));
const sources=['Cargo.toml','Cargo.lock','rust-toolchain.toml',...walk('crates'),...walk('web'),...walk('assets'),...toolSources,...walk('tools/e2e'),...walk('tools/art').filter(p=>!p.endsWith('.blend1')&&!p.split(path.sep).includes('__pycache__')),'.cargo/config.toml'].sort();
const sourceManifest=sources.map(file=>({path:relative(path.resolve(file)),bytes:fs.statSync(file).size,sha256:hash(fs.readFileSync(file))}));
const sourceHash=hash(JSON.stringify(sourceManifest));const id=`${version}-${sourceHash.slice(0,12)}`;
const binary='target/release/aether_game.exe',bundle='target/bevy_web/web-release/aether_game';
for(const file of [binary,path.join(bundle,'build/aether_game_bg.wasm')])if(!fs.existsSync(file))throw Error('Build release absent: '+file);
const rustTime=Math.max(...sources.filter(p=>(p.endsWith('.rs')&&p.replaceAll('\\','/').includes('/src/'))||p.endsWith('Cargo.toml')||p.endsWith('Cargo.lock')).map(p=>fs.statSync(p).mtimeMs));
if(fs.statSync(binary).mtimeMs<rustTime||fs.statSync(path.join(bundle,'build/aether_game_bg.wasm')).mtimeMs<rustTime)throw Error('Les sources sont plus récentes que le build. Relancer package.ps1 sans -SkipBuild.');
for(const file of walk('assets')){
 const target=path.join(bundle,file);if(!fs.existsSync(target)||hash(fs.readFileSync(file))!==hash(fs.readFileSync(target)))throw Error('Asset web périmé: '+file);
}
for(const file of walk('web')){
 const target=path.join(bundle,path.relative('web',file));if(!fs.existsSync(target)||hash(fs.readFileSync(file))!==hash(fs.readFileSync(target)))throw Error('Page web périmée: '+file);
}
const native=path.join('dist','windows',id),web=path.join('dist','web',id),versioned=path.join(web,'r',id);
if(fs.existsSync(native)||fs.existsSync(web))throw Error('Ce contenu a déjà un package. Conserver son manifeste ou changer les sources, sans écraser une livraison.');
fs.mkdirSync(native,{recursive:true});fs.mkdirSync(versioned,{recursive:true});
fs.copyFileSync(binary,path.join(native,'Aether-Isles.exe'));fs.cpSync('assets',path.join(native,'assets'),{recursive:true});fs.cpSync(bundle,versioned,{recursive:true});
const launcher=`<!doctype html><html lang="fr"><meta charset="utf-8"><title>Aether Isles</title><p>Ouverture du jeu… <a href="./r/${id}/">Continuer</a></p><script>location.replace('./r/${id}/'+location.search+location.hash)</script></html>`;
fs.writeFileSync(path.join(web,'index.html'),launcher);
const packages=workspace.packages.filter(p=>p.source);const notices=[];
for(const destination of [native,versioned]){
 for(const file of ['LICENSE-MIT','LICENSE-APACHE','docs/ASSETS.md','docs/CONTROLS.md'])if(fs.existsSync(file))fs.copyFileSync(file,path.join(destination,path.basename(file)));
 const licenses=path.join(destination,'THIRD_PARTY_LICENSES');fs.mkdirSync(licenses,{recursive:true});
 for(const pkg of packages){
  const directory=path.dirname(pkg.manifest_path),prefix=`${pkg.name}-${pkg.version}`;
  const files=fs.readdirSync(directory).filter(n=>/^(license|licence|copying|notice)([-.]|$)/i.test(n)&&fs.statSync(path.join(directory,n)).isFile());
  if(pkg.license_file&&fs.existsSync(pkg.license_file))files.push(path.relative(directory,pkg.license_file));
  for(const file of new Set(files))fs.copyFileSync(path.join(directory,file),path.join(licenses,prefix+'-'+path.basename(file)));
  if(destination===native)notices.push({name:pkg.name,version:pkg.version,license:pkg.license,repository:pkg.repository,license_files:files.map(f=>prefix+'-'+path.basename(f))});
 }
 fs.writeFileSync(path.join(destination,'THIRD_PARTY.json'),JSON.stringify(notices,null,2));
 fs.writeFileSync(path.join(destination,'BUILD.json'),JSON.stringify({id,version,built_at:new Date().toISOString(),source_sha256:sourceHash,rust:'1.98.0',save_format:saveFormat,git_commands_used:false},null,2));
}
fs.writeFileSync(path.join(native,'LIRE-MOI.txt'),`Aether Isles ${version}\r\nExtraire tout le ZIP, puis lancer Aether-Isles.exe. Conserver assets à côté.\r\nPrérequis Windows : runtime Microsoft Visual C++ v14 x64 (VCRUNTIME140.dll). Voir CONTROLS.md.\r\nClavier : F départ, Z/W/flèche haut moteur, Q/A/D direction, E/C altitude, S frein, M atlas, B interaction.\r\nTab atelier, G harpon, T marche, R secours. Ctrl+S sauvegarde ; L reprise ; Échap pause. Voir CONTROLS.md.\r\nLes sauvegardes v1 sont migrées en v${saveFormat}. Une sauvegarde v${saveFormat} ne peut pas être ouverte dans l'ancien jeu.\r\nLes données sont dans %LOCALAPPDATA%/AetherIsles/Reboot/data (AETHER_DATA peut remplacer ce chemin).\r\n`);
function manifest(directory,compress){
 for(const file of walk(directory))if(fs.statSync(file).mtime.getFullYear()<1980){const zipEpoch=new Date('1980-01-01T00:00:00Z');fs.utimesSync(file,zipEpoch,zipEpoch);}
 let files=walk(directory).map(file=>{const bytes=fs.readFileSync(file);const item={path:path.relative(directory,file).replaceAll('\\','/'),bytes:bytes.length,sha256:hash(bytes),transfer_bytes:bytes.length};
  if(compress&&/\.(wasm|js|html|json|txt|ttf|glb|wav|ktx2|wgsl)$/i.test(file)){
   const gz=zlib.gzipSync(bytes,{level:9}),br=zlib.brotliCompressSync(bytes,{params:{[zlib.constants.BROTLI_PARAM_QUALITY]:6}});
   fs.writeFileSync(file+'.gz',gz);fs.writeFileSync(file+'.br',br);item.gzip_bytes=gz.length;item.brotli_bytes=br.length;item.transfer_bytes=br.length;
  }return item;});
 const result={id,source_sha256:sourceHash,files,total_raw_bytes:files.reduce((s,f)=>s+f.bytes,0),total_transfer_bytes:files.reduce((s,f)=>s+f.transfer_bytes,0)};
 fs.writeFileSync(path.join(directory,'MANIFEST.json'),JSON.stringify(result,null,2));return result;
}
const nativeManifest=manifest(native,false),webManifest=manifest(web,true);
fs.writeFileSync('dist/latest.json',JSON.stringify({id,windows:native,web,web_entry:`r/${id}/`,source_sha256:sourceHash},null,2));
fs.writeFileSync('docs/evidence/source-manifest.json',JSON.stringify(sourceManifest,null,2));
fs.writeFileSync('docs/evidence/package-sizes.json',JSON.stringify({id,native_raw_bytes:nativeManifest.total_raw_bytes,web_raw_bytes:webManifest.total_raw_bytes,web_transfer_brotli_bytes:webManifest.total_transfer_bytes,largest_web_files:[...webManifest.files].sort((a,b)=>b.bytes-a.bytes).slice(0,10)},null,2));
console.log(JSON.stringify({id,native,web,sourceHash},null,2));
