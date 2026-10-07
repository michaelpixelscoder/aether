# Suivi exhaustif du reboot — 240 tâches

Registre initial de la livraison 0.1 du 2 octobre 2026, actualisé le 3 octobre pour les extensions activées. Ce registre reprend chaque identifiant du [plan de référence](reference/PLAN-DEVELOPPEMENT.md). **Fait** décrit le résultat technique observé, **partiel** signale un critère restant, **préparé** une configuration non exécutée à distance, **À faire** un lot autorisé dont le développement reste à entreprendre, **externe** une observation ou décision humaine nécessaire. La demande ultérieure de monde ouvert active les lots décrits dans [MONDE-OUVERT](MONDE-OUVERT.md) ; leur finition artistique et leur nouvelle distribution sont encore en cours. Les preuves historiques de 0.1 ne certifient pas automatiquement 0.2.

Les preuves de commande, les mesures et leurs limites sont rassemblées dans [VALIDATION](VALIDATION.md). Les choix se trouvent dans [DECISIONS](DECISIONS.md), les modules dans [ARCHITECTURE](ARCHITECTURE.md). Aucune référence à un commit n’est inventée : Git est exclu et les sources livrées ont une empreinte SHA256.

| Statut | Nombre |
| --- | ---: |
| Adapté | 9 |
| Fait | 189 |
| Préparé | 3 |
| Non applicable | 4 |
| Externe | 9 |
| Conditionnel différé | 9 |
| À faire | 14 |
| En cours | 3 |


## GOV

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| GOV-01 | Enregistrer commit historique, état Git, documents de référence et périmètre du reboot dans une fiche de démarrage. | Adapté | Références copiées ; empreinte SHA256 des sources livrées. Inspection Git exclue par instruction explicite. |
| GOV-02 | Écrire la promesse de la démo : construire au quai, traverser, s'ancrer, réparer et recommencer ; définir ses exclusions. | Fait | README : chantier, traversée, refuge, échec et secours ; protocole 10–15 min dans PLAYTEST. |
| GOV-03 | Adopter le registre D01–D24 comme défaut proposé et consigner les décisions produit de Damien lorsqu'elles arrivent. | Fait | DECISIONS consigne les choix initiaux et les corrections de fermeture. |
| GOV-04 | Décrire les quatre crates et les imports autorisés ; placer les contrats partagés dans le domaine. | Fait | ARCHITECTURE ; quatre crates, galerie sans simulation/game, tests sans GPU. |
| GOV-05 | Préparer `AGENTS.md` avec question de valeur, architecture, limites, commandes et définition de terminé. | Fait | AGENTS.md : règles locales, validation, limites et absence de Git. |
| GOV-06 | Vérifier les outils d'agents utilisés ; ajouter si nécessaire `CLAUDE.md` avec import `@AGENTS.md`. | Adapté | AGENTS chargé par Codex ; CLAUDE.md importe les mêmes règles. Aucun lancement de Claude annoncé. |
| GOV-07 | Créer un modèle de décision d'une page : problème, défaut, déclencheur, essai limité et résultat. | Fait | Modèle de décision, seuils et replis dans DECISIONS. |
| GOV-08 | Inventorier actifs réutilisables, provenance, licences et consommateurs : Knight, textures, fixtures, solveur. | Fait | ASSETS, licences jointes, seuls actifs consommés repris. |
| GOV-09 | Définir les scénarios canoniques et graines : pont, coque, deux chunks, câble et archipel. | Fait | Fixtures core et terrain partagé ; graine 7391 ; ordre stable BTreeMap. |
| GOV-10 | Fixer format des preuves : commit, cible, matériel, commande, résultat et limites. | Adapté | VALIDATION et JSON de mesure ; empreinte de sources remplace le commit demandé. |
| GOV-11 | Mettre les anciennes initiatives en référence historique et définir une seule feuille de route active. | Fait | Références historiques conservées ; TASKS est le suivi actif de cette livraison. |
| GOV-12 | Relire périmètre, décisions, risques et ordre des lots ; ouvrir seulement les prochaines tâches exécutables. | Fait | Décisions confrontées aux tests ; limites et prochains travaux explicités dans VALIDATION. |

## ENV

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| ENV-01 | Relever Windows, CPU/GPU/RAM, pilotes, Rust, Node et outils disponibles ; désigner la machine de référence. | Fait | evidence/environment.json : machine, pilotes, outils et résolution de référence. |
| ENV-02 | Installer/configurer MSVC C++ et SDK Windows manquants ; vérifier le linker dans un shell reproductible. | Fait | MSVC déjà disponible ; compilation et lancement Windows réussis. |
| ENV-03 | Créer la zone de reboot isolée, le workspace et les quatre crates sans déplacer le prototype. | Fait | Dossier aether-reboot isolé, workspace quatre crates ; prototype conservé. |
| ENV-04 | Épingler Rust compatible, Bevy 0.19 et Avian 0.7 ; générer le lockfile et déclarer les cibles. | Fait | Rust 1.98, Bevy 0.19, Avian 0.7, Cargo.lock et cible WASM. |
| ENV-05 | Aligner `glam` 0.32, Bevy Math et futures intégrations ; inspecter les versions dupliquées pertinentes. | Fait | glam public 0.32.1 ; arbre des dépendances dans evidence/dependencies.txt. |
| ENV-06 | Définir features natives/web et profils dev, release ; activer seulement les besoins réels. | Fait | Features explicites et profils dev/release/web-release ; builds des deux cibles. |
| ENV-07 | Installer une version verrouillée du CLI web et des outils nécessaires ; écrire les commandes Windows. | Fait | setup.ps1, CLI verrouillé et SHA256 vérifié ; documentation Windows. |
| ENV-08 | Tenter baseline historique, tests et captures pendant 2 h au maximum ; archiver succès ou échec. | Fait | Sept tests voxel historiques exécutés ; résultat et limites legacy archivés, sans déduire la jouabilité du prototype. |
| ENV-09 | Créer CI PR : fmt, Clippy, tests headless et vérification native Windows avec lockfile. | Préparé | ci/windows-validation.yml prêt ; activation distante exclue de cette livraison locale. |
| ENV-10 | Ajouter build WASM à la CI et archive de logs ; contrôler la correspondance wasm-bindgen/lockfile. | Préparé | Étape WASM et archivage des logs dans le modèle CI ; exécution locale faite, distante non faite. |
| ENV-11 | Documenter un parcours depuis checkout neuf ; faire une répétition dans un répertoire propre. | Fait | Copie source propre hors workspace vérifiée avec cache de dépendances partagé ; clean-source.json. |
| ENV-12 | Valider D01, D04 et environnement ; noter outils retenus et installer un suivi des temps de build. | Fait | verify.ps1 journalise builds natif/web ; socle et limites legacy séparés. |

