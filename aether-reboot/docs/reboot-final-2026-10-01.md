# Aether Isles audit consolidé et plan du reboot

Document final du 1er octobre 2026, établi pour Damien sur le dépôt au commit `54e5e0a`. Il consolide le premier audit, le rapport de Claude transmis dans la conversation, les vérifications complémentaires du code et les recherches citées ci-dessous. Il remplace les recommandations du [premier audit](audit-reboot-2026-10-01.md). La contrainte reste inchangée : le jeu et ses systèmes restent en Rust.

**Mise à jour après la seconde review :** le [contre-audit et registre de décisions](reboot/CONTRE-AUDIT.md) et le [plan de 240 tâches](reboot/PLAN-DEVELOPPEMENT.md) sont désormais les références actives. Les attributions inexactes ci-dessous ont été corrigées. Les propositions historiques de trois crates et de comparaisons systématiques sont remplacées par les choix du nouveau registre ; ce document conserve les constats et recherches de cette étape.

## Direction retenue

Aether a prouvé plusieurs briques, mais pas encore leur assemblage en jeu. La valeur du reboot est de permettre au joueur de **construire une machine, comprendre son comportement, naviguer et l'améliorer**. Le risque principal est de reconstruire longtemps un moteur et des outils sans éprouver cette expérience.

Le socle recommandé pour le nouveau workspace est **Rust, Bevy 0.19 et Avian 0.7**, avec versions exactes verrouillées après validation native et web. Une application principale assemble les systèmes ; les laboratoires partagent leur code et restent disponibles en développement. Les données voxel, commandes d'édition et codecs sont indépendants du rendu. La physique rigide utilise l'ECS de Bevy via Avian ; le solveur flexible demeure une bibliothèque spécialisée.

Le premier objectif jouable est limité : **deux îles, un petit vaisseau modifiable, une voile simplifiée, un courant et un ancrage fixe**. Le joueur rejoint la seconde île, échoue de manière compréhensible, puis améliore sa construction. La sauvegarde et la récupération après erreur font partie de cette expérience.

Hypothèses de travail proposées : solo, développement prioritaire sur Windows natif, démonstration web entretenue dès le début, construction voxel complétée par quelques composants modélisés. Ce sont des choix recommandés ; seul le maintien de Rust est déjà une contrainte explicite de l'utilisateur.

## État réel et limites de l'audit

Le workspace compte 13 packages, 17 fichiers Rust et 5 902 lignes Rust. Il contient quatre expériences : caméra libre, runner, éditeur voxel et laboratoire corde/tissu. On trouve aussi une galerie de rendu, un pipeline PBR, 22 tests unitaires déclarés et deux tests Playwright. Les assets suivis représentent environ 26,7 Mio ; les fichiers suivis sous `design` environ 124,8 Mio. Ces tailles de fichiers ne mesurent ni la taille du WASM, ni le téléchargement d'un joueur, ni l'occupation attribuable dans l'historique Git.

Les contrôles du premier audit ont confirmé le formatage Rust, les métadonnées Cargo et la syntaxe des loaders JavaScript. `cargo test --workspace --locked` a échoué avant l'exécution des tests, faute de `link.exe`. Visual Studio est présent, mais aucun linker n'a été trouvé dans son installation examinée. Les builds, Clippy, les performances et le rendu interactif ne sont donc pas validés sur cette machine. Cette consolidation ajoute des lectures ciblées et des recherches ; elle ne prétend pas avoir levé ce blocage.

| Système | Ce que le code établit | Ce qu'il reste à prouver |
| --- | --- | --- |
| Construction | Édition de cellules, cinq matériaux, picking, undo/redo, import/export `.vox` natif | Édition d'un corps mobile, sauvegarde complète, fiabilité web |
| Géométrie | Stockage sparse, greedy meshing, coordonnées de surface locales, atlas | Mises à jour par chunk, transparence correcte, coûts à grande taille |
| Déplacement | Runner sur spline, saut, planeur et pendule simplifié | Navigation libre, personnage sur navire mobile, physique stable à différents FPS |
| Flexible | Nœuds, masses, contraintes XPBD, ancrages, vent simplifié, coupe et rupture | Contacts, transmission des efforts au navire, voile propulsive |
| Visuel | Matériaux PBR, UI, assets et galerie de constructions | Cohérence du rendu distribué, lisibilité en mouvement, budgets mesurés |
| Distribution | Scripts, routes web, pipeline de publication | Build portable, récupération d'erreur, tests de session et publication validée |

