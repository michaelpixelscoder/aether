import fs from 'node:fs';
// Each assessment is deliberate. Never infer completion from the presence of a file.
const groups={
GOV:[
'Adapté|Références copiées ; empreinte SHA256 des sources livrées. Inspection Git exclue par instruction explicite.',
'Fait|README : chantier, traversée, refuge, échec et secours ; protocole 10–15 min dans PLAYTEST.',
'Fait|DECISIONS reprend D01–D24 et les arbitrages autonomes.',
'Fait|ARCHITECTURE ; quatre crates, galerie sans simulation/game, tests sans GPU.',
'Fait|AGENTS.md : règles locales, validation, limites et absence de Git.',
'Partiel|CLAUDE.md importe AGENTS.md. Chargement par une session Claude réelle non vérifié.',
'Fait|Modèle de décision, seuils et replis dans DECISIONS.',
'Fait|ASSETS, licences jointes, seuls actifs consommés repris.',
'Fait|Fixtures core et terrain partagé ; graine 7391 ; ordre stable BTreeMap.',
'Adapté|VALIDATION et JSON de mesure ; empreinte de sources remplace le commit demandé.',
'Fait|Références historiques conservées ; TASKS est le suivi actif de cette livraison.',
'Fait|Décisions confrontées aux tests ; limites et prochains travaux explicités dans VALIDATION.'
],
ENV:[
'Fait|evidence/environment.json : machine, pilotes, outils et résolution de référence.',
'Fait|MSVC déjà disponible ; compilation et lancement Windows réussis.',
'Fait|Dossier aether-reboot isolé, workspace quatre crates ; prototype conservé.',
'Fait|Rust 1.98, Bevy 0.19, Avian 0.7, Cargo.lock et cible WASM.',
'Fait|glam public 0.32.1 ; arbre des dépendances dans evidence/dependencies.txt.',
'Fait|Features explicites et profils dev/release/web-release ; builds des deux cibles.',
'Fait|setup.ps1, CLI verrouillé et SHA256 vérifié ; documentation Windows.',
'Partiel|Sept tests voxel historiques passent ; aucune validation visuelle complète du prototype ancien.',
'Préparé|ci/windows-validation.yml prêt ; activation distante exclue de cette livraison locale.',
'Préparé|Étape WASM et archivage des logs dans le modèle CI ; exécution locale faite, distante non faite.',
'Fait|Copie source propre hors workspace vérifiée avec cache de dépendances partagé ; clean-source.json.',
'Fait|verify.ps1 journalise builds natif/web ; socle et limites legacy séparés.'
],
APP:[
'Fait|Point de composition app.rs ; galerie autonome avec renderer partagé.',
'Fait|Boot/Loading/Error/Menu/Playing/Editing/Paused/Settings ; retours pause et préférences distincts.',
'Fait|SessionEntity et nettoyage ; vingt reprises avec comptage stable des entités.',
'Fait|Caméra HDR, éclairage, terrain, F3 ; captures jeu et galerie.',
'Fait|LoadState réel, timeout, Retry ; asset critique absent puis restauré testé navigateur.',
'Fait|Chrome/Edge WebGPU ; GPU absent testé avec message et reprise.',
'Fait|Intention de frame consommée dans PhysicsSchedule à 60 Hz ; interpolation Avian.',
'Partiel|F3 et JSON exposent frame/tick/mesh/entités ; compteur détaillé des contacts non affiché.',
'Fait|Harnais MinimalPlugins, temps piloté, nombre de ticks exact et scénarios finis.',
'Partiel|Pause, focus et rattrapage borné implémentés ; suspension longue réelle d’onglet non certifiée.',
'Fait|Benchmarks exportés, 5 s d’échauffement, p50/p95/p99 et unités.',
'Fait|Socle natif/web vérifié ; erreurs et limites récapitulées dans VALIDATION.'
],
PHY:[
'Fait|Avian FixedPostUpdate/PhysicsSchedule, SI et poses autoritaires ; chute/contact testés.',
'Fait|Îles statiques, corps distincts, ancres et points locaux partagés avec le domaine.',
'Fait|Voxel collider enfant compensé de −0,25 m ; surface et masse vérifiées.',
'Partiel|Coque/paroi, coque/coque, sphère/pont et marche testés ; matrice exhaustive de toutes arêtes en debug/release non établie.',
'Non applicable|Les contacts de référence passent avec le collider voxel ; repli boîtes inutile.',
'Partiel|Choix voxel documenté ; coût par paire de contacts non isolé du tick global.',
'Fait|Masse, centre et inertie explicites ; volumes tournés et théorème des axes parallèles.',
'Fait|Forces au point et couples locaux ; voile décentrée, sustentation et commandes bornées.',
'Partiel|Secours remet poses/vitesses et recrée la flotte ; sommeil/réveil non mesuré comme scénario autonome.',
'Fait|CCD linéaire : coque à 40 m/s et sphère à 100 m/s arrêtées par une paroi mince.',
'Fait|300 ticks à 30/60/120 FPS et frames irrégulières ; <1 cm, <0,5°, réserve <0,001.',
'Fait|Tuning versionné ; essais de 10 et 30 minutes simulées sans NaN.'
],
CHR:[
'Fait|Tnua 0.32 / adaptateur 0.12 ; même PhysicsSchedule, builds natif/WASM.',
'Partiel|Translation 5 m/s et rotation 30°/s : dérive <5 cm/30 s ; inclinaison extrême non certifiée.',
'Fait|Marche, saut, atterrissage ; conservation du mouvement du support testée à 5 m/s.',
'Partiel|Escaliers, plafond bas, stabilité de coque passent ; pente maximale configurée à 0,7 rad sans matrice complète de pentes/FPS.',
'Fait|Marche embarquée retenue, limites d’essais explicites dans D10.',
'Fait|Pilotage sans personnage physique actif ; une seule autorité des poses.',
'Fait|Recherche d’une cellule avec hauteur libre près du poste ; héritage des vitesses et refus si aucun emplacement.',
'Partiel|AZERTY/QWERTY logiques, flèches, UI/focus ; remappage utilisateur des touches non livré.',
'Fait|Suivi amorti en secondes, orbit et modes ; caméra ne modifie aucune pose physique.',
'Fait|Knight animé Idle/Walk/Jump ; représentation de remplacement visible.',
'Fait|Récupération hors limites et R au dernier quai ; construction conservée, fragments abandonnés.',
'Fait|Scénarios intégrés natif/web incarnent puis sauvegardent/rechargent le personnage.'
],
DAT:[
'Fait|BodyId et catalogues sérialisables distincts des Entity/Handle.',
'Fait|Cellules centrées de 0,5 m ; transformations tournées et coordonnées négatives testées.',
'Fait|Division euclidienne 16³ ; cas limites et extrêmes i32 testés.',
'Fait|Chunks denses dans BTreeMap sparse ; libération du vide et export stable.',
'Fait|Bois/métal/verre : masses, occlusion et matériaux explicites.',
'Fait|Poste, voile, sustentateur, réservoir, harpon ; empreintes orientées et supports validés.',
'Fait|Construction privée, révision, plafonds 10k cellules/128 par axe/coordonnées bornées.',
'Fait|propose partagé aperçu/commit ; transaction refusée sans mutation, chunks affectés retournés.',
'Fait|Fixtures du domaine et résolution explicite des IDs lors du chargement.',
'Fait|Masse, centre et inertie analytique des cellules et équipements.',
'Fait|Arc de snapshot immutable ; révisions avant application des résultats.',
'Fait|32 tests domaine exécutés sans renderer.'
],
MSH:[
'Fait|Prédicat air/opaque/verre testé ; verre ne cache pas l’opaque.',
'Fait|Greedy déterministe par masque, normales et winding testés.',
'Fait|Voisins des vrais chunks lus ; invalidation des faces aux frontières.',
'Fait|UV en mètres locaux, invariants à translation/rotation du corps.',
'Fait|Palette partagée, handles et racine runtime ; aucun chemin source obligatoire.',
'Fait|Mesh/verre séparé, culling désactivé, limites du tri alpha documentées.',
'Adapté|Bois PBR commun et couleurs du catalogue ; cartes essentielles manquantes donnent Error/Retry, pas un fallback silencieux.',
'Partiel|File dédoublonnée et résultats périmés ignorés ; priorité spatiale caméra non implémentée.',
'Partiel|Jobs natifs sur snapshots et application par révision ; absence de test unitaire à ordonnanceur artificiellement inversé.',
'Fait|WASM coopératif : 96 masques bornés/chunk et budget 1,5 ms par frame.',
'Partiel|CPU mesh/application et délai file→application mesurés ; upload GPU et délai exact commande→photon non mesurés.',
'Fait|Benchmark 1k/10k/50k répartis ; greedy simple conservé, plafond public inchangé.'
],
EDT:[
'Fait|Interaction UI bloque la construction ; 720p, 1080p et petite fenêtre testées.',
'Fait|DDA local borné par AABB ; parallèles, intérieur, négatifs et corps tournés testés.',
'Fait|Aperçu commun aux règles, ajout/retrait/refus, seuil clic/drag et orbit séparés.',
'Fait|Ajout/retrait transactionnels ; départ refusé sans fonctions requises.',
'Fait|Rotation par quart de tour, empreinte complète et conflits visibles.',
'Fait|Historique 128 transactions/16 Mio ; 100 éditions et annulations vérifiées.',
'Fait|Atelier au quai ; reconstruction collider/masses/meshes avant départ.',
'Fait|Connexité exacte par faces ; pont et boucle distante testés.',
'Fait|Scission sur blueprint, IDs neufs, composants répartis et masse conservée.',
'Fait|Origine locale conservée ; vitesse du fragment calculée au nouveau centre de masse.',
'Fait|32 corps maximum ; attachages libérés, undo flotte borné au quai.',
'Fait|Scission, undo/redo et reprise vérifiés par QA native et navigateur.'
],
SAV:[
'Fait|Blueprint v1 et fixture asymétrique avec rotation dans examples-data.',
'Fait|Session v1 : poses, consignes, réserves, acteur, progression et câble.',
'Fait|16 Mio maximum, 32 corps, 10k cellules par corps ; validation des valeurs/IDs.',
'Fait|Decode et validation avant remplacement ; import corrompu ne modifie pas la session.',
'Partiel|Temporaire, sync_all, remplacement et backup vérifié ; arrêt électrique à chaque instruction non simulé.',
'Fait|IndexedDB strict ; succès après oncomplete ; quota et abort testés.',
'Fait|Import/export JSON natif et web ; téléchargement et aller-retour navigateur vérifiés.',
'Fait|Autosave aux frontières sûres, mailbox unique ; mutations concurrentes d’identité bloquées.',
'Fait|Round-trip tourné/déplacé, valeurs, version inconnue et premières v1 testés.',
'Fait|Restauration corps→attaches→acteur ; intentions remises à zéro.',
'Fait|Backup disque/web, JSON tronqué, quota, transaction abortée et fermeture native exercés.',
'Fait|Flotte maximale encode/decode ; package déplacé et navigateur rechargé ; exemples versionnés.'
],
FLT:[
'Fait|tuning.rs : coefficients SI, plafonds de forces/rotation/altitude versionnés.',
'Fait|Vent seedé, temps SimClock à 60 Hz ; variation continue.',
'Fait|Vitesse linéaire + ω×r au point de voile dans le vent relatif.',
'Fait|Poussée selon normale/aire ; plafond et test face/profil.',
'Fait|Réglage de voile et couple de direction limités ; aucune téléportation de pilotage.',
'Fait|Traînée et frein au tick ; invariance FPS testée.',
'Fait|Demande verticale PD appliquée au centre des sustentateurs ; indépendante de la voile.',
'Fait|HUD vent/vitesse/altitude/trim, cap et distance du quai ; particules de direction.',
'Fait|Traversée aux intentions physiques et aux touches navigateur ; seuil <9 m et <3,5 m/s.',
'Partiel|Masse ajoutée augmente consommation et surcharge empêche vol ; comparaison jouée de voile déplacée non réalisée.',
'Partiel|Navigation automatisée terminable, caméra réglable ; confort humain reste à observer.',
'Fait|Traversée finale directement éprouvée avec sustentation Aether réelle.'
],
AET:[
'Fait|Unités de jeu distinctes ; capacité et quantité bornées.',
'Fait|Réserve par corps issue des réservoirs ; zéro capacité désactive le vol.',
'Fait|Demande limitée à 22 kN/sustentateur ; coque lourde testée sans flottement artificiel.',
'Fait|Débit au tick et force réduite à l’épuisement ; invariant FPS/pause.',
'Fait|Recharge bornée de 25 unités/s au quai ; test de non-déplacement et recharge.',
'Fait|Équipements violets et barre de réserve avec texte chiffré.',
'Fait|Avertissements réserve/déficit, son et R de secours documenté.',
'Fait|Session contient réserve ; blueprint/undo/import ne créent pas de carburant.',
'Fait|Retrait plafonne la réserve ; scission répartit proportionnellement aux capacités.',
'Fait|Panne coupe sustentation ; secours recrée la construction au dernier quai.',
'Partiel|Réserve de traversée et surconsommation de masse testées ; fréquence frustrante des pannes attend l’essai joueur.',
'Fait|Traversée physique réelle, reprise/câble/personnage et secours dans les scénarios intégrés.'
],
CAB:[
'Fait|DistanceJoint min=0/max=L ; absence de compression et dix minutes bornées.',
'Fait|Harpon local et index d’ancre persistés ; IDs validés.',
'Fait|Portée 70 m, visibilité exacte contre terrain, un seul câble.',
'Fait|Longueur initiale égale à la distance ; attache sans repositionner le corps.',
'Fait|Suppression du joint seule ; test de continuité des vitesses au lâcher.',
'Fait|Indicateur géométrique souple/tendu ; aucune force prétendue en newtons.',
'Fait|Courbe visuelle souple/tendue, couleur et signal audio de transition.',
'Fait|Attache initiale occultée refusée ; obstacle ultérieur libère le câble ; aucun wrapping.',
'Fait|Suppression/scission libère les attaches ; nettoyage avec la session.',
'Fait|Chargement après corps, validation du harpon et de la longueur ; QA save/load attaché.',
'Partiel|Erreur bornée <2 % sur dix minutes de fixture ; produit complet masse/vitesse/FPS maximal non certifié.',
'Fait|Balancier vers refuge avec accroche/lâcher réels, sans téléportation, scénario headless.'
],
CUR:[
'Fait|Tubes de segments 3D avec rayon, vitesse et mélange bornés.',
'Fait|Champ continu et nul hors volume ; test de frontière.',
'Fait|Relaxation 2 s, accélération maximale 8 m/s² ; forces uniquement.',
'Fait|Forces sur Vessel dynamique seulement ; personnage et ancres exclus.',
'Fait|Jardins et refuge atteints avec physique/obstacles de production.',
'Partiel|Ancre de refuge et fenêtre d’action validées par pilote scripté ; temps de réaction humain inconnu.',
'Fait|Courant violet et particules orientées par le même champ que la simulation.',
'Fait|Trois îles, quais et silhouettes, terrain/collision partagés.',
'Fait|Étapes de progression persistées, arrivée reconnue au quai.',
'Fait|Checkpoint et récupération de la coque principale avec recharge.',
'Partiel|Courant/voile/câble et masse/panne testés dans plusieurs fixtures ; combinaison exhaustive non certifiée.',
'Fait|Traversée navigateur aux touches réelles, sonde uniquement en lecture ; capture d’arrivée.'
],
UX:[
'Fait|Thème partagé, interface française et terminologie cohérente ; pas de système i18n général.',
'Fait|Échelle UI selon fenêtre ; boutons dans le viewport à 960×640.',
'Fait|Raccourcis contextuels, refus et états de préparation dans le footer.',
'Partiel|Objectifs déclenchés par édition/départ/arrivée ; tutoriel séparé de chaque geste voile/énergie à affiner avec joueurs.',
'Fait|HUD réserve, déficit, navigation, objectif et état du câble.',
'Fait|Volume, sensibilité, inversion, réduction du mouvement ; préférences séparées et validées.',
'Fait|Quatre sons originaux reproductibles, provenance et volume ; messages doublent les alertes.',
'Partiel|États doublés par texte et caméra réduite ; audit instrumenté des contrastes et niveaux de gris non réalisé.',
'Fait|PBR bois albedo/normal/ORM, sampler unique Repeat, UV métriques ; exception end-grain documentée.',
'Fait|Galerie partagée avec coque, composants, vitrage tourné et chunks négatifs ; capture/timeout/code de sortie.',
'Fait|Captures natives et navigateur inspectées ; raccords, répétition bois, volumes et panneaux revus.',
'Externe|Compréhension autonome sans explication à évaluer avec une personne ; un robot ne remplace pas cet essai.'
],
WEB:[
'Fait|Assets runtime séparés, manifestes SHA256, tailles et notices de provenance.',
'Fait|Package Windows déplacé hors dépôt ; assets relatifs exe et données utilisateur distinctes.',
'Fait|Bundle CLI testé à la racine et sous préfixe ; serveur Windows/Node.',
'Fait|Ensemble immuable r/version-empreinte et launcher non caché ; manifeste de chaque livraison.',
'Fait|Vérification GPU, chargement module/initialisation, Loading assets, erreurs/retry/timeout ; aucun pourcentage fictif.',
'Partiel|URL unique et rechargement testés ; historique back/forward et bfcache non validés séparément.',
'Partiel|Clavier, souris, redimensionnement, download et pause testés ; plein écran et changement prolongé d’onglet non certifiés.',
'Fait|Tailles Brotli, mémoire linéaire WASM, démarrage froid/chaud à 50 Mbit/s mesurés.',
'Fait|Huit tests Playwright : parcours, benchmark, éditeur/persistance, QA, GPU, asset, quota, backup/abort.',
'Adapté|Packages reproductibles, checksums et source-manifest ; empreinte remplace commit, aucune opération Git.',
'Fait|Première v1 et v1 actuelle compatibles ; entrée non cachée basculée nouveau → ancien → nouveau entre deux builds réels, IndexedDB conservé et assets cohérents.',
'Fait|Archives Windows/web, licences, contrôles et procédure RELEASE ; aucune publication.'
],
QA:[
'Fait|Matrice et preuves dans VALIDATION ; automatisation distinguée des observations humaines.',
'Partiel|30 min simulées navigation/recovery/codec ; édition/scission/stockage rendus vérifiés séparément, pas une session humaine rendue de 30 min.',
'Partiel|20 cycles et 100 éditions/annulations ; entités stables et mémoire de benchmark mesurée, pas une preuve de fuite nulle à durée infinie.',
'Partiel|DDA tourné, continuité scission, reload et FPS physique vérifiés ; erreur visuelle interpolée non mesurée pixel par pixel.',
'Fait|Intentions physiques répétées et ticks exacts ; tolérances documentées, aucun déterminisme inter-CPU promis.',
'Fait|Tests génératifs bornés, i32 extrêmes, JSON/IDs/limites, poids et fichiers invalides refusés.',
'Partiel|1080p, 10k cellules et 32 corps dynamiques : frame/tick/mémoire/mesh mesurés ; timestamps GPU dédiés absents.',
'Fait|Stress 50k répartis, plafond d’import et flotte maximale testés ; limite publique conservée.',
'Partiel|Quota/abort/backup, fermeture native et asset absent testés ; coupure électrique et longue suspension OS non reproduites.',
'Fait|Windows natif, Chrome et Edge réels ; versions et options d’automatisation consignées.',
'Préparé|Régressions et logs sélectionnés dans verify/CI ; durée distante de CI inconnue tant que non activée.',
'Partiel|Aucun P0/P1 observé sur parcours validés ; réserves de mesure/UX humaines explicites, pas de certification définitive universelle.'
],
PLY:[
'Fait|PLAYTEST : protocole 10–15 min, départ, amélioration, courant/câble et secours.',
'Fait|Grille vierge faits/paroles/interprétation et indicateurs de compréhension.',
'Fait|Candidat identifié, archives, commandes, lancement et invitation modèle sans envoi.',
'Externe|Répétition avec une personne non réalisée ; fiche et build prêts.',
'Externe|Cinq personnes à observer ; aucune fiche ni satisfaction inventée.',
'Externe|Consolidation dépend des cinq observations réelles.',
'Externe|Hypothèse de plaisir/compréhension à confronter aux observations, pas aux FPS.',
'Externe|Trois corrections maximum à choisir sur les faits recueillis.',
'Externe|Cycle de correction dépend du choix PLY-08 ; aucun changement spéculatif présenté comme validé.',
'Externe|Second essai avec deux personnes après corrections ; non réalisable par automatisation.',
'Fait|Notes, contrôles, récupération, limites et archives de ce candidat technique.',
'Externe|Décision de produit avec Damien après essais ; dossier technique préparé, jalon humain non fermé.'
]};
// Closure evidence supersedes the first candidate, without rewriting the original plan.
const closure={
GOV:{3:'Fait|DECISIONS consigne les choix initiaux et les corrections de fermeture.',6:'Adapté|AGENTS chargé par Codex ; CLAUDE.md importe les mêmes règles. Aucun lancement de Claude annoncé.'},
ENV:{8:'Fait|Sept tests voxel historiques exécutés ; résultat et limites legacy archivés, sans déduire la jouabilité du prototype.'},
APP:{8:'Fait|F3/JSON : entités actives distinctes des indices alloués, contacts, triangles, CPU et passes GPU reçues.',10:'Fait|Perte réelle du focus canvas, pause et suspension JS de 65 s éprouvées ; rattrapage borné.'},
PHY:{4:'Fait|Contacts coque/paroi/coque, pont, arêtes, pentes et escaliers éprouvés en dev et release.',6:'Fait|Trois paires isolées, phases broad/narrow, échauffement et percentiles : evidence/pair-benchmark.json.',9:'Fait|Sommeil/réveil, téléportation avec reset interpolation et reprise des contacts éprouvés.'},
CHR:{2:'Fait|Support à 5 m/s et 30°/s : dérive <5 cm/30 s ; pentes 15/30/39° vérifiées séparément.',4:'Fait|Pentes 15/30/39° à 30/60/120 Hz, arêtes internes, escaliers et plafond bas : matrice envelope.',8:'Fait|Dix-huit commandes réaffectables, conflits contextuels, AZERTY/QWERTY logiques ; persistance et reset testés.',10:'Fait|Knight Idle/Walk/Jump ; absence du GLB éprouvée dans les deux navigateurs, représentation de secours et marche utilisables.'},
DAT:{12:'Fait|Tests purs du domaine, dont équivalence volume et 9 000 intersections du terrain fusionné.'},
MSH:{8:'Fait|Corps édité prioritaire puis distance caméra ; file stable et dédoublonnée.',9:'Fait|Test du vrai ordonnanceur : résultat récent puis obsolète, révision récente conservée.',11:'Adapté|CPU maillage/application/file et passes GPU instrumentés ; 20 éditions input→PNG donnent une borne haute, pas un délai écran/photon.'},
SAV:{5:'Fait|Douze interruptions injectées aux frontières backup/principale ; au moins une sauvegarde valide conservée.'},
FLT:{10:'Fait|Masse/consommation et voile décalée ±0,5 m mesurées ; COMPARAISONS.md.',11:'Fait|Traversées aux intentions et touches, caméra amortie/réduction du mouvement ; appréciation humaine réservée à PLY.'},
AET:{11:'Fait|Réserve et panne mesurées sur navire chargé ; recharge et récupération testées. Frustration perçue à observer avec PLY.'},
CAB:{11:'Fait|54 combinaisons masse 1k/10k/900k, vitesse 0/25/80, longueur 3/80, FPS 30/60/120 ; pause et reprise incluses.'},
CUR:{6:'Fait|Approche sans téléportation, portée et visibilité réelles ; retard 0/0,5/1,5 s avant accroche, refuge atteint et réserve positive.',7:'Fait|Courant lumineux orienté selon le champ, texte et indicateurs ; couleur seule jamais nécessaire.',8:'Fait|Trois îles, quais, ruines et observatoire ; géométrie et visibilité du domaine partagées avec Blender.',11:'Fait|Navire chargé combinant voile, courant, câble, faible réserve puis panne ; fixture intégrée réussie.'},
UX:{4:'Fait|Huit étapes observées dans les actions réelles, persistées et rejouables ; navigateur vérifie les huit bits.',8:'Fait|Contraste sRGB ≥4,5:1, focus et états compris ; informations doublées par texte, caméra réduite.',9:'Fait|Cèdre albedo/normal, toile indigo, UV métriques, mipmaps en lumière linéaire, anisotropie et éclairage d’environnement.'},
WEB:{6:'Fait|Retour/avance historique avec rechargement et sauvegarde conservée ; pas de certification bfcache.',7:'Fait|Clavier/souris, focus, plein écran, redimensionnement 960×640 et suspension JS 65 s éprouvés.',9:'Fait|Seize tests dans Chrome et Edge : parcours, QA, erreurs, stockage, commandes, tutoriel, focus, historique, plein écran et voile animée.'},
QA:{2:'Fait|Session rendue ≥30 min, cycles édition/undo/redo/save/load/navigation/câble/marche/recovery ; sonde en lecture seule.',3:'Fait|Vingt recréations et cent éditions/annulations ; comptage des entités réellement actives, meshes, matériaux et file.',4:'Fait|Interpolation mesurée en mètres et reset téléportation ; points d’équipements et picking vérifiés après reprise/scission.',7:'Fait|Baseline finale 1080p 10k cellules/32 corps, CPU/contact/triangles, GPU natif et mémoire. GPU web absent explicitement.',9:'Fait|Stockage refusé/interrompu, backup, fermeture native, assets absents et suspension JS prolongée testés.',12:'Fait|Aucun défaut P0/P1 observé dans les scénarios validés ; limites et défauts résiduels consignés dans DEFAUTS.md.'}
};
for(const [group,items] of Object.entries(closure))for(const [number,assessment] of Object.entries(items))groups[group][Number(number)-1]=assessment;
const soak=JSON.parse(fs.readFileSync('docs/evidence/rendered-soak.json','utf8'));
if(!soak.finished||soak.elapsed_seconds<1800||soak.errors.length)throw Error('Endurance finale non validée ; ne pas publier le registre de fermeture');
const tour=JSON.parse(fs.readFileSync('docs/evidence/world-tour.json','utf8'));
if(!tour.finished||tour.errors.length||tour.wasm_sha256!==soak.wasm_sha256)throw Error('Le parcours du monde ouvert et l’endurance doivent concerner le même candidat');
const plan=fs.readFileSync('docs/reference/PLAN-DEVELOPPEMENT.md','utf8');
const rows=plan.split(/\r?\n/).filter(l=>/^\| [A-Z]+-\d{2} \|/.test(l)).map(line=>{
 const [id,title,acceptance,dependencies,estimate]=line.split('|').slice(1).map(s=>s.trim());
 const [group,number]=id.split('-');let assessment=groups[group]?.[Number(number)-1];
 if(group==='FLEX')assessment='Conditionnel différé|La voile animée et la force simplifiée restent actives ; aucun solveur XPBD de tissu ou d’enroulement livré. Le retour positif sur la toile ne justifie pas ce coût mécanique.';
 if(group==='EXT')assessment=({
  '01':'Adapté|Demande directe de Damien : monde plus vaste et fidèle, navigation motorisée. D47–D63 activent le lot, sans prétendre disposer de cinq playtests.',
  '04':'En cours|D92 active le réseau demandé : domaine par segments et laboratoire physique sur la vraie coque, coupure ciblée des moteurs et sustentation indépendante éprouvées. Authoring des conduites, interface et persistance restent ouverts.',
  '05':'En cours|Quantités entières, scission de segments, débits bornés, fuite, pompe et conservation sur dix mille opérations passent. Raccord aux annulations de l’éditeur, scission réelle de coque et sauvegarde de session encore nécessaire.',
  '08':'Fait|204 îles nouvelles, neuf régions et trois ports historiques ; collisions chargées par proximité, préparation avant portail, LOD praticables et IDs stables. Tests world, assets et parcours rendu.',
  '10':'Adapté|Modèles Blender détaillés et satellites déterministes ; placements, quais, ressources et invariants vérifiés. Pas de génération de topologie arbitraire ; catalogue borné associé à la version de sauvegarde.',
  '12':'En cours|R60 et laboratoire R67 validés séparément ; fermeture visuelle, nouveaux systèmes, performances et distribution 0.2 encore en cours. Les preuves historiques ne certifient pas le candidat final.'
 })[number]||'Conditionnel différé|Ce sous-système du plan initial n’est pas implémenté par le lot monde ouvert. Voir MONDE-OUVERT : eau animée/récoltable, réseau d’énergie commun sans tuyaux, pas de coopération ni de codec VOX.';
 if(group==='OPT')assessment=({
 '01':'Fait|CPU maillage/décoration/application/file et édition→PNG mesurés. Sonde 5 Hz corrigée (D41), profil CDP attribuant le délai résiduel aux tangentes décoratives (D45). Contacts Edge traités séparément ; COMPARAISONS.',
 '02':'Non applicable|Les coûts dominants étaient les contacts et les tangentes décoratives, pas le greedy ; aucune raison mesurée de remplacer ce dernier.',
 '03':'Non applicable|Aucune stratégie de meshing concurrente retenue ; comparaison artificielle non nécessaire.',
 '04':'Non applicable|Greedy actuel conservé, priorité et ordonnanceur éprouvés ; pas de migration sans gain mesuré.',
 '06':'Adapté|SSAO/TAA du renderer partagé au lieu d’un AO par sommet, contrôlés en jeu et galerie.',
 '10':'Fait|Compression Brotli/gzip, ensemble versionné et démarrage froid/chaud mesurés.',
 '12':'Fait|Baseline finale et historique avant/après conservés ; aucun prototype concurrent dans le chemin distribué.'
 })[number]||'Conditionnel différé|Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan.';
 if(!assessment)throw Error('Évaluation manquante '+id);const separator=assessment.indexOf('|');
 return {id,title,acceptance,dependencies,initial_estimate_hours:Number(estimate),status:assessment.slice(0,separator),assessment:assessment.slice(separator+1)};
});
if(rows.length!==240||new Set(rows.map(r=>r.id)).size!==240)throw Error('Le plan doit contenir exactement 240 identifiants');
const counts={};for(const row of rows)counts[row.status]=(counts[row.status]||0)+1;
let output='# Suivi exhaustif du reboot — 240 tâches\n\nÉtat de la livraison locale du 2 octobre 2026. Ce registre reprend chaque identifiant du [plan de référence](reference/PLAN-DEVELOPPEMENT.md). **Fait** décrit le résultat technique observé, **partiel** signale un critère restant, **préparé** une configuration non exécutée à distance, **externe** une observation ou décision humaine nécessaire. Les extensions conditionnelles ne sont pas des fonctions promises dans la démo.\n\nLes preuves de commande, les mesures et leurs limites sont rassemblées dans [VALIDATION](VALIDATION.md). Les choix se trouvent dans [DECISIONS](DECISIONS.md), les modules dans [ARCHITECTURE](ARCHITECTURE.md). Aucune référence à un commit n’est inventée : Git est exclu et les sources livrées ont une empreinte SHA256.\n\n';
output+='| Statut | Nombre |\n| --- | ---: |\n'+Object.entries(counts).map(([s,c])=>`| ${s} | ${c} |`).join('\n')+'\n\n';
let previous='';for(const row of rows){const group=row.id.split('-')[0];if(group!==previous){output+=`\n## ${group}\n\n| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |\n| --- | --- | --- | --- |\n`;previous=group;}output+=`| ${row.id} | ${row.title} | ${row.status} | ${row.assessment} |\n`;}
output+='\n## Extension du monde ouvert\n\nLa demande directe de Damien active le streaming et le catalogue étendu. [MONDE-OUVERT](MONDE-OUVERT.md) détaille les interactions, les choix issus des planches et les limites ; [RENDU](RENDU.md) porte la comparaison graphique. Le parcours de récolte et de caverne est exécuté aux touches, sans téléportation par la sonde.\n\n## Suites dépendant d’observations ou de services externes\n\n1. Réaliser le protocole avec une première personne puis cinq observations séparées ; exploiter les fiches sans présumer du plaisir.\n2. Choisir les corrections sur ces faits puis conduire le second essai humain prévu.\n3. Activer la CI distante lors d’une étape autorisant les opérations de dépôt. La même validation est exécutée localement ici.\n4. Les sous-systèmes FLEX et EXT restants gardent leur propre justification et leurs critères ; ils ne sont pas marqués terminés par analogie avec le monde ouvert.\n5. Étendre la matrice à d’autres machines et GPU lors de leur mise à disposition ; les résultats présents identifient précisément la machine éprouvée.\n';
fs.writeFileSync('docs/TASKS.md',output);fs.writeFileSync('docs/tasks.json',JSON.stringify({total:rows.length,counts,tasks:rows},null,2));console.log(JSON.stringify(counts));