## APP

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| APP-01 | Composer les plugins depuis un point d'entrée court ; créer une scène vide et un outil galerie minimal. | Fait | Point de composition app.rs ; galerie autonome avec renderer partagé. |
| APP-02 | Implémenter états Boot, Menu, Loading, Playing, Paused, Error et sortie ; définir les transitions légales. | Fait | Boot/Loading/Error/Menu/Playing/Editing/Paused/Settings ; retours pause et préférences distincts. |
| APP-03 | Donner un propriétaire de session aux entités/ressources temporaires et une procédure de nettoyage. | Fait | SessionEntity et nettoyage ; vingt reprises avec comptage stable des entités. |
| APP-04 | Initialiser fenêtre, caméra, éclairage sobre et carte de sol ; inclure commandes de debug. | Fait | Caméra HDR, éclairage, terrain, F3 ; captures jeu et galerie. |
| APP-05 | Ajouter chargement d'assets observé, erreur visible et tentative de reprise ; finir Loading après disponibilité réelle. | Fait | LoadState réel, timeout, Retry ; asset critique absent puis restauré testé navigateur. |
| APP-06 | Démarrer la scène minimale WebGPU sur Chrome/Edge cible ; tester absence d'adapter GPU. | Fait | Chrome/Edge WebGPU ; GPU absent testé avec message et reprise. |
| APP-07 | Définir collecte d'input, consommation au tick et interpolation du rendu dans des SystemSets ordonnés. | Fait | Intention de frame consommée dans PhysicsSchedule à 60 Hz ; interpolation Avian. |
| APP-08 | Ajouter overlay dev : frame/tick, corps, contacts, jobs et état ; journaux structurés désactivables. | Fait | F3/JSON : entités actives distinctes des indices alloués, contacts, triangles, CPU et passes GPU reçues. |
| APP-09 | Créer harnais headless avec temps piloté et captures d'état finies ; enregistrer les graines. | Fait | Harnais MinimalPlugins, temps piloté, nombre de ticks exact et scénarios finis. |
| APP-10 | Définir comportement de pause, perte de focus et reprise d'onglet ; borner le rattrapage des ticks. | Fait | Perte réelle du focus canvas, pause et suspension JS de 65 s éprouvées ; rattrapage borné. |
| APP-11 | Ajouter profils de test reproductibles et export de mesures ; isoler échauffement et séquence mesurée. | Fait | Benchmarks exportés, 5 s d’échauffement, p50/p95/p99 et unités. |
| APP-12 | Faire revue de socle : cycle de session, erreur d'asset, première image web et dépendances des crates. | Fait | Socle natif/web vérifié ; erreurs et limites récapitulées dans VALIDATION. |

## PHY

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| PHY-01 | Installer Avian dans le schedule choisi ; déclarer axes, unités m/kg/s, gravité et autorité des poses. | Fait | Avian FixedPostUpdate/PhysicsSchedule, SI et poses autoritaires ; chute/contact testés. |
| PHY-02 | Créer île statique, petite coque dynamique et points d'ancrage locaux à partir de fixtures. | Fait | Îles statiques, corps distincts, ancres et points locaux partagés avec le domaine. |
| PHY-03 | Produire collider voxel de la fixture et afficher collider/mesh superposés pour diagnostic. | Fait | Voxel collider enfant compensé de −0,25 m ; surface et masse vérifiées. |
| PHY-04 | Tester coque/île, coque/coque, sphère/pont et franchissement d'arêtes internes en debug/release. | Fait | Contacts coque/paroi/coque, pont, arêtes, pentes et escaliers éprouvés en dev et release. |
| PHY-05 | Si PHY-04 échoue, isoler le cas ≤ 4 h puis implémenter repli en boîtes fusionnées. | Non applicable | Les contacts de référence passent avec le collider voxel ; repli boîtes inutile. |
| PHY-06 | Choisir et verrouiller le chemin collision initial ; enregistrer le coût par type de paire. | Fait | Trois paires isolées, phases broad/narrow, échauffement et percentiles : evidence/pair-benchmark.json. |
| PHY-07 | Calculer masse, centre et inertie des fixtures ; éviter qu'Avian les recalcule différemment sans contrôle. | Fait | Masse, centre et inertie explicites ; volumes tournés et théorème des axes parallèles. |
| PHY-08 | Appliquer force et couple à un point local transformé ; exposer commandes d'essai bornées. | Fait | Forces au point et couples locaux ; voile décentrée, sustentation et commandes bornées. |
| PHY-09 | Tester sommeil/réveil, contact après téléportation de debug et remise au quai. | Fait | Sommeil/réveil, téléportation avec reset interpolation et reprise des contacts éprouvés. |
| PHY-10 | Valider vitesse maximale initiale, petite épaisseur, chute et activation de CCD si nécessaire. | Fait | CCD linéaire : coque à 40 m/s et sphère à 100 m/s arrêtées par une paroi mince. |
| PHY-11 | Jouer mêmes intentions à 30/60/120 FPS et avec frames irrégulières, à ticks physiques constants. | Fait | 300 ticks à 30/60/120 FPS et frames irrégulières ; <1 cm, <0,5°, réserve <0,001. |
| PHY-12 | Enregistrer paramètres physiques initiaux et revue de stabilité ; fermer les alternatives non nécessaires. | Fait | Tuning versionné ; essais de 10 et 30 minutes simulées sans NaN. |

## CHR

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| CHR-01 | Intégrer `bevy-tnua` 0.32, adaptateur Avian3d 0.12 et couche commune ; capsule de test. | Fait | Tnua 0.32 / adaptateur 0.12 ; même PhysicsSchedule, builds natif/WASM. |
| CHR-02 | Éprouver support en translation, rotation et inclinaison avec Tnua pendant ≤ 4 h. | Fait | Support à 5 m/s et 30°/s : dérive <5 cm/30 s ; pentes 15/30/39° vérifiées séparément. |
| CHR-03 | Éprouver marche, bord, saut et atterrissage sur le même pont pendant ≤ 4 h. | Fait | Marche, saut, atterrissage ; conservation du mouvement du support testée à 5 m/s. |
| CHR-04 | Tester interaction masse personnage/navire, escalier, plafond et variation des FPS dans le budget restant. | Fait | Pentes 15/30/39° à 30/60/120 Hz, arêtes internes, escaliers et plafond bas : matrice envelope. |
| CHR-05 | Décider marche embarquée ou repli après plafond ; fixer le périmètre de la démo. | Fait | Marche embarquée retenue, limites d’essais explicites dans D10. |
| CHR-06 | Créer état pilotage : attache locale au poste, neutralisation du contrôleur, commandes de navire. | Fait | Pilotage sans personnage physique actif ; une seule autorité des poses. |
| CHR-07 | Définir entrée/sortie du poste et, si repli, autoriser débarquement seulement au quai immobile. | Fait | Recherche d’une cellule avec hauteur libre près du poste ; héritage des vitesses et refus si aucun emplacement. |
| CHR-08 | Ajouter intention par acteur contrôlable, focus UI et raccourcis configurables de base. | Fait | Dix-huit commandes ; AZERTY/QWERTY, Maj, refus visible et échange atomique. R75 conserve les entités UI, le focus, les positions et la surbrillance après attribution/conflit. Clic+frappe, Entrée consommée, persistance et pilotage remappé testés. Voir D108–D109 et VALIDATION. |
| CHR-09 | Créer caméra de suivi amortie en secondes, orbit de construction et transition de mode. | Fait | Suivi amorti en secondes, orbit et modes ; caméra ne modifie aucune pose physique. |
| CHR-10 | Réutiliser Knight et animations utiles ; prévoir capsule visible de remplacement en cas d'asset absent. | Fait | Knight Idle/Walk/Jump ; absence du GLB éprouvée dans les deux navigateurs, représentation de secours et marche utilisables. |
| CHR-11 | Créer récupération joueur hors limites et après perte de support ; dernier quai valide. | Fait | Récupération hors limites et R au dernier quai ; construction conservée, fragments abandonnés. |
| CHR-12 | Rejouer scène embarquée ou poste fixe dans les deux builds ; publier limitations connues. | Fait | Scénarios intégrés natif/web incarnent puis sauvegardent/rechargent le personnage. |