## Conclusions des deux audits à retenir

La vision du [BRIEF](../BRIEF.md) reste utile : vaisseau construit par le joueur, courants en trois dimensions, monde physique lisible et concentration de la simulation près du joueur. Elle sert à choisir les expériences à produire ; elle ne doit pas devenir une liste de systèmes à implémenter tous avant de jouer.

Les coordonnées de surface locales, les fixtures, le graphe partagé corde/tissu, les opérations réversibles et la provenance des assets constituent de vrais acquis. Il faut conserver leurs scénarios de référence et leurs tests avant de modifier les algorithmes.

Le code souffre surtout de responsabilités mélangées, d'absence de contrat de corps voxel, de reconstruction globale, de persistance limitée et de validation insuffisante. Le découpage apparent en crates ne suffit pas : `aether_runner` contient un jeu entier et Shipwright concentre interface, données, codec, caméra et rendu.

Le reboot doit introduire un corps voxel identifié, dans son propre repère, avec un état enregistré et des représentations dérivées pour le rendu et la collision. Il doit également faire de la construction et du déplacement deux activités sur le même vaisseau.

## Corrections et critiques du rapport de Claude

Les désaccords suivants changent les décisions du reboot. Ils portent sur les preuves et les coûts, pas sur l'intérêt général du rapport.

| Proposition ou affirmation | Arbitrage final |
| --- | --- |
| « Rien ne bouge physiquement », « pas de vent » | Incorrect au sens large : le laboratoire intègre un graphe XPBD et applique un vent simplifié. La lacune exacte est l'absence de corps voxel mobile couplé à ces efforts. |
| Archiver le runner, considéré comme éloigné du cœur du BRIEF | Accord sur l'archivage de l'expérience autonome, en conservant contrôles, animations et enseignements. Claude ne l'a pas qualifié d'inutile. La note [first public experience](../thoughts/first-public-experience/first-public-experience.thought.md) autorise une petite expérience de déplacement ; le manque est la preuve de transfert vers le jeu central. |
| « Trois semaines de polish », aucun playtest externe | L'historique donne des dates de commits, pas le temps réellement travaillé. Aucun compte rendu externe exploitable n'a été trouvé ; cela ne prouve pas qu'aucun essai n'a eu lieu. Retenir une mauvaise traçabilité, pas une mesure de productivité. |
| `Resource` unique et représentation d'un seul corps | Précision commune, pas désaccord : la ressource actuelle ne modélise pas plusieurs corps. Une collection ou des composants permettent de changer cela ; ce n'est pas une limite de Rust ou de Bevy. |
| `ActorIntent` global et intentions indépendantes | Précision commune : l'architecture présente partage la même intention. Prévoir une intention par acteur contrôlable lorsque le besoin existe. |
| Risque de comportement indéfini latent via DLL ; « compilé trois fois » | Accord sur le risque de frontière `*mut App` sans contrat ABI durable ; Claude ne prétendait pas démontrer un cas d'UB. Le facteur trois reste non mesuré : trois types de cibles Cargo ne signifient pas trois compilations intégrales indépendantes. |
| Le runner utilise Euler explicite | La mise à jour de vitesse puis position visible dans le pendule correspond plutôt à une intégration semi-implicite. Le vrai problème demeure le pas variable et le lissage caméra fixe par image. |
| Voxels, XPBD, champs et fluides en Rust pur ; corps en Bevy/Avian | Accord sur la séparation proposée : Claude prévoyait déjà une couche de corps en Bevy/Avian. Tester aussi l'intégration ECS sans fenêtre et conserver une seule autorité physique. |
| Tout placer dans `FixedUpdate` | Retenir un temps physique fixe et un ordre explicite. Avian avance par défaut dans `FixedPostUpdate` ; le couplage doit respecter ses schedules et sous-pas. La caméra et l'interface suivent le rendu. |
| Binary greedy et AO immédiatement obligatoires | Bonne piste de comparaison, pas un choix acquis. La transparence, les halos de chunks, les UV et l'AO font partie du coût réel. Un mesher plus simple peut suffire au premier navire. |
| 10 à 20 sous-pas XPBD avec une itération comme réglage universel | L'article Small Steps motive une expérience, pas un réglage garanti pour tout graphe et tout couplage. Mesurer stabilité et coût avec nos contacts, masses et ancrages. |
| `big_space` indispensable dès quelques kilomètres | Trop absolu. Le besoin dépend de l'erreur tolérée, des distances à l'origine et des repères physiques. Préparer les repères locaux ; ajouter une origine flottante quand les mesures la justifient. |
| « 100 000 voxels à 60 FPS » comme seuil de sortie initial | Inexploitable sans matériel, surfaces visibles, matériaux, corps actifs et contraintes. Remplacer par une scène de référence instrumentée et des budgets explicites. |
| Reporter les matériaux finaux ; commencer avec couleurs et AO | Accord sur le report des matériaux finaux : Claude ne proposait pas de supprimer toute lisibilité. Ajouter tôt lecture du vent, des volumes, de la tension et de la trajectoire ; l'AO reste conditionnelle à son apport. |
| Moins de 400 lignes, neuf crates dès le départ | Limites indicatives, pas critères de qualité. Commencer avec quelques frontières utiles ; ne pas créer `aether_fluid` avant d'implémenter un réseau. |
| Toute la doc périmée doit être supprimée | Archiver les observations et décisions historiques, dater les preuves et maintenir une courte feuille de route. Supprimer les doublons plutôt que perdre les apprentissages. |
| Toute publication doit être manuelle | Le besoin est une validation préalable et une version récupérable. Un déploiement automatique après contrôles réussis convient aux previews ; réserver tags ou action manuelle aux versions annoncées. |
| Tous les binaires sous LFS | Politique trop large. Choisir selon taille et fréquence de modification ; vérifier que CI et archives récupèrent les vrais assets. |
| L'IA explique les défauts d'architecture | Les signatures de commits n'établissent pas une causalité. Le remède concret est commun à tous les contributeurs : règles, changements ciblés et preuves de validation. |

