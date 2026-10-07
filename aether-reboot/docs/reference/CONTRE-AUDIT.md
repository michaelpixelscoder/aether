# Contre-audit et décisions du reboot Aether Isles

1er octobre 2026. Référence du prototype : `54e5e0a`. Ce document examine la seconde review de Claude et corrige les recommandations précédentes. Il accompagne le [plan détaillé de développement](PLAN-DEVELOPPEMENT.md), qui constitue désormais la liste de travail. Le [rapport consolidé précédent](../reboot-final-2026-10-01.md) reste la référence pour l'inventaire du prototype et ses constats détaillés. Le développement du reboot n'a pas commencé.

## Vue de décision

**But : construire un petit vaisseau, comprendre ses réactions, utiliser un courant et un câble pour rejoindre une autre île, puis améliorer sa construction.** Rust reste le langage. Le plan couvre un premier jeu solo Windows avec démonstration web ; cette cible est une hypothèse de travail explicite, révisable par Damien.

Le choix de départ est Bevy 0.19, Avian 0.7, Tnua pour la marche, cellules de 0,5 m, chunks de 16³ cellules et greedy meshing simple. Aucun tournoi de bibliothèques n'est obligatoire. Une alternative ne s'ouvre qu'après un échec mesuré, avec une limite de temps et une décision consignée.

L'Aether intervient tôt par un réservoir et un dispositif de sustentation. Le réseau de tuyaux reste différé. Le câble initial utilise un joint de distance Avian avec une longueur maximale ; le solveur de tissu n'est pas une dépendance de la première traversée.

Quatre crates forment le socle : `aether_core`, `aether_sim`, `aether_view` et `aether_game`. La galerie dépend de `core` et `view`, jamais du jeu. `aether_flex` ne sera créé que lors de l'extension flexible. Les modules, contrats et tests font respecter ces frontières ; le nombre de crates ne garantit pas à lui seul la qualité ou la vitesse de compilation.

Le premier verrou à lever est l'environnement de compilation. Le premier risque de jeu est le personnage sur un pont mobile. L'essai Tnua dispose de **12 heures de travail effectif** ; si ses critères ne sont pas atteints, la première démo utilise le mode pilotage avec personnage attaché à un poste. Le jeu peut alors avancer, mais la marche embarquée reste explicitement non validée.

Le plan détaille **240 tâches : 204 dans le périmètre de la démo et 36 conditionnelles**. Ce volume sert à exposer les coûts et les dépendances, pas à imposer 240 fonctionnalités. Les 592 h nominales du périmètre démo sont une estimation initiale sans mesure de vélocité ; les extensions restent hors engagement. Les lots peuvent se chevaucher selon leurs prérequis et le jeu doit être essayé dès les premières fixtures pilotables.

## Ce que la review corrige à juste titre

Mon tableau précédent a parfois amplifié les formulations de Claude. Il proposait d'archiver le runner, pas de qualifier ce travail d'inutile ; il parlait d'un risque de comportement indéfini « latent », pas d'un défaut démontré ; son architecture réservait bien une couche Bevy/Avian aux corps. Il reportait les matériaux finaux, pas toute lisibilité visuelle. Les remarques sur `Resource` et `ActorIntent` étaient surtout des précisions communes. Ces points sont corrigés dans le document précédent.

La review identifie aussi un défaut réel de mon plan : comparer systématiquement plusieurs meshers, résolutions, solveurs et contrôleurs crée du travail sans critère d'arrêt. La présente version donne un choix par défaut concret par sujet et réserve les comparaisons aux échecs utiles.

La galerie dépendant du renderer situé dans la crate de jeu créait une frontière confuse. Le renderer devient `aether_view`. Une crate `aether_sim` regroupe l'intégration Avian et les forces ; le point d'entrée ne contient que la composition, les états et le scénario.

La compilation de l'ancien prototype est utile pour la référence, mais n'est pas un verrou pour le nouveau socle. Après installation de l'environnement, **deux heures au maximum** sont consacrées à sa remise en route. En cas d'échec, conserver sources, assets, logs et résultats de lecture, puis poursuivre le reboot. Aucun résultat historique n'est inventé.