## DAT

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| DAT-01 | Définir IDs stables de corps, blocs, matériaux et composants ; séparer IDs du domaine et Entity Bevy. | Fait | BodyId et catalogues sérialisables distincts des Entity/Handle. |
| DAT-02 | Définir conventions cellule-centre, axes, rotations autorisées, mètres et conversions monde/local. | Fait | Cellules centrées de 0,5 m ; transformations tournées et coordonnées négatives testées. |
| DAT-03 | Créer ChunkCoord et CellCoord avec division euclidienne et indexation 16³. | Fait | Division euclidienne 16³ ; cas limites et extrêmes i32 testés. |
| DAT-04 | Implémenter chunks denses alloués à la demande dans une table sparse ; itération stable d'export. | Fait | Chunks denses dans BTreeMap sparse ; libération du vide et export stable. |
| DAT-05 | Définir catalogue minimal bois/métal/verre avec masse, visibilité, collision et matériau distincts. | Fait | Bois/métal/verre : masses, occlusion et matériaux explicites. |
| DAT-06 | Définir composants fonctionnels : sustentateur, réservoir, voile, poste et harpon ; empreintes et attaches. | Fait | Poste, voile, sustentateur, réservoir, harpon ; empreintes orientées et supports validés. |
| DAT-07 | Introduire BodyDefinition, pose de session et révision ; mettre limites 10 000 cellules/128 par axe. | Fait | Construction privée, révision, plafonds 10k cellules/128 par axe/coordonnées bornées. |
| DAT-08 | Définir commandes Add/Remove/PlaceComponent et résultat transactionnel avec cellules/chunks affectés. | Fait | propose partagé aperçu/commit ; transaction refusée sans mutation, chunks affectés retournés. |
| DAT-09 | Adapter les fixtures PHY au domaine ; construire une table BodyId vers entités de simulation. | Fait | Fixtures du domaine et résolution explicite des IDs lors du chargement. |
| DAT-10 | Extraire calculs masse, centre et tenseur d'inertie des voxels et composants, avec unités. | Fait | Masse, centre et inertie analytique des cellules et équipements. |
| DAT-11 | Définir snapshots immuables pour rendu et calculs asynchrones ; révision corps/chunk dans chaque résultat. | Fait | Arc de snapshot immutable ; révisions avant application des résultats. |
| DAT-12 | Revue des invariants et tests sans Bevy renderer ; figer contrats initiaux et échelle D05. | Fait | Tests purs du domaine, dont équivalence volume et 9 000 intersections du terrain fusionné. |

## MSH

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| MSH-01 | Définir prédicat de face visible selon matériau source/voisin, avec règles air/opaque/verre. | Fait | Prédicat air/opaque/verre testé ; verre ne cache pas l’opaque. |
| MSH-02 | Implémenter greedy simple par masque de tranche, ordre déterministe et partition par matériau. | Fait | Greedy déterministe par masque, normales et winding testés. |
| MSH-03 | Fournir halo voisin de chaque chunk et invalider les faces voisines après édition sur frontière. | Fait | Voisins des vrais chunks lus ; invalidation des faces aux frontières. |
| MSH-04 | Transporter coordonnées de surface locales continues et orientation des faces indépendantes des UV finaux. | Fait | UV en mètres locaux, invariants à translation/rotation du corps. |
| MSH-05 | Convertir résultats en meshes Bevy avec handles réutilisés ; centraliser matériaux et textures du catalogue. | Fait | Palette partagée, handles et racine runtime ; aucun chemin source obligatoire. |
| MSH-06 | Séparer rendu opaque/verre ; choisir paramètres transparents simples et documenter limites de tri. | Fait | Mesh/verre séparé, culling désactivé, limites du tri alpha documentées. |
| MSH-07 | Créer matériaux sobres de départ et fallback ; brancher les textures bois existantes par données. | Adapté | Bois PBR commun et couleurs du catalogue ; cartes essentielles manquantes donnent Error/Retry, pas un fallback silencieux. |
| MSH-08 | Créer file de chunks sales, dédoublonnage, priorité caméra/édition et annulation logique des anciens jobs. | Fait | Corps édité prioritaire puis distance caméra ; file stable et dédoublonnée. |
| MSH-09 | Exécuter meshing en tâches natives sans partager de mutation ; appliquer les résultats au bon tick/frame. | Fait | Test du vrai ordonnanceur : résultat récent puis obsolète, révision récente conservée. |
| MSH-10 | Fournir chemin web coopératif avec tranches de travail bornées, sans supposer Rayon multithread. | Fait | WASM coopératif : 96 masques bornés/chunk et budget 1,5 ms par frame. |
| MSH-11 | Instrumenter temps mesh, application/upload, triangles et latence commande-vers-image. | Adapté | CPU maillage/application/file et passes GPU instrumentés ; 20 éditions input→PNG donnent une borne haute, pas un délai écran/photon. |
| MSH-12 | Tester scènes 1k/10k et stress 50k, adopter greedy sauf seuil D07 réellement dépassé. | Fait | Benchmark 1k/10k/50k répartis ; greedy simple conservé, plafond public inchangé. |