La conclusion « un seul jeu, des laboratoires partagés, des responsabilités claires et des critères jouables » est retenue. L'obligation de réécrire chaque algorithme et d'adopter immédiatement toute la stack proposée est écartée.

## Registre des problèmes à traiter

Les chemins et lignes renvoient au commit audité. « Confirmé » signifie visible dans le code ; cela ne signifie pas que l'effet visuel ou le coût a été mesuré à l'exécution.

| Priorité | Constat et preuve | Action dans le reboot |
| --- | --- | --- |
| P0 | Construction et simulation de navire ne sont pas reliées | Une coque éditable reçoit effectivement forces, couples et tension |
| P0 | Environnement natif incomplet ; CI sans tests Rust ni contrôles de PR | Setup Windows reproductible et CI avant l'extraction des systèmes |
| P1 | Rebuild global et recréation des matériaux dans `games/shipwright/src/lib.rs:1175` | Chunks sales, cache de matériaux, résultats de meshing révisés |
| P1 | Verre traité opaque dans `shipwright:1292` | Règle de visibilité entre matériaux ; passes opaque et transparente distinctes |
| P1 | `CARGO_MANIFEST_DIR` utilisé pour trouver les maps à l'exécution, `shipwright:1405` | Catalogue d'assets relatif au package distribué ; test depuis un répertoire indépendant du checkout |
| P1 | Sélection répétée du minimum dans `aether_voxels/src/lib.rs:557` | Greedy simple par chunk ; alternative conditionnelle aux mesures du nouveau registre |
| P1 | Picking parcourt tous les voxels, `shipwright:1122` | Traversée DDA dans le repère du corps, après filtrage spatial des corps |
| P1 | Gravité et pendule dans `Update`, `aether_runner/src/lib.rs:623` ; `lerp(..., 0.1)` caméra, ligne 820 | Tick fixe, intentions accumulées, interpolation et amortissement dépendant du temps |
| P1 | `.vox` retire l'origine puis recentre ; format sans composants, `shipwright:763` et `812` | Format de jeu versionné, origines et IDs conservés ; `.vox` reste un format d'échange |
| P1 | Parseur avec limites et validations incomplètes, notamment `count * 4` | Codec éprouvé et validation applicative stricte ; tests de fichiers malformés |
| P1 | Import/export web non implémentés, `shipwright:899` et `926` | Adaptateurs fichier natif et téléchargement/sélection web sur un codec commun |
| P1 | `activeGame` défini avant chargement sans reprise ; `popstate` ignore une session active, `web/lobby-loader.js` | États de chargement, erreur récupérable, retour au menu et nettoyage de session |
| P1 | Topologie flexible mutable publiquement et résolution parallèle unsafe, `aether_flexible/src/lib.rs:117` et `432` | API de mutation contrôlée, révision de topologie, tests séquentiel/parallèle avant réutilisation |
| P2 | Damping par sous-pas, rupture après correction, corde bilatérale | Définir amortissement par seconde, tension, compression et rupture attendus |
| P2 | Constantes d'atlas et matériaux spécifiques dupliquées dans jeu/galerie | Renderer et définitions de matériaux communs ; mesures après découpe de l'atlas |
| P2 | Hitbox du viewport codée en pixels, `shipwright:992` ; panneau `bottom: -100`, laboratoire ligne 361 | Géométrie issue du layout réel ; validation petites fenêtres et différentes échelles UI |
| P2 | Capture sans action, `shipwright:745` ; New efface l'historique | Actions complètes, désactivées ou retirées ; politique claire de récupération d'une construction |
| P2 | Sources d'auteur copiées par `scripts/build-web.sh` | `assets/` runtime et `art-src/` auteur séparés ; manifeste d'export vérifié |
| P2 | Bibliothèques voxelize réexportées mais non utilisées ; anciennes pages et stratégies d'UV sans usage runtime trouvé | Retirer du nouveau socle les dépendances et chemins sans consommateur réel |
| P2 | Tests browser centrés sur ouverture du canvas ; erreurs effacées dans le test runner | Vérifier première image, actions d'édition, sauvegarde et transitions |
| P2 | Statuts d'initiatives obsolètes, scripts GNU et outils partiellement épinglés | README exécutable, petit journal de décisions et outils verrouillés |