## Corrections de mes propres recommandations

| Point antérieur | Limite reconnue | Correction opérationnelle |
| --- | --- | --- |
| Garder Bevy 0.18 pour commencer | Trop conservateur pour un nouveau workspace disposant d'une stack compatible | Bevy 0.19 / Avian 0.7 ; verrouiller la résolution qui compile |
| Persistance et éditeur avant le risque de navigation | Pouvait consacrer beaucoup de travail à une base physique non éprouvée | Essais corps, marche embarquée et câble avant l'éditeur complet |
| Trois crates avec tout Bevy dans `aether_game` | Séparation du renderer insuffisamment définie | Quatre crates avec dépendances explicites et galerie indépendante |
| Alternatives systématiquement comparées | Coût de recherche sans déclencheur | Défauts choisis, seuils, essais courts et replis |
| Pas de durée maximale | Risque de prolonger indéfiniment une preuve technique | Budgets d'investigation ci-dessous et point de décision à chaque sortie |
| PBR distribué et UI adaptative omis au premier passage | Audit initial incomplet | Tâches explicites package hors checkout, viewport réel et échelles UI |
| Contraste excessif avec le rapport de Claude | Attribution inexacte de certains désaccords | Distinguer accord, précision et correction ; conserver les preuves utiles |
| `min_by_key` ligne 563 | Référence erronée | Vérifié par `rg` : `aether_voxels/src/lib.rs:557` au commit audité |

## Points de la review à préciser à leur tour