## EDT

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| EDT-01 | Calculer viewport interactif depuis layout UI ; exclure pointeurs capturés par panneaux et boutons. | Fait | Interaction UI bloque la construction ; 720p, 1080p et petite fenêtre testées. |
| EDT-02 | Implémenter DDA local avec intervalle de rayon et filtre AABB des corps ; règles aux frontières. | Fait | DDA local borné par AABB ; parallèles, intérieur, négatifs et corps tournés testés. |
| EDT-03 | Montrer aperçu d'ajout/retrait, face ciblée et raison du refus ; distinguer clic et déplacement caméra. | Fait | Aperçu commun aux règles, ajout/retrait/refus, seuil clic/drag et orbit séparés. |
| EDT-04 | Relier outils Add/Remove aux commandes validées ; décrire blocage du dernier élément fonctionnel requis. | Fait | Ajout/retrait transactionnels ; départ refusé sans fonctions requises. |
| EDT-05 | Ajouter placement/rotation des composants à empreinte ; afficher attaches et conflits. | Fait | Rotation par quart de tour, empreinte complète et conflits visibles. |
| EDT-06 | Implémenter historique borné de transactions et redo ; invalider redo seulement après nouvelle édition. | Fait | Historique 128 transactions/16 Mio ; 100 éditions et annulations vérifiées. |
| EDT-07 | Passer édition/pilotage au quai ; mettre à jour masse/collider/mesh en transaction avant reprise. | Fait | Atelier au quai ; reconstruction collider/masses/meshes avant départ. |
| EDT-08 | Définir graphe structurel par voisinage de faces et liens de composants ; calculer connexité exacte. | Fait | Connexité exacte par faces ; pont et boucle distante testés. |
| EDT-09 | Préparer scission sur snapshot, nouvelles origines/IDs et transfert des composants/attaches. | Fait | Scission sur blueprint, IDs neufs, composants répartis et masse conservée. |
| EDT-10 | Appliquer scission et vitesses de corps rigide : v fragment = v parent + ω × décalage du centre. | Fait | Origine locale conservée ; vitesse du fragment calculée au nouveau centre de masse. |
| EDT-11 | Borner débris actifs et durée de calcul ; définir suppression d'une attache et annulation au quai. | Fait | 32 corps maximum ; attachages libérés, undo flotte borné au quai. |
| EDT-12 | Rejouer construction complète, reset récupérable, scission et undo au quai sur natif/web. | Fait | Scission, undo/redo et reprise vérifiés par QA native et navigateur. |

## SAV

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| SAV-01 | Définir blueprint v1 : unités, catalogue, cellules, composants, orientations et attaches ; ordre stable. | Fait | Blueprint v1 et fixture asymétrique avec rotation dans examples-data. |
| SAV-02 | Définir session v1 : poses/vitesses, acteur/mode, progression, réservoir et connexions ; enveloppe versionnée. | Fait | Session v1 : poses, consignes, réserves, acteur, progression et câble. |
| SAV-03 | Implémenter encode/decode avec plafonds de taille, cellules, corps et valeurs finies. | Fait | 16 Mio maximum, 32 corps, 10k cellules par corps ; validation des valeurs/IDs. |
| SAV-04 | Construire validation sémantique transactionnelle et staging de chargement. | Fait | Decode et validation avant remplacement ; import corrompu ne modifie pas la session. |
| SAV-05 | Écrire adaptateur disque : temporaire, flush selon garantie choisie, remplacement Windows et copie de secours. | Fait | Douze interruptions injectées aux frontières backup/principale ; au moins une sauvegarde valide conservée. |
| SAV-06 | Créer adapter IndexedDB avec quota/permission/indisponibilité gérés ; stockage après confirmation de transaction. | Fait | IndexedDB strict ; succès après oncomplete ; quota et abort testés. |
| SAV-07 | Implémenter export/import manuel JSON natif et web ; noms sûrs et messages de résultat. | Fait | Import/export JSON natif et web ; téléchargement et aller-retour navigateur vérifiés. |
| SAV-08 | Définir autosave temporisé aux frontières sûres, sérialisation snapshot et limite des écritures simultanées. | Fait | Autosave aux frontières sûres, mailbox unique ; mutations concurrentes d’identité bloquées. |
| SAV-09 | Écrire tests round-trip de corps déplacé/tourné, composants et valeurs de session ; prévoir migration testée. | Fait | Round-trip tourné/déplacé, valeurs, version inconnue et premières v1 testés. |
| SAV-10 | Restaurer session en phases : données, corps, composants, attaches, acteur, puis reprise des ticks. | Fait | Restauration corps→attaches→acteur ; intentions remises à zéro. |
| SAV-11 | Tester sauvegarde en cours de fermeture, quota saturé, fichier tronqué et recovery depuis backup. | Fait | Backup disque/web, JSON tronqué, quota, transaction abortée et fermeture native exercés. |
| SAV-12 | Valider D17/D18 sur package déplacé et navigateur rechargé ; livrer exemples versionnés. | Fait | Flotte maximale encode/decode ; package déplacé et navigateur rechargé ; exemples versionnés. |

## FLT

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| FLT-01 | Définir enveloppe de vol : masse, vitesses, altitude locale, rotation et limites de forces. | Fait | tuning.rs : coefficients SI, plafonds de forces/rotation/altitude versionnés. |
| FLT-02 | Créer champ de vent constant puis variation douce déterministe par graine et temps physique. | Fait | Vent seedé, temps SimClock à 60 Hz ; variation continue. |
| FLT-03 | Échantillonner vent relatif à la vitesse du point d'application, incluant rotation du navire. | Fait | Vitesse linéaire + ω×r au point de voile dans le vent relatif. |
| FLT-04 | Appliquer poussée simplifiée selon aire/orientation, avec coefficients bornés et force au bon point. | Fait | Poussée selon normale/aire ; plafond et test face/profil. |
| FLT-05 | Implémenter réglage de voile et commande de direction avec autorité limitée et coût physique explicite. | Fait | Réglage de voile et couple de direction limités ; aucune téléportation de pilotage. |
| FLT-06 | Ajouter traînée linéaire/angulaire temporellement cohérente et limites de confort. | Fait | Traînée et frein au tick ; invariance FPS testée. |
| FLT-07 | Fournir interface de sustentation au composant Aether : force verticale demandée et point d'application. | Fait | Demande verticale PD appliquée au centre des sustentateurs ; indépendante de la voile. |
| FLT-08 | Ajouter indicateurs de vent, vitesse, altitude et direction réellement utiles au pilotage. | Fait | HUD vent/vitesse/altitude/trim, cap et distance du quai ; particules de direction. |
| FLT-09 | Construire trajet simple de deux quais et contrôler départ/arrivée, freinage et accostage. | Fait | Traversée aux intentions physiques et aux touches navigateur ; seuil <9 m et <3,5 m/s. |
| FLT-10 | Comparer deux constructions prescrites : masse ajoutée et voile déplacée ; enregistrer effets. | Fait | Masse/consommation et voile décalée ±0,5 m mesurées ; COMPARAISONS.md. |
| FLT-11 | Régler confort : mouvements caméra, accélération initiale et commandes ; essai interne court. | Fait | Traversées aux intentions et touches, caméra amortie/réduction du mouvement ; appréciation humaine réservée à PLY. |
| FLT-12 | Valider première traversée technique avec sustentation de fixture puis préparer branchement Aether réel. | Fait | Traversée finale directement éprouvée avec sustentation Aether réelle. |