Précisions utiles : le défaut PBR fait dépendre le package du chemin source embarqué à la compilation ; il échoue lorsque ce chemin n'existe pas sur la machine cible. Le viewport présente une fragilité de layout ; un bug spécifique HiDPI reste à reproduire. La caméra de la galerie utilise des vues de fixtures : on ne doit pas forcer toutes les caméras à partager le même comportement interactif.

Le doublon `assets/textures/shipwright/wooden_plank.png` et `assets/voxel_materials/wooden_plank/textures/base_color.png` est confirmé par SHA-256 identique ; chaque fichier fait 1 681 471 octets. C'est une suppression simple lors de l'export sélectif, pas un chantier prioritaire avant la navigation.

## Recherches et choix techniques

### Versions de Bevy et Avian

La publication officielle confirme Bevy 0.19, son nouveau système de scènes, la saisie de texte et des améliorations du rendu. Le gain annoncé sur `many_cubes` concerne un benchmark précis ; il ne prédit pas celui de notre mesher ou de nos corps mobiles. [Annonce Bevy 0.19](https://bevy.org/news/bevy-0-19/).

La compatibilité Bevy 0.19 / Avian 0.7 est explicitement documentée. **Choix proposé pour le reboot : ce couple**, validé dans une scène minimale avant de porter les systèmes. Cela révise la recommandation initiale de conserver 0.18 pour le nouveau socle. Le prototype historique reste une référence en 0.18.1. Ne pas programmer dès maintenant une migration 0.20 ; les fonctionnalités nécessaires et la compatibilité décideront du moment. [Table de compatibilité Avian](https://github.com/avianphysics/avian#version-table).

### Collision et personnage sur le navire

Les colliders voxels existent dans Avian. Leur disponibilité ne garantit pas toutes les paires de formes ni leur coût sous édition. Tester : coque contre île, coque contre coque, débris, capsule et sphère sur un pont, franchissement des cellules et modification de collision en mouvement. Garder comme repli des colliders composés de boîtes regroupées, sans créer un corps rigide par voxel. [API Collider](https://docs.rs/avian3d/latest/avian3d/collision/collider/struct.Collider.html).

Le ticket Rapier #1029 cité par Claude était ouvert lors de la consultation. Il rapporte une assertion du graphe de contacts dans Rapier 0.36.0 avec Parry 0.31.1. **Il n'établit pas que le solveur d'Avian possède le même bug.** Conserver son scénario de balle roulante comme cas de test, sans le transformer en motif de rejet d'Avian. [Ticket original](https://github.com/dimforge/rapier/issues/1029).

Le personnage sur un pont en translation et rotation est un risque précoce. `bevy-tnua` annonce le support des plateformes mobiles et tournantes et une intégration Avian : c'est un candidat à essayer avant d'écrire un contrôleur complet. La validation doit couvrir marche, bord, saut, vitesse héritée, atterrissage et destruction sous les pieds. [Fonctionnalités Tnua](https://github.com/idanarye/bevy-tnua#features).

### Stockage et meshing

Séparer stockage persistant et représentation temporaire de meshing. Proposition : table sparse de chunks par corps, données denses ou compactées à l'intérieur, palette de blocs stable. La taille du chunk se choisit avec le coût d'édition, les halos et les scènes réelles ; elle ne doit pas être fixée uniquement par un benchmark de bibliothèque.

`binary-greedy-meshing` est une piste sérieuse. Son README Rust décrit un tampon 64³ correspondant à **62³ cellules utiles plus les voisins**, ainsi que des masques d'opacité et de transparence. Il faut compter leur maintenance et la conversion de sortie vers les meshes Bevy. Les chiffres « 50–200 µs » et « ×30 » ne sont pas retenus comme promesses pour Aether. [README versionné de la crate](https://docs.rs/crate/binary-greedy-meshing/0.5.2/source/README.md).

Comparer trois options sur les mêmes données : mesher actuel comme référence, greedy simple par tranche, binary greedy. Mesurer aussi triangles finaux, mapping, uploads et modification d'un bloc en bordure. L'AO par sommet n'est pas un ajout gratuit : ses valeurs influencent la fusion des faces et la triangulation. [Méthode d'AO et greedy meshing](https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/).

La destruction initiale peut utiliser la connectivité des cellules. Une recherche limitée à un petit voisinage ne suffit pas toujours à prouver une séparation : un chemin peut contourner la coupure loin de celle-ci. Commencer par un calcul exact sur les corps de taille bornée ; optimiser avec un graphe de chunks ensuite. Lors d'une scission, préserver poses mondiales, masse, centre de masse, vitesses linéaires et angulaires. Ne pas copier une supposée architecture de Teardown sans preuve ni besoin.

### Temps physique et couplage flexible

Avian s'exécute à pas fixe par défaut dans `FixedPostUpdate`. Construire l'ordre des commandes, forces, contraintes, contacts et publication d'état autour de ses schedules ; ne pas appeler deux fois le moteur au motif que tous les systèmes doivent être « dans FixedUpdate ». [Documentation Avian](https://docs.rs/avian3d/latest/avian3d/).

L'étude Small Steps justifie la comparaison de nombreux petits sous-pas à peu d'itérations avec des pas plus grands davantage itérés. Elle ne fixe pas notre budget de contacts, d'intégration et de couplage. Comparer notamment 2×10, 5×4, 10×2 et 20×1, puis mesurer le temps complet et l'erreur de contrainte. Le damping devra être défini par seconde, par exemple par une décroissance exponentielle, pour conserver son sens quand le sous-pas change. [Small Steps in Physics Simulation](https://matthias-research.github.io/pages/publications/smallsteps.pdf).

Débuter le câble jouable par une contrainte de longueur maximale compatible avec les corps rigides et un rendu lisible. Porter le graphe flexible quand la courbure, la coupe ou les contacts de corde apportent une valeur réelle. Le graphe actuel reste utile comme référence. En cas de deux solveurs, documenter exactement le transfert des efforts aux ancrages ; une simple correction visuelle n'est pas un couplage physique.

Pour la voile, commencer par une surface simplifiée : vent relatif, orientation, aire, force et point d'application. Une grille physique plus grossière que les tuiles de rendu évite de faire dépendre la poussée du niveau de détail. Ajouter la déformation XPBD lorsque la navigation fonctionne déjà. La portance verticale du vaisseau doit avoir sa propre règle d'Aether ou de flottabilité magique : une voile ne doit pas servir d'explication implicite à la suspension d'une coque immobile.

### Import et sauvegarde

`dot_vox` couvre modèles, palette et graphe de scène ; il réduit l'entretien d'un parseur maison. Il ne remplace pas les règles d'import du jeu : limites, axes, matériaux inconnus, modèles multiples et dimensions. Valider un fichier asymétrique avec une hauteur identifiable dans MagicaVoxel pour trancher la conversion Y-up/Z-up ; l'audit n'a pas exécuté cet aller-retour externe. [Documentation dot_vox](https://docs.rs/dot_vox/5.2.0/dot_vox/).

Créer un format Aether distinct : version, unités, identifiants persistants, corps, cellules, orientation des composants et attaches. Différencier blueprint de construction et sauvegarde de session. Le premier peut omettre les vitesses ; la seconde doit préserver l'état nécessaire à la reprise. Écriture native atomique et récupération de la dernière sauvegarde valide ; import/export web sur les mêmes données.

### Monde et fluides

`big_space` résout un vrai problème de précision avec repères locaux et origine flottante. Il n'est pas requis pour tester deux îles proches, et ne fournit pas à lui seul le streaming ou la physique multi-repères. Préserver une frontière coordonnées monde/locales ; tester l'erreur loin de l'origine avant de choisir cette dépendance et sa version compatible. [Documentation big_space](https://docs.rs/big_space/latest/big_space/).

Les réseaux de fluide de Factorio constituent une inspiration pour l'Aether : l'article regroupe **tuyaux, tuyaux souterrains et réservoirs** en segments, avec un volume commun. L'adaptation au jeu doit définir capacité, débit, coupure, fuite, scission et conservation de quantité. Ce modèle convient à un réseau de machines ; il ne simule pas un cours d'eau, une cascade ou un courant atmosphérique. [Factorio FFF 416](https://www.factorio.com/blog/post/fff-416).

Retenir cette piste pour une extension après le premier playtest. Reporter l'automate cellulaire d'eau : il demande son propre contrat de conservation, de frontières et de couplage aux corps. Un écoulement visuel stylisé peut précéder une simulation volumétrique.

### Web et chaîne de travail

Essayer `bevy_cli` avant de recréer un serveur et un packager maison. Il fournit compilation web, bindings, serveur local, bundle et `wasm-opt` activé par défaut en release. Vérifier sous Windows les assets, le préfixe GitHub Pages et les options non interactives de CI. Un petit `xtask` Rust peut compléter les besoins propres au projet, comme la galerie ou la validation des catalogues. [Documentation Bevy CLI](https://thebevyflock.github.io/bevy_cli/cli/web.html).

Le support de WebGPU s'est étendu aux grands navigateurs, avec des différences de plateformes et de matériel. Cela ne valide pas les effets Bevy sur toutes les cibles. Choisir explicitement les navigateurs supportés, détecter les capacités et prévoir un message utile si elles manquent. WebGL2 peut rester une option si sa portée justifie le coût de maintien. [État décrit par les équipes web](https://web.dev/blog/webgpu-supported-major-browsers).

La démonstration publique doit contenir le jeu utile, sans les laboratoires de développement et sources d'auteur. Un WASM séparé par démo n'est intéressant que si plusieurs démos distinctes sont réellement publiées. Mesurer téléchargement, compilation, chargement d'assets et première image jouable. Rayon peut fonctionner en repli séquentiel sur une cible sans threads ; l'accélération native ne se transpose pas automatiquement au navigateur. [Comportement Rayon 1.13](https://docs.rs/rayon-core/1.13.0/rayon_core/).

## Architecture de départ

Commencer avec **trois crates de production**, puis extraire seulement ce qui se justifie :

| Crate | Contenu | Limite |
| --- | --- | --- |
| `aether_core` | IDs, blocs, corps/chunks, commandes, codec, connectivité, calculs de masse et échantillonnage des champs | Aucune dépendance au rendu, à la fenêtre ou au système de fichiers de la plateforme |
| `aether_flex` | Graphe et solveur flexible, API contrôlée, scénarios numériques | Pas de handles Bevy ni de connaissance de l'UI |
| `aether_game` | Plugins Bevy/Avian, contrôleur, navigation, renderer, éditeur, UI et adaptateurs de persistance | Modules par responsabilité ; orchestration et état de session explicites |

Ajouter un outil de galerie qui réutilise le renderer du jeu. Les laboratoires peuvent être sélectionnés en développement ou compilés comme exemples/outils ; ils n'ont pas à être tous inclus dans le binaire public. Les champs et réseaux restent des modules tant qu'une crate autonome ne simplifie pas réellement leur usage.

Les contrats à écrire dès le départ sont les suivants :

1. **Corps et coordonnées.** `BodyId` stable, cellules locales, pose mondiale et unités déclarées. Aucun identifiant Bevy n'est utilisé comme identité persistante.
2. **Blocs et composants.** Distinguer structure, matériau de rendu et composant fonctionnel. Une voile, un moteur ou une ancre peut occuper plusieurs cellules avec une orientation et des attaches propres.
3. **Édition.** Une commande validée produit un changement, une opération inverse et une révision. Une commande invalide laisse le corps intact. Définir le comportement de l'édition en vol et de la perte d'une attache.
4. **Représentations dérivées.** Mesh et collider proviennent d'une version des données. Un résultat asynchrone obsolète ne remplace jamais la version actuelle. Le changement physique intervient à une frontière de tick.
5. **Physique.** Un seul état fait autorité pour poses et vitesses. Les inputs sont consommés au tick ; le renderer interpole sans réécrire la simulation.
6. **Persistance.** Séparer blueprint et session ; erreurs explicites, taille bornée, migration de schéma et politique d'autosave.

Pour commencer, comparer visuellement une grille structurelle de 0,25 m et de 0,5 m sur la même coque. **0,5 m est une hypothèse de travail**, pas une contrainte du brief. Les tuiles d'eau de 10 cm mentionnées dans celui-ci décrivent le rendu de l'eau, pas une obligation d'utiliser cette échelle pour tous les solides. La résolution de rendu, le stockage structurel et la collision peuvent différer.

## Feuille de route jouable

L'ordre ci-dessous remonte les risques de physique avant la réalisation d'un éditeur complet. Il n'impose pas de durée calendaire inventée ; chaque étape sort avec une preuve courte, des mesures et une décision.

| Étape | Livrable | Critère de sortie |
| --- | --- | --- |
| 0 Reproductibilité | Environnement Windows, versions épinglées, CI de PR, lancement natif et web, baseline historique | Un checkout documenté compile ; les tests historiques ont un résultat connu ; une scène minimale 0.19/0.7 s'affiche sur les cibles choisies |
| 1 Risques physiques | Petite coque, île, contacts, déplacement et rotation, personnage embarqué, câble simplifié | Le personnage marche et saute d'un pont mobile ; la coque réagit à une force et à un ancrage sans instabilité majeure |
| 2 Construction persistante | Corps/chunks, édition locale, matériaux simples, undo/redo, save/load, petite scission | Une création transformée se recharge exactement ; retirer un support sépare un fragment ; mesh et collision correspondent |
| 3 Traversée | Portance magique explicite, masse/inertie, voile simplifiée, commandes et deux îles | Modifier la masse ou la voile modifie la trajectoire de façon compréhensible ; un joueur atteint l'autre île |
| 4 Signature Aether | Courant local, embranchement simple, harpon, libération et réparation | Attacher puis libérer le câble aide à choisir la trajectoire ; l'échec se comprend et se récupère rapidement |
| 5 Test public court | Première expérience sauvegardable, export web, tutoriel minimal, son et visuels utiles | Cinq sessions d'observation documentées, puis une correction et un nouveau test ciblé |
| 6 Extensions justifiées | Tissu déformable, Aether en tuyaux, corps plus grands ou monde plus vaste | Chaque extension répond à une difficulté ou une envie observée dans les étapes précédentes |

La phase 1 peut utiliser une géométrie rudimentaire et des commandes de debug. Elle doit prouver la viabilité des choix physiques avant de bâtir tous les outils autour. Une scission minimale à l'étape 2 protège l'architecture ; une destruction générale avec des milliers de débris reste hors périmètre initial.

Le réseau d'Aether n'est pas une condition d'entrée au premier playtest. Son ajout avant toute observation joueur repousserait précisément l'apprentissage que les deux audits jugent prioritaire.

## Mesures et critères de qualité

Définir une machine de référence avec CPU, GPU, RAM, OS, résolution, profil de compilation et, pour le web, navigateur et état du cache. Mesurer des percentiles, pas seulement une moyenne de FPS. Les valeurs suivantes sont des **objectifs provisoires**, jamais des résultats actuels : 60 FPS en 1080p sur la machine choisie, environ 4 ms de physique par tick et réponse d'édition sous 100 ms au percentile 95.

Les scènes couvrent : coque compacte, construction creuse, damier fragmenté, verre contre opaque, deux vrais chunks voisins, coque contre coque, plateforme tournante, câble tendu puis coupé et chargement d'une sauvegarde invalide. Les tailles 1 000, 10 000 et 50 000 voxels servent à explorer la courbe de coût. Le nombre de corps actifs, contacts, contraintes, faces visibles et éditions par seconde accompagne chaque mesure.

Les tests nécessaires portent sur les invariants : save/load, commandes inverses, frontières de chunks, visibilité des faces, origine locale, connectivité, masse/inertie, conservation lors de la scission et rejection des résultats périmés. Tester la même séquence d'intentions avec un rendu à 30/60/120 FPS et des frames irrégulières ; une stabilité locale ne prouve pas un déterminisme multiplateforme.

Pour l'intégration Bevy, utiliser une application sans fenêtre avec le temps piloté et les plugins requis. L'exemple officiel montre comment tester une application ; un nouveau framework de test n'est pas un prérequis. Les tests headless complètent, sans remplacer, les essais GPU et navigateur. [Tests d'applications Bevy 0.19](https://github.com/bevyengine/bevy/blob/v0.19.0/tests/how_to_test_apps.rs).

La galerie utilise les mêmes matériaux et le même chemin de rendu que le jeu. Elle capture après chargement des assets et préparation du rendu, avec un délai maximal et une erreur explicite en cas d'échec. Les comparaisons visuelles gardent des tolérances adaptées aux GPU. Le package natif est aussi lancé depuis un emplacement sans checkout pour détecter les chemins de compilation embarqués.

Les cinq playtests sont une première observation qualitative, pas une validation statistique. Noter le temps avant une première modification, la compréhension du vent et du câble, les causes d'échec expliquées par le joueur, les demandes d'aide et l'envie de réessayer. Si les joueurs ne relient pas leur construction à son comportement, améliorer cette relation avant d'ajouter du contenu.

## Réutilisation et conduite du travail

| Acquis | Traitement |
| --- | --- |
| Brief et concept art | Conserver la vision et les références ; relier chaque version à un périmètre court |
| Mesher, coordonnées et fixtures | Garder comme référence ; extraire les contrats et remplacer seulement sur preuves |
| Solveur flexible | Porter avec API encapsulée, damping explicite et tests de grandes tailles |
| Runner | Archiver l'expérience autonome ; garder intentions, animations et enseignements de contrôle |
| Shipwright | Recomposer données, commandes, picking et UI ; conserver les scénarios d'interaction |
| Galerie et textures | Réutiliser avec renderer commun et assets runtime propres |
| Parseur VOX | Remplacer par un adaptateur contrôlé autour de `dot_vox` si l'échange reste utile |
| Chargement DLL et lobby multi-jeux | Retirer du chemin produit ; laboratoires sélectionnés en développement |
| Scripts et documentation | Réduire aux commandes réellement utilisées et aux preuves vérifiables |

Conserver le prototype historique sur un état Git identifiable. Construire le nouveau socle dans une branche ou un workspace isolé, en portant une responsabilité à la fois. Ne pas faire un grand commit qui mélange migration Bevy, nouveau stockage, nouvelle physique et nouveau rendu.

Créer un court `AGENTS.md` lors du lancement du reboot : dépendances autorisées, conventions de coordonnées, autorité physique, commandes de validation et règle de préservation des fichiers existants. Les mêmes règles s'appliquent aux humains et aux agents. Un changement est terminé lorsque son comportement est observable, ses tests pertinents passent et sa documentation d'usage correspond au code.

La CI valide les PR avec formatage, Clippy, tests et compilation native/web verrouillée. Les tests visuels GPU peuvent être exécutés sur un runner dédié ou un poste identifié ; ne pas confondre un simple build avec leur réussite. Épingler les outils de CI ; `cargo nextest`, hot-patching, BSN et Conventional Commits sont des options de travail, pas des conditions nécessaires au premier navire.

La documentation active tient dans une vision, une feuille de route, quelques décisions courtes et des résultats de tests. Les anciennes initiatives restent des archives datées. Avant toute nouvelle infrastructure, poser la question opérationnelle : **quel comportement du joueur ou quel risque mesurable cette tâche permet-elle de valider ?**

## Décisions à confirmer au démarrage

Les choix à confirmer sont peu nombreux : priorité Windows/web, solo à court terme, taille des blocs après comparaison, limites initiales des constructions et place du personnage embarqué. La proposition de référence est un jeu solo Windows avec démo web, une grille de 0,5 m à éprouver, un petit archipel réalisé à la main et un personnage capable de marcher sur le navire.

Le succès du reboot ne se mesurera pas au nombre de crates, de matériaux ou de systèmes annoncés. Il se constatera lorsqu'une modification de construction produit un effet que le joueur comprend, utilise pour naviguer et veut améliorer.