**Tnua n'a pas une version unique.** Le tableau officiel associe Bevy 0.19 à `bevy-tnua` **0.32**, à `bevy-tnua-avian3d` **0.12** et à la couche d'intégration **0.13**, avec Avian 0.7. La formule « Tnua 0.12 » confond le contrôleur et son adaptateur. Ces compatibilités documentées restent à compiler dans notre environnement. [Versions Tnua](https://github.com/idanarye/bevy-tnua#versions).

**Toutes les bibliothèques n'ont pas une version Bevy à respecter.** `dot_vox` 5.2.0 ne dépend pas de Bevy à l'exécution. `binary-greedy-meshing` 0.5.2 n'a pas de dépendances runtime ; son Bevy 0.16.1 concerne ses exemples/tests. Leur intégration doit être testée, mais une version Bevy différente dans les dépendances de développement n'est pas un conflit pour Aether. En revanche, `big_space` 0.12.0 dépend réellement des sous-crates Bevy 0.18 : cette version n'est pas directement le choix du socle 0.19. Cette dépendance reste différée. [Manifeste dot_vox](https://docs.rs/crate/dot_vox/5.2.0/source/Cargo.toml), [manifeste binary greedy](https://docs.rs/crate/binary-greedy-meshing/0.5.2/source/Cargo.toml), [manifeste big_space](https://docs.rs/crate/big_space/0.12.0/source/Cargo.toml).

**L'alignement des maths est utile.** Le manifeste de `bevy_math` 0.19.0 utilise `glam` 0.32.0. Le socle emploiera directement `glam` 0.32 dans `aether_core`, avec résolution commune contrôlée par Cargo ; aucune conversion de vecteurs entre deux versions ne doit traverser nos interfaces. [Manifeste Bevy Math](https://raw.githubusercontent.com/bevyengine/bevy/v0.19.0/crates/bevy_math/Cargo.toml).

**Un pas de représentation f32 n'est pas une erreur physique garantie.** Le calcul local donne 0,244140625 mm entre deux valeurs f32 à 4 000 m, et 0,48828125 mm à 4 096 m. L'ordre de grandeur cité par Claude est raisonnable ; rotations, soustractions et solveurs peuvent amplifier les erreurs. Conserver un petit archipel près de l'origine et mesurer les contacts suffit à ce stade.

**Un seuil de 16 ms doit préciser ce qui est mesuré.** Ce seuil peut représenter une image entière à 60 FPS, pas le budget disponible pour un mesher qui partagerait le thread principal. Ici, on distingue mesh CPU d'un chunk, travail principal par image et latence commande-vers-image. Une seule pointe ne déclenche pas une réécriture.

**Les replis changent parfois le produit.** Fixer le joueur à un poste permet de valider navigation et construction ; cela ne valide pas la marche sur le pont. Une position simplement parentée au navire tout en restant un corps dynamique peut provoquer une double autorité. Le repli utilise un état de pilotage explicite, avec règles d'entrée/sortie.

**Les limites d'étape sont des règles de décision, pas des promesses de livraison.** Les tâches ont une charge initiale estimée. Les budgets courts s'appliquent aux recherches et problèmes bloquants ; une expiration impose un repli ou un arbitrage visible, jamais de déclarer un test réussi ou de baisser silencieusement son exigence.

**La lecture de `AGENTS.md` par Claude Code dépend de la version et de la configuration.** La documentation actuelle décrit une lecture directe à partir de certaines versions, ainsi que l'import `@AGENTS.md` depuis `CLAUDE.md`. La recommandation robuste est une seule source de règles, et un fichier d'import si nécessaire après vérification de l'outil installé. Aucun doublon de contenu à entretenir. [Documentation officielle Claude Code](https://code.claude.com/docs/en/memory#use-an-existing-agents-md).

## Registre unique des décisions

Ces choix sont les options proposées par défaut pour l'exécution du plan. « Sortie » signifie le jalon où le choix est vérifié ; les seuils sont des objectifs de départ à mesurer sur la machine de référence, pas des performances déjà obtenues.

| ID | Sujet et défaut | Révision seulement si | Sortie et repli |
| --- | --- | --- | --- |
| D01 | Rust ; Bevy 0.19 et Avian 0.7 | Incompatibilité reproductible avec une fonction indispensable | ENV-12 ; 4 h de diagnostic, version corrective compatible épinglée, arbitrage si impossible |
| D02 | Windows natif, solo ; web Chrome/Edge desktop WebGPU | Le matériel de référence n'expose pas les capacités nécessaires | APP-06 ; essai WebGL2 borné à 4 h, sinon cible annoncée explicitement limitée |
| D03 | `core`, `sim`, `view`, `game` ; aucune dépendance de `view` vers `game` | Cycle ou responsabilité impossible à tester isolément | APP-12 ; déplacer le contrat, pas ajouter une façade vide |
| D04 | `glam` 0.32 dans le domaine | Cargo révèle des types publics incompatibles | ENV-05 ; alignement via dépendance workspace ou `bevy_math` minimal |
| D05 | Grille 0,5 m ; chunk 16³ avec halo pour meshing | Construction réellement illisible ou trop coûteuse sur la fixture choisie | DAT-12 et MSH-12 ; un seul essai ciblé, pas une matrice de résolutions |
| D06 | Corps bornés : 10 000 cellules occupées par vaisseau, 128 par axe ; 32 corps dynamiques actifs en référence | Une scène nécessaire dépasse ces limites ou la mémoire mesurée | QA-07 ; modifier le contenu ou les limites avec preuve |
| D07 | Greedy simple par tranche, matériaux et visibilité explicites | Mesh chunk p95 > 8 ms ou latence édition p95 > 100 ms, après profilage | MSH-12 ; OPT-01 à OPT-04, essai binary greedy ≤ 8 h |
| D08 | Jobs de mesh asynchrones natifs ; travail web découpé et borné | Le thread principal dépasse 2 ms/image pour les tâches d'édition | MSH-11 ; réduire lot de travail ; threads web différés |
| D09 | Colliders voxels Avian pour essais corps/îles | Contact incorrect reproductible ou coût hors budget physique | PHY-06 ; diagnostic ≤ 4 h puis boîtes fusionnées, mêmes cas de contact |
| D10 | Tnua 0.32 avec adaptateur Avian3d 0.12 | Critères marche/rotation/saut non atteints après 12 h d'essai | CHR-05 ; poste de pilotage fixe et dette de marche identifiée |
| D11 | Tick fixe 60 Hz, autorité Avian et ordre explicite | Instabilité reproductible sur vitesse/charge retenue | PHY-12 ; régler sous-pas Avian dans un essai de 4 h, puis borner le scénario |
| D12 | Câble par joint Avian, distance min 0 et max L | Erreur de longueur ou énergie injectée incompatible avec le scénario | CAB-12 ; longueur/charge/vitesse bornées, pas de second solveur improvisé |
| D13 | Voile rigide simplifiée, force issue du vent relatif | Le joueur ne comprend pas ou n'utilise pas son orientation | FLT-12 ; réglage de lisibilité/coefficients avant tissu déformable |
| D14 | Réservoir Aether, consommation de sustentation, recharge au quai | La panne punit sans produire une décision compréhensible | AET-12 ; réserve de secours et retour quai ; pas de tuyaux pour la démo |
| D15 | Édition au quai avec corps immobilisé ; pas d'édition libre en vol | Les playtests démontrent que la réparation en vol est indispensable | EDT-12 ; action de réparation bornée, évolution ultérieure du contrat |
| D16 | Connexité exacte sur corps borné ; scission transactionnelle | Calcul bloquant, quantité de débris ou pose incohérente | EDT-11 ; découper le calcul et limiter débris, jamais approximation cachée |
| D17 | JSON versionné pour blueprints et sessions initiales, IDs stables | Taille/temps mesurés hors budget | SAV-12 ; compression ou codec binaire migré ultérieurement |
| D18 | Autosave natif atomique ; IndexedDB web et export manuel | Quota, stockage refusé ou écriture échouée | SAV-11 ; dernière sauvegarde valide et export ; avertissement actionnable |
| D19 | Galerie comme outil `core` + `view`, captures fixes | Fixture diverge du renderer public | UX-11 ; partager les fonctions de construction, pas copier les matériaux |
| D20 | Ancienne baseline informative, ≤ 2 h après setup | Build historique encore bloqué | ENV-08 ; archiver le blocage et poursuivre le socle |
| D21 | `bevy_cli` pour web ; petit outil Rust seulement pour besoins propres | Bundle Windows/préfixe irréparable en 4 h | WEB-03 ; wrapper ciblé autour de Cargo/wasm-bindgen |
| D22 | Aucun XPBD maison dans la boucle initiale ; extension à 10×1 | Extension flexible activée et erreur/stabilité hors critères | FLEX-04 ; essai 20×1 puis réglage ciblé, aucun réglage universel annoncé |
| D23 | Ni `big_space`, ni fluide volumétrique, ni multijoueur dans la démo | Besoin observé et accepté après PLY-12 | EXT ; une expérience bornée avant création de sous-système |
| D24 | CI sur PR ; previews après contrôles ; version annoncée par action explicite | Le déploiement ne garantit pas retour à la version précédente | WEB-12 ; conserver l'artefact validé, publication séparée |

Le joint de distance Avian documente bien des bornes minimum/maximum et des ancrages locaux. La tâche d'intégration doit vérifier sa configuration, son ordonnancement et la lecture des efforts sur la version épinglée. [DistanceJoint 0.7](https://docs.rs/avian3d/0.7.0/avian3d/dynamics/joints/struct.DistanceJoint.html).

## Frontières du socle

`aether_core` contient IDs, cellules/chunks, catalogue, commandes et schémas de fichiers. Il peut utiliser `glam` et `serde`, mais ne lance ni renderer ni accès disque. `aether_sim` adapte ces données à Bevy ECS/Avian, possède les forces et lit les intentions. `aether_view` contient meshes, matériaux, caméra, feedback, widgets et snapshots de présentation ; il dépend du domaine, pas des scénarios. `aether_game` compose les plugins, états, ports de stockage et scénarios, avec des modules dédiés et un point d'entrée court.

La galerie instancie `aether_view` sur des fixtures du domaine. Les tests physiques instancient `aether_sim` sans fenêtre. Un contrôleur peut publier ses intentions depuis `game` ; la simulation n'interroge pas directement un bouton d'interface. Les actions de l'éditeur sont des commandes validées par le domaine, consommées à une frontière de tick quand elles modifient la physique.

## Des constats du prototype aux tâches du reboot

Les défauts de structure sont observés dans les sources ; leurs coûts d'exécution restent à mesurer. Cette correspondance empêche le plan de devenir une liste de technologies détachée des problèmes rencontrés.

| Apprentissage ou défaut | Traitement vérifiable dans le plan |
| --- | --- |
| Plusieurs expériences sans traversée intégrée | FLT-10, AET-12, CUR-12 et PLY-07 : modifier une construction doit changer une navigation comprise |
| Frontière dynamique `*mut App` et cycle de vie du lobby | APP-01 à 03 et WEB-05/06 : composition statique, une session, erreurs et retour récupérables |
| Shipwright et runner concentrent des responsabilités | GOV-04, APP-12 et UX-10 : dépendances de crates vérifiées, domaine et galerie indépendants du jeu |
| Reconstructions et matériaux globaux après édition | MSH-03, MSH-05, MSH-08 à 12 : invalidation locale, cache, mesure de latence |
| Minimum recherché à répétition, `aether_voxels/src/lib.rs:557` | MSH-02 : greedy simple ; OPT-01 à 04 seulement après dépassement mesuré |
| Picking linéaire et viewport codé en pixels | EDT-01/02 : layout réel, filtrage AABB puis DDA local |
| Verre opaque et renderer distinct de la galerie | MSH-01/06, UX-10/11 : règles de visibilité et fixtures réellement partagées |
| Assets via `CARGO_MANIFEST_DIR`, `shipwright/src/lib.rs:1405` | WEB-01/02 et SAV-12 : package lancé hors checkout avec assets et sauvegardes séparés |
| Panneau du laboratoire à `bottom: px(-100)`, ligne 361 | UX-02 et EDT-01 : fenêtres, échelles et zones interactives issues du layout |
| Pas variable et caméra amortie par image | APP-07, PHY-11, CHR-09 et QA-04 : temps physique, intentions et interpolation séparés |
| `.vox` utilisé comme sauvegarde incomplète | SAV-01 à 12 : blueprints/sessions versionnés ; EXT-02/03 : échange `.vox` facultatif |
| Flexible unsafe et topologie mutable, sans preuve suffisante | FLEX-02/08/10 : API contrôlée, référence séquentielle et parallélisation uniquement mesurée |
| Captures et tests qui prouvent peu de comportement | APP-09, WEB-09, QA et PLY : invariants, parcours et observations avec limites de preuve |
| Duplications d'assets et sources d'auteur distribuées | GOV-08 et WEB-01 : provenance, manifeste runtime et empreintes ; ne bloque pas la physique |

L'édition en vol, les tissus, les tuyaux, l'eau volumique et le monde étendu sont reportés pour réduire le risque initial. Cela modifie le périmètre de la première démo, pas la vision à long terme. Une observation de joueur ou un budget dépassé peut ouvrir une extension ; sa seule présence dans l'ancien code ne suffit pas.

## Limites de ce contre audit

Le numéro de ligne du mesher, les manifestes et les compatibilités publiées ont été revérifiés. Le calcul d'espacement f32 a été exécuté localement. Aucun nouveau test de jeu ni benchmark n'a été exécuté ; le dernier état connu de compilation reste bloqué par le linker MSVC absent. Les hypothèses de budget devront être étalonnées à ENV/PHY, sans retarder la première scène jouable par une recherche exhaustive.

Le plan détaillé distingue les tâches de la première démo, les replis conditionnels et les extensions. Sa longueur répond à la demande de préparation détaillée ; seules les tâches dont les prérequis sont satisfaits doivent être engagées. Une tâche cochée doit porter une preuve, et une fonctionnalité reportée doit rester visible comme telle.