## AET

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| AET-01 | Définir quantité/capacité d'Aether en unités de jeu, débit de consommation et états du sustentateur. | Fait | Unités de jeu distinctes ; capacité et quantité bornées. |
| AET-02 | Relier réservoir unique du navire aux consommateurs par référence validée, sans réseau spatial. | Fait | Réserve par corps issue des réservoirs ; zéro capacité désactive le vol. |
| AET-03 | Calculer demande de sustentation, capacité maximale et force disponible selon masse et commande. | Fait | Demande limitée à 22 kN/sustentateur ; coque lourde testée sans flottement artificiel. |
| AET-04 | Débiter la consommation au tick selon commande choisie ; arrêter force à épuisement sans quantité négative. | Fait | Débit au tick et force réduite à l’épuisement ; invariant FPS/pause. |
| AET-05 | Ajouter recharge au quai avec déclenchement clair, débit borné et arrêt à capacité. | Fait | Recharge bornée de 25 unités/s au quai ; test de non-déplacement et recharge. |
| AET-06 | Donner aux composants une représentation violette et un indicateur de quantité réel. | Fait | Équipements violets et barre de réserve avec texte chiffré. |
| AET-07 | Signaler réserve basse, déficit de sustentation et panne ; ajouter action de secours cohérente. | Fait | Avertissements réserve/déficit, son et R de secours documenté. |
| AET-08 | Persister quantité, état et recharge dans session ; blueprint stocke capacité/configuration sans dupliquer du carburant. | Fait | Session contient réserve ; blueprint/undo/import ne créent pas de carburant. |
| AET-09 | Définir réservoir supprimé/scindé et répartition du contenu ; borner inventaire perdu/récupéré. | Fait | Retrait plafonne la réserve ; scission répartit proportionnellement aux capacités. |
| AET-10 | Tester panne en vol et retour à un quai/checkpoint ; prévenir une reprise irrémédiablement bloquée. | Fait | Panne coupe sustentation ; secours recrée la construction au dernier quai. |
| AET-11 | Régler capacité initiale et coût du parcours ; comparer coque légère et chargée. | Fait | Réserve et panne mesurées sur navire chargé ; recharge et récupération testées. Frustration perçue à observer avec PLY. |
| AET-12 | Effectuer traversée avec Aether réel, save/reload et panne volontaire ; fermer J3. | Fait | Traversée physique réelle, reprise/câble/personnage et secours dans les scénarios intégrés. |

## CAB

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| CAB-01 | Dès J1, vérifier API/features DistanceJoint sur coque de fixture ; créer ancre fixe et longueur maximale. | Fait | DistanceJoint min=0/max=L ; absence de compression et dix minutes bornées. |
| CAB-02 | Représenter ancrage fixe et harpon par IDs et points locaux, avec cycle attaché/détaché. | Fait | Harpon local et index d’ancre persistés ; IDs validés. |
| CAB-03 | Créer ciblage à portée et ligne de vue ; feedback valide/invalide ; limite initiale un câble actif. | Fait | Portée 70 m, visibilité exacte contre terrain, un seul câble. |
| CAB-04 | Attacher au tick sans imposer instantanément une longueur incompatible ; borner paramètres autorisés. | Fait | Longueur initiale égale à la distance ; attache sans repositionner le corps. |
| CAB-05 | Libérer ou couper le câble en conservant l'état physique courant ; effacer le joint une seule fois. | Fait | Suppression du joint seule ; test de continuité des vitesses au lâcher. |
| CAB-06 | Lire l'effort disponible dans le solveur ou définir un indicateur approché clairement nommé. | Fait | Indicateur géométrique souple/tendu ; aucune force prétendue en newtons. |
| CAB-07 | Rendre câble lâche/tendu entre points interpolés, avec couleur et son de tension. | Fait | Courbe visuelle souple/tendue, couleur et signal audio de transition. |
| CAB-08 | Définir câble traversant un obstacle : bloquer attache initiale, avertir ou rompre selon règle du prototype. | Fait | Attache initiale occultée refusée ; obstacle ultérieur libère le câble ; aucun wrapping. |
| CAB-09 | Gérer destruction/scission de la cellule d'attache et suppression du corps ; transférer ou libérer. | Fait | Suppression/scission libère les attaches ; nettoyage avec la session. |
| CAB-10 | Sauver et restaurer câble après reconstruction des corps ; ID stable et longueur validée. | Fait | Chargement après corps, validation du harpon et de la longueur ; QA save/load attaché. |
| CAB-11 | Éprouver masse maximale, vitesse maximale, pause/reprise et 30/60/120 FPS ; enregistrer erreur de longueur. | Fait | 54 combinaisons masse 1k/10k/900k, vitesse 0/25/80, longueur 3/80, FPS 30/60/120 ; pause et reprise incluses. |
| CAB-12 | Régler une manœuvre de balancier autour d'ancre ; appliquer repli borné si échec. | Fait | Balancier vers refuge avec accroche/lâcher réels, sans téléportation, scénario headless. |

## CUR

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| CUR-01 | Définir trajet 3D du courant avec segments/tube, rayon, direction, intensité et zones de transition. | Fait | Tubes de segments 3D avec rayon, vitesse et mélange bornés. |
| CUR-02 | Implémenter échantillonnage continu de vitesse cible et poids d'influence près des joints de segments. | Fait | Champ continu et nul hors volume ; test de frontière. |
| CUR-03 | Convertir champ en force de relaxation bornée sur la vitesse relative, avec constante de temps explicite. | Fait | Relaxation 2 s, accélération maximale 8 m/s² ; forces uniquement. |
| CUR-04 | Définir quelles entités reçoivent le courant et éviter double application au personnage attaché. | Fait | Forces sur Vessel dynamique seulement ; personnage et ancres exclus. |
| CUR-05 | Composer un embranchement avec choix de trajectoire et destination de récupération pour branche ratée. | Fait | Jardins et refuge atteints avec physique/obstacles de production. |
| CUR-06 | Placer ancre et obstacles lisibles autour de l'embranchement ; régler portée et fenêtre d'action. | Fait | Approche sans téléportation, portée et visibilité réelles ; retard 0/0,5/1,5 s avant accroche, refuge atteint et réserve positive. |
| CUR-07 | Dessiner ruban violet, particules de direction et variation de densité au cœur du courant. | Fait | Courant lumineux orienté selon le champ, texte et indicateurs ; couleur seule jamais nécessaire. |
| CUR-08 | Assembler deux îles, quais et repères de navigation ; préserver lignes de vue du trajet et du but. | Fait | Trois îles, quais, ruines et observatoire ; géométrie et visibilité du domaine partagées avec Blender. |
| CUR-09 | Définir progression départ, première traversée, manœuvre, arrivée ; événements persistants. | Fait | Étapes de progression persistées, arrivée reconnue au quai. |
| CUR-10 | Ajouter checkpoint sûr et réparation/ravitaillement entre essais ; garder coût d'échec court. | Fait | Checkpoint et récupération de la coque principale avec recharge. |
| CUR-11 | Tester combinaisons courant/voile/câble, réserve faible et navire chargé dans le même scénario. | Fait | Navire chargé combinant voile, courant, câble, faible réserve puis panne ; fixture intégrée réussie. |
| CUR-12 | Jouer scénario complet sans commandes de debug et préparer capture de référence J4. | Fait | Traversée navigateur aux touches réelles, sonde uniquement en lecture ; capture d’arrivée. |

