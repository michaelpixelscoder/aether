import fs from 'node:fs';
import crypto from 'node:crypto';
const read=name=>JSON.parse(fs.readFileSync('docs/evidence/'+name,'utf8'));
const release=JSON.parse(fs.readFileSync('dist/latest.json','utf8'));
const native=read('native-benchmark.json'),chrome=read('chrome-benchmark.json'),edge=read('msedge-benchmark.json');
const qa=read('native-qa.json'),processes=read('native-process-measurements.json'),portable=read('portable-validation.json');
const cold=read('web-package-check.json'),sizes=read('package-sizes.json'),clean=read('clean-source.json'),gallery=read('gallery-check.json');
const soak=read('rendered-soak.json'),contrast=read('contrast.json'),art=read('art-assets.json');
const worldArt=read('open-world-assets.json'),tour=read('world-tour.json');
const worldModelCount=worldArt.islands.glbs.length+worldArt.variants.glbs.length+worldArt.vault.glbs.length+worldArt.kits.length;
const total=[...fs.readFileSync('docs/evidence/tests.log','utf8').matchAll(/test result: ok\. (\d+) passed/g)].reduce((n,m)=>n+Number(m[1]),0);
const browsers=['chrome','msedge'].map(name=>({name,...read(`browser-${name}.json`).stats}));
const wasmHash=crypto.createHash('sha256').update(fs.readFileSync(`dist/web/${release.id}/r/${release.id}/build/aether_game_bg.wasm`)).digest('hex');
const latency=read('edit-latency.json'),sail=read('sail-wind.json');
if(!soak.asset_files?.length)throw Error('Empreintes des actifs de l’endurance absentes');
for(const asset of soak.asset_files){
 const distributed=fs.readFileSync(`dist/web/${release.id}/r/${release.id}/assets/${asset.path}`);
 if(crypto.createHash('sha256').update(distributed).digest('hex')!==asset.sha256)throw Error('Actif modifié après endurance : '+asset.path);
}
const migration=cold.migration;
const migrationValid=migration?.old_save_format===1&&migration.new_save_format===2&&migration.refit_persisted&&migration.legacy_refusal?.phase==='Menu'&&migration.visited_builds.length===4&&migration.visited_builds[0]===migration.visited_builds[2]&&migration.visited_builds[1]===release.id&&migration.visited_builds[3]===release.id&&!migration.mixed_asset_versions;
if(total<124||qa.qa_failure||!qa.qa_finished||gallery.exit_code!==0||portable.exit_code!==0||clean.exit_code!==0||browsers.some(b=>b.unexpected||b.expected<16)||!soak.finished||soak.elapsed_seconds<1800||soak.errors.length||soak.wasm_sha256!==wasmHash||latency.wasm_sha256!==wasmHash||latency.samples_ms.length<20||latency.errors.length||sail.errors.length||sail.surfaces<3||!contrast.pass||!art.terrain_matches_domain||!migrationValid||cold.id!==release.id||portable.id!==release.id||worldModelCount!==34||worldArt.trader_boxes!==90||worldArt.sky?.status!=='passed'||!tour.finished||tour.reloaded?.checkpoint!==800||tour.errors.length||tour.wasm_sha256!==wasmHash)throw Error('Preuves finales incomplètes, périmées ou en échec');
const f=n=>n.toFixed(2).replace('.',','),mib=n=>f(n/1048576);
const row=(name,r,budget)=>`| ${name} | ${r.frame_ms_p50_p95_p99.map(f).join(' / ')} | ${r.tick_ms_p50_p95_p99.map(f).join(' / ')} | ${r.frame_ms_p50_p95_p99[1]<=budget?'Respecté':'Dépassé'} |`;
const memory=processes.find(r=>r.mode==='benchmark');
const gpu=Object.keys(native.render_pass_ms_p50_p95_p99||{}).filter(n=>n.endsWith('elapsed_gpu'));
let report=`# Validation du candidat ${release.id}

Rapport du ${new Date().toISOString().slice(0,10)}. Empreinte des sources : ${release.source_sha256}. Aucune opération Git, publication ou prise de contact avec des joueurs.

## Résultats

Windows et WebGPU compilent. Les neuf régions, la propulsion, les routes de vent, la récolte, le commerce, les portails et les observations de créatures prolongent la construction, le câble et la marche. Le [registre des 240 tâches](TASKS.md) conserve les identifiants du plan initial et distingue les extensions activées par la demande du monde ouvert. Les essais de compréhension/plaisir suivent [PLAYTEST](PLAYTEST.md), sans résultats inventés.

| Vérification | Preuve |
| --- | --- |
| Format, Clippy natif/WASM, builds optimisés | [Clippy natif](evidence/clippy.log), [WASM](evidence/wasm-clippy.log), [build Windows](evidence/native-build.log), [web](evidence/web-build.log) |
| ${total} tests Rust réussis | [Log](evidence/tests.log) |
| QA Windows : ${qa.checks.length} contrôles, 20 recréations et 100 éditions/annulations | [JSON](evidence/native-qa.json) |
| Chrome / Edge : ${browsers.map(b=>b.expected).join(' / ')} tests réussis | [Chrome](evidence/browser-chrome.json), [Edge](evidence/browser-msedge.json) |
| Endurance rendue : ${f(soak.elapsed_seconds/60)} minutes, ${soak.cycles.length} cycles sans erreur | [JSON et empreinte WASM](evidence/rendered-soak.json) |
| Contrastes ≥4,5:1, y compris les états des boutons et du focus | [Calcul sRGB](evidence/contrast.json) |
| ${art.models.length} GLB, manifeste terrain et générateur vérifiés | [Empreintes](evidence/art-assets.json) |
| ${worldModelCount} GLB supplémentaires, collisions et pivots animés vérifiés | [Monde, faune et navires](evidence/open-world-assets.json) |
| Récolte de cristal et d'eau, descente physique vers les cavernes et observation de deux espèces | [Touches réellement jouées et trace](evidence/world-tour.json) |
| Galerie de production indépendante | [Capture](evidence/gallery.png), [sortie 0](evidence/gallery-check.json) |
| Voile voxel : vent apparent, pause, reprise, meshes constants | [Mesures](evidence/sail-wind.json), [vidéo du jeu](evidence/sail-wind.webm) |
| Sources copiées hors workspace, check offline en ${f(clean.elapsed_seconds)} s | [Preuve](evidence/clean-source.json), cache de dépendances partagé |
| Archive Windows extraite hors workspace, ${portable.verified_files} fichiers vérifiés | [Portable](evidence/portable-validation.json) |
| Préfixe web, cache, compression, migration 1→2 et refus explicite du lecteur ancien | [Package](evidence/web-package-check.json) |

Les matrices comprennent pentes 15/30/39° à 30/60/120 Hz, jonctions de cellules, sommeil/réveil, 54 combinaisons du câble avec pause, navire chargé combinant voile/courant/câble/réserve, et interpolation effectivement activée. Le retard visuel autorisé d'un tick est mesuré en mètres puis remis à zéro aux téléportations. La QA rendue vérifie aussi les points d'équipements et le picking après reprise/scission.

Les interruptions de stockage sont injectées à douze frontières backup/principale ; ce ne sont pas des coupures électriques. Les navigateurs subissent quota, abort, backup, remappage, plein écran, historique, perte réelle du focus canvas et suspension JS de 65 secondes via CDP Debugger. Ni la veille Windows ni le bfcache ne sont certifiés. La fermeture Windows attend une sauvegarde, et un échec ne réemploie pas un ancien succès.

Le script QA court emploie des téléportations de préparation. La traversée physique est testée séparément aux intentions et aux touches. L'endurance rendue utilise les commandes clavier/souris et une sonde en lecture seule ; les scripts ne remplacent pas une observation humaine.

## Performance

Machine : Windows 11 Pro 26200, Ryzen 9 9950X3D, 64 Go, RTX 5080, pilote 32.0.15.9649. [Inventaire](evidence/environment.json), [navigateurs](evidence/browser-versions.json). Rust 1.98, Bevy 0.19 / internes 0.19.1, Avian 0.7, Tnua 0.32 / adaptateur 0.12. Navigateurs complets en headless accéléré D3D11/WebGPU ; ces options d'automatisation ne sont pas requises pour jouer.

Scène 1920×1080 : un corps dynamique de 10 000 cellules, 31 autres navires dynamiques, îles et effets. Mesure 35 s, premières cinq secondes exclues des frames. Physique conservée ; personnages invisibles des navires secondaires supprimés. WebAssembly optimisé pour la vitesse. Les boîtes de terrain coplanaires sont réunies à volume et intersections identiques, vérifiés sur 9 000 segments générés.

| Cible | Frame p50 / p95 / p99, ms | Tick p50 / p95 / p99, ms | Budget p95 |
| --- | ---: | ---: | --- |
${row('Windows',native,16.7)}
${row('Chrome',chrome,33.3)}
${row('Edge',edge,33.3)}

Objectifs : 16,7 ms natif, 33,3 ms web. [Windows](evidence/native-benchmark.json), [Chrome](evidence/chrome-benchmark.json), [Edge](evidence/msedge-benchmark.json) comprennent phases physiques, contacts, triangles, meshing CPU/application et délai de file. Les timestamps GPU Windows ont été reçus pour ${gpu.length} passes. Les valeurs GPU manquantes côté web ne sont pas extrapolées des FPS. Un intervalle de frame ou un délai de file n'est pas une mesure commande→photon ni une mesure isolée d'upload GPU.

Windows : working set échantillonné ${mib(memory.sampled_peak_working_set_bytes)} Mio, mémoire privée ${mib(memory.sampled_peak_private_bytes)} Mio. WASM : Chrome ${mib(chrome.wasm_linear_memory_bytes)} Mio, Edge ${mib(edge.wasm_linear_memory_bytes)} Mio. VRAM non isolée. Les vingt cycles natifs donnent ${Math.min(...qa.entity_samples_20_cycles)} à ${Math.max(...qa.entity_samples_20_cycles)} entités ; meshes, matériaux et file sont aussi contrôlés. Le test borné ne prétend pas prouver l'absence de toute fuite sur une durée infinie.

Vingt éditions après trois échauffements : entrée→lecture PNG locale p50/p95/p99 ${latency.p50_p95_p99_ms.map(f).join(' / ')} ms. La zone de 64×64 pixels entoure le point édité ; une capture complète est prise hors chronométrage. [Échantillons et séparation préparation/lecture](evidence/edit-latency.json). Le polling et la lecture du navigateur restent inclus : cette borne haute ne certifie pas la latence physique de l'écran. Les scénarios comparatifs et l'historique d'optimisation sont réunis dans [COMPARAISONS](COMPARAISONS.md).

## Distribution

Windows brut ${mib(sizes.native_raw_bytes)} Mio ; web brut ${mib(sizes.web_raw_bytes)} Mio, somme transférable Brotli ${mib(sizes.web_transfer_brotli_bytes)} Mio, notices comprises. [Inventaire](evidence/package-sizes.json). Cette somme diffère du trafic exact du premier écran.

Chrome à 50 Mbit/s descendant, 10 montant, 40 ms de latence simulée :

| Cache | Menu interactif, s | WASM, Mio |
| --- | ---: | ---: |
${cold.runs.map(r=>`| ${r.phase==='cold'?'Froid':'Chaud'} | ${f(r.first_interactive_seconds)} | ${mib(r.wasm_linear_memory_bytes)} |`).join('\n')}

Froid : nouveau contexte sans cache HTTP, sans purge des caches OS/GPU. Windows conserve le prérequis Visual C++ v14 x64, présent ici ; l'extraction portable n'est pas un essai sur Windows vierge. Manifestes SHA256 et identifiants immuables. Un ancien exécutable crée réellement une sauvegarde v1 ; le nouveau la migre, ajoute deux moteurs puis enregistre en v2 ; l'ancien refuse ce format et le nouveau retrouve la cargaison et la construction. Revenir au binaire 0.1 exige une sauvegarde v1 conservée séparément. Aucun hébergement public modifié. Voir [RELEASE](RELEASE.md), [CONTROLS](CONTROLS.md) et [ASSETS](ASSETS.md).

## Limites

Les retours spontanés de Damien sont consignés dans PLAYTEST ; les cinq observations indépendantes restent à réaliser. CI fournie mais non activée à distance. Le monde est borné et composé d'îles déterministes ; les cascades sont animées et leurs bassins récoltables, sans hydraulique volumique. Les créatures sont observables, sans combat, et le commerce s'effectue aux ports. Verre à tri alpha, voile par morph targets, câble sans enroulement, aucun multijoueur. Les résultats concernent la machine identifiée. La fidélité aux planches est jugée séparément dans [RENDU](RENDU.md) ; les succès techniques ne remplacent pas ce jugement. Arbitrages : [DECISIONS](DECISIONS.md).
`;
fs.writeFileSync('docs/VALIDATION.md',report);console.log(`${release.id} : ${total} tests Rust et ${soak.cycles.length} cycles d'endurance.`);