## UX

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| UX-01 | Définir thème partagé, hiérarchie des actions et vocabulaire français/clé de texte centralisé. | Fait | Thème partagé, interface française et terminologie cohérente ; pas de système i18n général. |
| UX-02 | Rendre panneaux adaptatifs au viewport et à l'échelle UI ; min-size et état compact explicites. | Fait | Échelle UI selon fenêtre ; boutons dans le viewport à 960×640. |
| UX-03 | Afficher raccourcis contextuels, verrouillage quai/pilotage et explications de refus de commande. | Fait | Raccourcis contextuels et refus dans le footer. R75 : voyage rapide par pause/atlas, toutes les îles et lieux remarquables répartis en quatre catégories paginées ; collision d'arrivée vérifiée avant mutation, retour au pilotage sans réinitialiser les ressources. Voir D110 et VALIDATION. |
| UX-04 | Créer tutoriel par actions réelles : bloc, voile, énergie, départ, câble ; étapes rejouables. | Fait | Huit étapes observées dans les actions réelles, persistées et rejouables ; navigateur vérifie les huit bits. |
| UX-05 | Ajouter HUD compact de réserve, sustentation, vitesse, objectif et état de câble. | Fait | HUD réserve, déficit, navigation, objectif et état du câble. |
| UX-06 | Ajouter réglages volume, sensibilité, inversion et caméra ; persister préférences séparées de la session. | Fait | Volume, sensibilité, inversion, réduction du mouvement ; préférences séparées et validées. |
| UX-07 | Préparer feedback audio utile : pose/refus, tension, réserve et arrivée, avec provenance des sons. | Fait | Quatre sons originaux reproductibles, provenance et volume ; messages doublent les alertes. |
| UX-08 | Vérifier contrastes, texte, sens du vent et états indépendants de la couleur ; réduire mouvements caméra. | Fait | Contraste sRGB ≥4,5:1, focus et états compris ; informations doublées par texte, caméra réduite. |
| UX-09 | Centraliser PBR du bois, sampler et modes debug ; supprimer paths de compilation et doublons runtime. | Fait | Cèdre albedo/normal, toile indigo, UV métriques, mipmaps en lumière linéaire, anisotropie et éclairage d’environnement. |
| UX-10 | Créer galerie canonique : corps tourné, verre, vrais chunks, coque et composants, plusieurs lumières utiles. | Fait | Galerie partagée avec coque, composants, vitrage tourné et chunks négatifs ; capture/timeout/code de sortie. |
| UX-11 | Capturer et relire les fixtures ainsi qu'une scène de jeu à vitesse normale. | Fait | Captures natives et navigateur inspectées ; raccords, répétition bois, volumes et panneaux revus. |
| UX-12 | Valider compréhension de l'interface lors d'un essai interne sans explication orale. | Externe | Compréhension autonome sans explication à évaluer avec une personne ; un robot ne remplace pas cet essai. |

## WEB

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| WEB-01 | Séparer assets runtime et sources d'auteur ; produire manifeste avec tailles et empreintes. | Fait | Assets runtime séparés, manifestes SHA256, tailles et notices de provenance. |
| WEB-02 | Produire package Windows autonome et définir répertoires lecture seule/données utilisateur. | Fait | Package Windows déplacé hors dépôt ; assets relatifs exe et données utilisateur distinctes. |
| WEB-03 | Configurer bundle Bevy CLI avec préfixe de déploiement ; fallback wrapper seulement après 4 h de blocage. | Fait | Bundle CLI testé à la racine et sous préfixe ; serveur Windows/Node. |
| WEB-04 | Versionner noms/manifestes du JS, WASM et assets ; éviter mélange de versions après déploiement. | Fait | Ensemble immuable r/version-empreinte et launcher non caché ; manifeste de chaque livraison. |
| WEB-05 | Afficher phases téléchargement, initialisation et première image jouable ; gérer erreurs et retry. | Fait | Vérification GPU, chargement module/initialisation, Loading assets, erreurs/retry/timeout ; aucun pourcentage fictif. |
| WEB-06 | Définir URL initiale et retour menu cohérents avec une seule application ; traiter back/forward. | Fait | Retour/avance historique avec rechargement et sauvegarde conservée ; pas de certification bfcache. |
| WEB-07 | Tester clavier/souris, focus, redimensionnement, plein écran et permissions de téléchargement. | Fait | Clavier/souris, focus, plein écran, redimensionnement 960×640 et suspension JS 65 s éprouvés. |
| WEB-08 | Mesurer bundle compressé, mémoire et démarrage froid/chaud ; identifier le plus gros contributeur. | Fait | Tailles Brotli, mémoire linéaire WASM, démarrage froid/chaud à 50 Mbit/s mesurés. |
| WEB-09 | Ajouter smoke Playwright sur menu, première image, construction, save/load et retour ; conserver erreurs dès le début. | Fait | Quinze tests dans Chrome et Edge : parcours, QA, erreurs, stockage, commandes, tutoriel, focus, historique, plein écran et voile animée. |
| WEB-10 | Préparer pipeline de release : réutilisation artefact validé, checksums, numéro de version et notes. | Adapté | Packages reproductibles, checksums et source-manifest ; empreinte remplace commit, aucune opération Git. |
| WEB-11 | Vérifier sauvegardes entre deux builds compatibles et préparer restauration de version précédente. | Fait | Première v1 et v1 actuelle compatibles ; entrée non cachée basculée nouveau → ancien → nouveau entre deux builds réels, IndexedDB conservé et assets cohérents. |
| WEB-12 | Revue de package natif/web, capacités affichées et procédure de publication pour décision ultérieure. | Fait | Archives Windows/web, licences, contrôles et procédure RELEASE ; aucune publication. |

## QA

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| QA-01 | Constituer matrice courte des parcours critiques : nouvelle partie, construire, traverser, câble, échec, reprise. | Fait | Matrice et preuves dans VALIDATION ; automatisation distinguée des observations humaines. |
| QA-02 | Exécuter scénario intégré de 30 minutes avec navigation, édition au quai et sauvegardes répétées. | Fait | Session rendue ≥30 min, cycles édition/undo/redo/save/load/navigation/câble/marche/recovery ; sonde en lecture seule. |
| QA-03 | Mesurer vingt cycles menu/session et cent éditions/annulations/scissions dans une fixture bornée. | Fait | Vingt recréations et cent éditions/annulations ; comptage des entités réellement actives, meshes, matériaux et file. |
| QA-04 | Vérifier synchronisation poses physiques, meshes, picking, ancrages et interpolation après scission/reprise. | Fait | Interpolation mesurée en mètres et reset téléportation ; points d’équipements et picking vérifiés après reprise/scission. |
| QA-05 | Rejouer intentions enregistrées dans le même build et comparer invariants, énergie bornée et progression. | Fait | Intentions physiques répétées et ticks exacts ; tolérances documentées, aucun déterminisme inter-CPU promis. |
| QA-06 | Renforcer validation des entrées : JSON tronqué, IDs répétés, valeurs non finies, dimensions extrêmes et jobs obsolètes. | Fait | Tests génératifs bornés, i32 extrêmes, JSON/IDs/limites, poids et fichiers invalides refusés. |
| QA-07 | Mesurer scène cible à 1080p : navire 10 000 cellules, au plus 32 corps actifs, îles et effets du parcours. | Fait | Baseline finale 1080p 10k cellules/32 corps, CPU/contact/triangles, GPU natif et mémoire. GPU web absent explicitement. |
| QA-08 | Tester dépassement des limites : 50 000 cellules en fixture de stress, import hors borne, trop de débris et backlog de meshing. | Fait | Stress 50k répartis, plafond d’import et flotte maximale testés ; limite publique conservée. |
| QA-09 | Injecter échec disque/quota web, perte de focus, fermeture pendant save et asset absent dans le package. | Fait | Stockage refusé/interrompu, backup, fermeture native, assets absents et suspension JS prolongée testés. |
| QA-10 | Exécuter matrice Windows natif, Chrome WebGPU et Edge WebGPU ; documenter pilotes/résolutions testés. | Fait | Windows natif, Chrome et Edge réels ; versions et options d’automatisation consignées. |
| QA-11 | Sélectionner les régressions à maintenir en CI et conserver artefacts utiles sans captures inutiles à chaque commit. | Préparé | Régressions et logs sélectionnés dans verify/CI ; durée distante de CI inconnue tant que non activée. |
| QA-12 | Trier défauts et fermer le jalon technique : P0 perte/crash, P1 parcours bloqué, P2 gêne bornée, P3 cosmétique. | Fait | Aucun défaut P0/P1 observé dans les scénarios validés ; limites et défauts résiduels consignés dans DEFAUTS.md. |

## PLY

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| PLY-01 | Écrire protocole de 10–15 minutes sans assistance : construire une amélioration, partir, utiliser courant/câble, revenir. | Fait | PLAYTEST : protocole 10–15 min, départ, amélioration, courant/câble et secours. |
| PLY-02 | Définir grille d'observation : temps jusqu'au départ, erreurs, compréhension de l'Aether et envie de modifier le vaisseau. | Fait | Grille vierge faits/paroles/interprétation et indicateurs de compréhension. |
| PLY-03 | Préparer build candidat, fiche de commandes et instructions de lancement pour les personnes choisies par Damien. | Fait | Candidat identifié, archives, commandes, lancement et invitation modèle sans envoi. |
| PLY-04 | Faire une répétition du protocole avec une personne connaissant peu la version pour déceler ses défauts. | Externe | Répétition avec une personne non réalisée ; fiche et build prêts. |
| PLY-05 | Observer cinq personnes séparément ; prendre notes et, avec leur accord, captures utiles du parcours. | Externe | Cinq personnes à observer ; aucune fiche ni satisfaction inventée. |
| PLY-06 | Consolider observations sans surinterpréter le petit échantillon ; classer fréquence, gravité et étape concernée. | Externe | Consolidation dépend des cinq observations réelles. |
| PLY-07 | Évaluer hypothèse centrale : une modification du vaisseau produit-elle un effet compris et une nouvelle tentative volontaire ? | Externe | Hypothèse de plaisir/compréhension à confronter aux observations, pas aux FPS. |
| PLY-08 | Choisir au plus trois corrections de parcours et une hypothèse de plaisir pour un cycle court ; estimer avant de modifier. | Externe | Trois corrections maximum à choisir sur les faits recueillis. |
| PLY-09 | Réaliser le cycle de corrections sélectionné, en conservant la portée de la démo. | Externe | Cycle de correction dépend du choix PLY-08 ; aucun changement spéculatif présenté comme validé. |
| PLY-10 | Faire un second essai ciblé avec au moins deux personnes disponibles, en distinguant nouveaux joueurs et joueurs déjà formés. | Externe | Second essai avec deux personnes après corrections ; non réalisable par automatisation. |
| PLY-11 | Préparer notes de version, contrôles, limitations connues, procédure de récupération et archive du candidat retenu. | Fait | Notes, contrôles, récupération, limites et archives de ce candidat technique. |
| PLY-12 | Décider avec Damien : démo prête, second cycle ciblé ou changement de priorité ; classer les extensions par bénéfice constaté. | Externe | Décision de produit avec Damien après essais ; dossier technique préparé, jalon humain non fermé. |

## FLEX

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| FLEX-01 | Formuler l'interaction manquante et une scène où le modèle simplifié échoue ; accepter ou refuser l'activation. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-02 | Isoler noyau flexible Rust avec `glam` aligné et états privés ; définir nœuds, contraintes et révisions de topologie. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-03 | Porter intégration séquentielle, masses, ancrages et amortissement en secondes ; conserver fixtures historiques utiles. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-04 | Commencer XPBD à 10 sous-pas × 1 itération ; mesurer erreur et coût sur une seule scène justifiée. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-05 | Implémenter corde unilatérale et tension exploitable ; distinguer mou, tendu et rupture. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-06 | Définir un seul transfert d'efforts avec Avian et l'ordre des sous-pas ; tester corps/nœud et deux corps. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-07 | Prototyper voile à grille grossière et forces de vent réparties ; rendre une surface plus détaillée sans changer la poussée. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-08 | Traiter suppression et rupture de contrainte comme transaction de topologie ; conserver historique et attaches valides. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-09 | Tester contacts simplifiés de corde seulement si la scène FLEX-01 les exige ; borner nombre de contacts et rayon. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-10 | Profiler avant toute parallélisation ; si nécessaire, établir groupes sans conflits et comparer à la référence séquentielle. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-11 | Mesurer le même graphe natif/WASM et sauvegarde/reprise avec un navire mobile. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |
| FLEX-12 | Faire essai joueur ciblé et décider intégration, maintien expérimental ou abandon. | À faire | La demande de toutes les fonctionnalités des planches autorise la poursuite de ce lot. Les morph targets actuels animent la voile ; ils ne constituent pas une simulation XPBD ni une corde avec contacts. Développement et critères FLEX restent ouverts ; ancien motif « déclencheur absent » remplacé. Aucun résultat de ce lot n’est déclaré fait. |

## OPT

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| OPT-01 | Capturer un profil d'édition lente et attribuer le coût : extraction, meshing, allocation, upload, collider ou attente. | Fait | CPU maillage/décoration/application/file et édition→PNG mesurés. Sonde 5 Hz corrigée (D41), profil CDP attribuant le délai résiduel aux tangentes décoratives (D45). Contacts Edge traités séparément ; COMPARAISONS. |
| OPT-02 | Si meshing dominant, brancher `binary-greedy-meshing` 0.5.2 sur fixture via adaptateur isolé. | Non applicable | Les coûts dominants étaient les contacts et les tangentes décoratives, pas le greedy ; aucune raison mesurée de remplacer ce dernier. |
| OPT-03 | Vérifier sorties de l'adaptateur : faces, matériaux, halos, UV, transparence et cellules de coordonnées négatives. | Non applicable | Aucune stratégie de meshing concurrente retenue ; comparaison artificielle non nécessaire. |
| OPT-04 | Mesurer gain complet et choisir conserver greedy simple ou intégrer adaptateur. | Non applicable | Greedy actuel conservé, priorité et ordonnanceur éprouvés ; pas de migration sans gain mesuré. |
| OPT-05 | Si mémoire des chunks domine, mesurer occupation puis prototyper palette locale ou stockage uniforme des chunks pleins/vides. | Conditionnel différé | Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan. |
| OPT-06 | Si l'occlusion améliore la lecture observée, ajouter AO par sommet avec règle de fusion compatible. | Adapté | SSAO/TAA du renderer partagé au lieu d’un AO par sommet, contrôlés en jeu et galerie. |
| OPT-07 | Si textures/draw calls dominent, essayer une seule évolution d'atlas ou d'array de textures sur le renderer partagé. | Conditionnel différé | Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan. |
| OPT-08 | Si le travail coopératif web dépasse durablement les budgets, évaluer workers/threads avec contraintes d'hébergement réelles. | Conditionnel différé | Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan. |
| OPT-09 | Si les colliders dominent les éditions, regrouper reconstructions et réutiliser résultats par révision de corps. | Conditionnel différé | Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan. |
| OPT-10 | Si le démarrage web est trop long, réduire le contributeur dominant identifié : features, compression ou assets initiaux. | Fait | Compression Brotli/gzip, ensemble versionné et démarrage froid/chaud mesurés. |
| OPT-11 | Si trop de débris actifs coûtent cher, éprouver sommeil, regroupement visuel ou expiration annoncée des fragments secondaires. | Conditionnel différé | Le profilage ne déclenche pas cette expérimentation particulière ; garder les seuils du plan. |
| OPT-12 | Mettre à jour baseline de performance et décision de chaque optimisation tentée ; retirer prototypes inutiles du chemin distribué. | Fait | Baseline finale et historique avant/après conservés ; aucun prototype concurrent dans le chemin distribué. |

## EXT

| ID | Travail du plan | État | Réalisation, preuve ou suite nécessaire |
| --- | --- | --- | --- |
| EXT-01 | Choisir une extension à partir des observations et rédiger hypothèse, scène minimale, plafond et condition d'abandon. | Fait | Demande explicite de Damien : monde vaste inspiré des 41 planches, propulsion praticable et détail élevé. Périmètre, critères et limites dans MONDE-OUVERT, RENDU et DECISIONS. |
| EXT-02 | Si import `.vox` utile, intégrer `dot_vox` 5.2 avec limites et mapping palette/axes ; traiter modèles et origine explicitement. | Conditionnel différé | Déclencheur du plan absent : aucune observation joueur ne justifie cette extension. D22/D23 ; lot non activé. |
| EXT-03 | Si échange externe utile, exporter `.vox` puis faire aller-retour réel dans MagicaVoxel sur fixture asymétrique. | Conditionnel différé | Déclencheur du plan absent : aucune observation joueur ne justifie cette extension. D22/D23 ; lot non activé. |
| EXT-04 | Prototyper réseau Aether par segments connectés avec capacité commune et débits bornés, sur un navire de test. | En cours | Atelier logique V3, vannes et réserve raccordés à la coque. Suites complètes Chrome/Edge R67f17/17 et migration réelle V1→V2→V3 PASS. Fondation spatiale R72 pure125mm/280mm, six tests PASS ; kit industriel R69 chargé HD/LOD isolément. Raccords réels, pose des conduites, collisions et dommages restent ouverts (D103–D105, CONDUITES). |
| EXT-05 | Étudier coupure, fuite et scission du réseau ; définir distribution de quantité lors des changements de topologie. | En cours | Conservation entière sur10000 opérations, fuites/réparation/recharge, scission de vraie coque, plafond d’annulation exact et compteurs persistants. Workspace R72 complet158 tests PASS ; V3 rechargé par vrais binaires. Déclenchement des dégâts, transferts jouables, maillage et scission des conduites physiques restent ouverts. |
| EXT-06 | Seulement si l'eau volumique a une interaction nécessaire, prototyper automate borné dans un bassin statique. | À faire | Lot autorisé dans le périmètre des planches. Bassins et cascades actuels sont des représentations et points de collecte ; automate volumique conservatif, interactions et couplage physique restent à développer. Aucun résultat volumique annoncé. |
| EXT-07 | Prototyper un seul état de matériau motivé par le jeu, par exemple mouillé ou endommagé, avec effet visible. | À faire | Lot autorisé dans le périmètre des planches. Les fuites de réserve sont persistantes ; elles ne constituent pas encore un état de matériau mouillé ou endommagé ni une cause jouable de dommage. Effet visible, stockage et effet mécanique restent à développer. |
| EXT-08 | Étudier streaming d'îles si le nombre d'îles nécessaire dépasse la scène chargée ; charger/décharger deux régions de fixture. | Adapté | Catalogue de 204 îles plus trois quais ; collisions chargées par proximité avec hystérésis, représentation HD/LOD. Le trajet Aube→Hollow traverse les régions. Les actifs graphiques restent résidents : aucun streaming disque complet n'est revendiqué. |
| EXT-09 | Si erreur de précision observée loin de l'origine, quantifier seuil puis essayer repère flottant et version compatible de `big_space`. | Conditionnel différé | Déclencheur du plan absent : aucune observation joueur ne justifie cette extension. D22/D23 ; lot non activé. |
| EXT-10 | Prototyper génération reproductible d'une petite île utile au parcours, avec contraintes d'atterrissage et ressources lisibles. | Adapté | Modèles Blender rédigés et placement déterministe, avec quais, bassins, cristaux et portes partagés par le rendu et la physique. Catalogue et couloirs testés ; pas de génération de terrain infinie. |
| EXT-11 | Si coopération demandée, rédiger étude réseau bornée : autorité serveur, commandes d'édition, IDs, reconnexion et coût physique. | Conditionnel différé | Déclencheur du plan absent : aucune observation joueur ne justifie cette extension. D22/D23 ; lot non activé. |
| EXT-12 | Pour l'extension choisie, conclure conserver/abandonner, chiffrer mise en production et mettre à jour décisions actives. | En cours | R67f Chrome/Edge complets17/17 et migration réelle V1→V2→V3 PASS ; world-tour R60 dix étapes historique PASS. R72 workspace158 tests et QA140 contrôles PASS, Windows/Web compilables. R66 courants promus/global PASS ; maison entière R71 non promue après quatre captures. Dernier jugeR47 4/10 ; benchmark calme R72 p95 18,51 ms dépasse 16,7 ms. Endurance finale, fonctionnalités restantes des planches et paquet0.2 ouverts. |

## Suites dépendant d’observations ou de services externes

1. Réaliser le protocole avec une première personne puis cinq observations séparées ; exploiter les fiches sans présumer du plaisir.
2. Choisir les corrections sur ces faits puis conduire le second essai humain prévu.
3. Activer la CI distante lors d’une étape autorisant les opérations de dépôt. La même validation est exécutée localement ici.
4. Suivre les extensions activées par la demande de monde ouvert ; conserver un statut explicite pour les systèmes encore ouverts dans MONDE-OUVERT et les expérimentations sans déclencheur.
5. Étendre la matrice à d’autres machines et GPU lors de leur mise à disposition ; les résultats présents identifient précisément la machine éprouvée.
