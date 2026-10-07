# Plan détaillé du développement Aether Isles

Préparation du 1er octobre 2026. Décisions de référence : [contre audit et registre des choix](CONTRE-AUDIT.md). Toutes les tâches ci-dessous sont **à faire** : leur présence ne signifie pas que le développement est commencé ni que les hypothèses ont été validées.

## Comment exécuter ce plan

Le plan contient 20 lots de 12 tâches : **240 tâches**, dont 204 dans les lots de la première démo et 36 dans les lots conditionnels ou d'extension. Certaines lignes de la première démo décrivent un repli : elles ne sont exécutées que si leur condition est rencontrée. Une tâche devenue inutile est marquée « non applicable » avec motif, jamais « réussie ».

Chaque ligne donne un travail et son livrable, une preuve d'acceptation, des dépendances immédiates et une charge initiale en heures de travail effectif. La charge inclut implémentation et vérification locale ; elle n'est ni un délai calendaire ni une promesse. Réestimer après le socle et le premier risque physique. Temps d'attente de téléchargement, recrutement et réponses externes à suivre séparément. Les plafonds d'investigation du contre-audit priment sur les estimations d'une exploration.

Un seul responsable par tâche lorsqu'elle est démarrée ; Damien décide des changements de produit, la personne ou l'agent exécutant apporte les preuves techniques. Garder au plus un lot principal et une tâche indépendante en cours pour une personne. Les tâches ne sont pas des obligations de créer 240 PR : regrouper 2 à 5 tâches cohérentes quand cela reste facile à relire et annuler.

Statuts à renseigner lors du développement : à faire, en cours, bloqué avec cause, fait avec preuve, essai non concluant, reporté ou non applicable. Une preuve est un résultat de test, un rapport de mesure, un artefact ou une observation manuelle précise associée au commit. Les tests sont ajoutés pour protéger un comportement significatif, pas pour recopier chaque fonction.

Les chemins indiqués sont ceux du **nouveau workspace proposé**, à créer dans une branche/zone isolée au démarrage. Aucune tâche n'autorise à supprimer le prototype historique. Le découpage des modules peut évoluer sans changer les responsabilités.

## Jalons et règles de sortie

| Jalon | Lots | Résultat observable | Décision en cas d'échec |
| --- | --- | --- | --- |
| J0 Socle | GOV, ENV, APP | Scène native et web, règles et validation reproductibles | Dépendances : diagnostic 4 h ; legacy : 2 h maximum ; bloquer seulement sur un prérequis du nouveau socle |
| J1 Risques physiques | PHY, CHR, essai CAB-01 | Corps, contacts, câble simple, poste de pilotage et marche embarquée évalués | Collision voxel : diagnostic 4 h puis boîtes ; Tnua : 12 h puis poste fixe ; dette déclarée |
| J2 Construction durable | DAT, MSH, EDT, SAV | Construction au quai, scission bornée, sauvegarde et reprise | Meshing : profilage puis essai OPT ≤ 8 h ; limite de construction explicitée |
| J3 Première traversée | FLT, AET | Voile et sustentation Aether relient deux îles | Réglage force/lecture ≤ 8 h ; réduire vent et masse, sans masquer la panne |
| J4 Manœuvre signature | CAB, CUR | Ancrage et courant permettent de choisir une trajectoire | Câble : diagnostic ≤ 8 h ; réduire charge/vitesse/longueur du scénario |
| J5 Démo testable | UX, WEB, QA, PLY | Jeu court stable, exportable, observé par cinq personnes | Corriger les blocages d'abord ; choisir une seule hypothèse de fun par itération |
| J6 Extensions | FLEX, EXT | Une amélioration justifiée par une observation | Une expérience à la fois ; ne bloque pas la sortie J5 |

Les dates seront calculées à partir de la capacité réelle de l'équipe et des charges réestimées. Un point de revue a lieu à chaque fin de lot et toutes les 12 heures d'investigation d'un risque. Après deux tentatives ciblées sans progrès, appliquer le repli documenté ou faire arbitrer le périmètre ; ne pas ouvrir trois nouvelles technologies.

L'ordre des chapitres regroupe les responsabilités, il n'impose pas de finir un lot entier avant toute tâche du suivant. CAB-01 se fait à J1 sur fixture. DAT et le début de FLT/AET peuvent avancer dès leurs prérequis disponibles ; la persistance complète n'est pas requise pour tester voile et réserve. J2/J3 désignent les sorties intégrées. Dès qu'une coque de fixture se pilote, faire un essai de navigation court avant de poursuivre les finitions de l'éditeur. Le lot OPT est transversal : seule l'optimisation d'un défaut bloquant devient nécessaire pour le jalon concerné ; ses autres tâches restent différées.

## Charge prévisionnelle et lecture rapide des lots

La somme des estimations des 204 tâches de la démo est **592 h**, avant réserve et avant retrait des replis devenus non applicables. C'est une estimation de planification issue du découpage, sans vitesse d'équipe mesurée. Pour dimensionner le projet, prévoir provisoirement **environ 770 à 890 h avec 30 à 50 % de réserve** ; recalculer après J0/J1. Cette fourchette ne garantit pas la livraison et ne doit pas empêcher de réduire le périmètre à un jalon.

À titre de conversion, 20 h effectives par semaine correspondent à environ 39–45 semaines pour cette enveloppe complète. Le premier déplacement pilotable arrive à J1, bien avant la démo distribuable ; le nombre de tâches ne doit jamais devenir une raison de repousser les essais de jeu. Réduire cette durée exige d'abord de retirer du périmètre, pas de diviser arbitrairement toutes les charges.

Les 36 tâches conditionnelles totalisent **140 h d'essais et d'études** si elles étaient toutes activées, ce qui n'est pas recommandé. Cette somme n'est pas à ajouter automatiquement au projet et ne chiffre pas une version de production des extensions.

| Lot | Sujet | Tâches | Charge nominale | Sortie |
| --- | --- | --- | --- | --- |
| GOV | Cadrage et règles | 12 | 15 h | J0 |
| ENV | Environnement et CI | 12 | 22 h | J0 |
| APP | Application et mesures | 12 | 31 h | J0 |
| PHY | Corps et contacts | 12 | 38 h | J1, dont repli conditionnel |
| CHR | Personnage et poste | 12 | 31 h | J1 |
| DAT | Données et contrats | 12 | 36 h | J2 |
| MSH | Géométrie et rendu | 12 | 40 h | J2 |
| EDT | Construction et scission | 12 | 43 h | J2 |
| SAV | Sauvegarde et reprise | 12 | 41 h | J2 |
| FLT | Navigation et voile | 12 | 37 h | J3 |
| AET | Réserve et sustentation | 12 | 35 h | J3 |
| CAB | Harpon et câble | 12 | 39 h | Essai J1, intégration J4 |
| CUR | Courant et archipel | 12 | 39 h | J4 |
| UX | Lisibilité et retours | 12 | 38 h | J5 |
| WEB | Packaging et distribution | 12 | 37 h | J5 |
| QA | Robustesse et budgets | 12 | 38 h | J5 |
| PLY | Observations et corrections | 12 | 32 h | J5 |
| FLEX | Flexibles avancés | 12 | 48 h | Après J5, conditionnel |
| OPT | Optimisations mesurées | 12 | 38 h | À la demande d'un budget dépassé |
| EXT | Études d'extensions | 12 | 54 h | Après J5, conditionnel |

Le graphe de prérequis donne l'ordre technique ; les priorités indiquent l'importance, pas une invitation à lancer tous les P0 simultanément. Les tâches de production dépendent de comportements validés. Une tâche de **décision après essai**, comme CHR-05, peut au contraire consommer un essai non concluant documenté et choisir son repli. L'expiration d'un budget n'autorise pas les autres tâches à ignorer un prérequis nécessaire.

## GOV Cadrage et traçabilité

Priorité P0, jalon J0. Livrables : `docs/`, règles de contribution et fixtures de référence. Début possible avant la compilation. L'objectif est de rendre les choix exécutables, sans développer un processus documentaire autonome.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| GOV-01 | Enregistrer commit historique, état Git, documents de référence et périmètre du reboot dans une fiche de démarrage. | Le point de départ est identifiable et les fichiers locaux existants sont inventoriés. | — | 1 |
| GOV-02 | Écrire la promesse de la démo : construire au quai, traverser, s'ancrer, réparer et recommencer ; définir ses exclusions. | Une page décrit début, objectif, échec, reprise et durée visée de 10–15 minutes. | GOV-01 | 1 |
| GOV-03 | Adopter le registre D01–D24 comme défaut proposé et consigner les décisions produit de Damien lorsqu'elles arrivent. | Chaque réserve utilisateur est reliée à un choix ; aucune hypothèse n'est présentée comme validée. | GOV-02 | 1 |
| GOV-04 | Décrire les quatre crates et les imports autorisés ; placer les contrats partagés dans le domaine. | Graphe sans cycle ; galerie indépendante de `game` ; simulation testable sans fenêtre. | GOV-03 | 2 |
| GOV-05 | Préparer `AGENTS.md` avec question de valeur, architecture, limites, commandes et définition de terminé. | Un contributeur trouve comment valider sa modification ; règles courtes et propres au projet. | GOV-04 | 1 |
| GOV-06 | Vérifier les outils d'agents utilisés ; ajouter si nécessaire `CLAUDE.md` avec import `@AGENTS.md`. | Les instructions chargées sont vérifiées sur les versions installées ; contenu maintenu une seule fois. | GOV-05 | 1 |
| GOV-07 | Créer un modèle de décision d'une page : problème, défaut, déclencheur, essai limité et résultat. | Une décision exemple contient un seuil et un repli sans comparaison obligatoire. | GOV-03 | 1 |
| GOV-08 | Inventorier actifs réutilisables, provenance, licences et consommateurs : Knight, textures, fixtures, solveur. | Chaque actif repris a une source ; dépendances non consommées ne migrent pas par défaut. | GOV-01 | 2 |
| GOV-09 | Définir les scénarios canoniques et graines : pont, coque, deux chunks, câble et archipel. | Même entrée produit les mêmes données de scène indépendamment de l'ordre des HashMap. | GOV-02 | 2 |
| GOV-10 | Fixer format des preuves : commit, cible, matériel, commande, résultat et limites. | Un exemple de mesure est reproductible et distingue CPU, GPU, tick et frame. | GOV-09 | 1 |
| GOV-11 | Mettre les anciennes initiatives en référence historique et définir une seule feuille de route active. | Les liens entrants indiquent clairement le document actif ; aucune preuve ancienne n'est supprimée. | GOV-01, GOV-07 | 1 |
| GOV-12 | Relire périmètre, décisions, risques et ordre des lots ; ouvrir seulement les prochaines tâches exécutables. | Promesse et décisions cohérentes, questions restantes explicites, liste courte de travaux prêts. | GOV-03, GOV-04, GOV-05, GOV-08, GOV-09, GOV-10, GOV-11 | 1 |

## ENV Environnement et chaîne de validation

Priorité P0, J0. Livrables : toolchain, workspace, CI et commandes Windows. L'ancien projet n'est pas un préalable au succès du nouveau. Les installations seront effectuées au démarrage du développement dans le cadre autorisé et des permissions disponibles.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| ENV-01 | Relever Windows, CPU/GPU/RAM, pilotes, Rust, Node et outils disponibles ; désigner la machine de référence. | Fiche matérielle et versions enregistrées, résolution 1080p et navigateur de test choisis. | GOV-01 | 1 |
| ENV-02 | Installer/configurer MSVC C++ et SDK Windows manquants ; vérifier le linker dans un shell reproductible. | Un petit exécutable Rust compile et se lance ; chemin et installation documentés. | ENV-01 | 2 |
| ENV-03 | Créer la zone de reboot isolée, le workspace et les quatre crates sans déplacer le prototype. | `cargo metadata` expose les packages attendus ; historique et assets originaux accessibles. | GOV-04, ENV-02 | 2 |
| ENV-04 | Épingler Rust compatible, Bevy 0.19 et Avian 0.7 ; générer le lockfile et déclarer les cibles. | Compilation minimale réussie avec `--locked` ; rustfmt, Clippy et cible WASM disponibles. | ENV-03 | 2 |
| ENV-05 | Aligner `glam` 0.32, Bevy Math et futures intégrations ; inspecter les versions dupliquées pertinentes. | Un vecteur du domaine est utilisé sans conversion de version ; arbre Cargo archivé. | ENV-04 | 1 |
| ENV-06 | Définir features natives/web et profils dev, release ; activer seulement les besoins réels. | Build native sans features web inutiles ; build WASM sans backend natif imposé. | ENV-04 | 2 |
| ENV-07 | Installer une version verrouillée du CLI web et des outils nécessaires ; écrire les commandes Windows. | Un utilisateur peut retrouver installation, build, lancement et nettoyage sans Bash GNU. | ENV-04 | 2 |
| ENV-08 | Tenter baseline historique, tests et captures pendant 2 h au maximum ; archiver succès ou échec. | Log daté et limites connues ; l'échec ne bloque pas APP ni PHY. | ENV-02, GOV-08 | 2 |
| ENV-09 | Créer CI PR : fmt, Clippy, tests headless et vérification native Windows avec lockfile. | Un échec significatif bloque le job ; aucun token de déploiement nécessaire à la validation. | ENV-06 | 3 |
| ENV-10 | Ajouter build WASM à la CI et archive de logs ; contrôler la correspondance wasm-bindgen/lockfile. | Le job compile avec outils épinglés et échoue avec diagnostic exploitable si incompatible. | ENV-07, ENV-09 | 2 |
| ENV-11 | Documenter un parcours depuis checkout neuf ; faire une répétition dans un répertoire propre. | Aucune dépendance cachée au répertoire initial ; commandes réellement exécutées et durée relevée. | ENV-09, ENV-10 | 2 |
| ENV-12 | Valider D01, D04 et environnement ; noter outils retenus et installer un suivi des temps de build. | Nouveau socle compilable ; problèmes legacy séparés ; preuve des deux cibles. | ENV-05, ENV-06, ENV-07, ENV-09, ENV-10, ENV-11 | 1 |

## APP Application et instrumentation

Priorité P0, J0. Modules : `game/app`, `game/session`, `view/debug`. Ce lot établit le cycle de vie qui manquait au lobby historique.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| APP-01 | Composer les plugins depuis un point d'entrée court ; créer une scène vide et un outil galerie minimal. | `view` ne dépend pas de `game` ; mêmes fonctions de rendu accessibles à la galerie. | ENV-12, GOV-04 | 2 |
| APP-02 | Implémenter états Boot, Menu, Loading, Playing, Paused, Error et sortie ; définir les transitions légales. | Une transition illégale ne lance pas une deuxième session ; test de cycle menu-jeu-menu. | APP-01 | 3 |
| APP-03 | Donner un propriétaire de session aux entités/ressources temporaires et une procédure de nettoyage. | Dix lancements/retours ne multiplient pas caméras, lumières ou systèmes de session. | APP-02 | 3 |
| APP-04 | Initialiser fenêtre, caméra, éclairage sobre et carte de sol ; inclure commandes de debug. | Une image identique dans les grandes lignes est obtenue en natif et galerie. | APP-01 | 2 |
| APP-05 | Ajouter chargement d'assets observé, erreur visible et tentative de reprise ; finir Loading après disponibilité réelle. | Asset absent produit un écran récupérable, pas une barre figée ou un canvas vide. | APP-02, APP-04 | 3 |
| APP-06 | Démarrer la scène minimale WebGPU sur Chrome/Edge cible ; tester absence d'adapter GPU. | Première image et input reçus ; erreur de capacité lisible ; décision D02 enregistrée. | ENV-07, APP-05 | 3 |
| APP-07 | Définir collecte d'input, consommation au tick et interpolation du rendu dans des SystemSets ordonnés. | Un schéma de schedule correspond au code ; aucune double avance de physique. | APP-02 | 3 |
| APP-08 | Ajouter overlay dev : frame/tick, corps, contacts, jobs et état ; journaux structurés désactivables. | Un capture de diagnostic identifie la scène et les compteurs sans polluer l'UI publique. | APP-04, APP-07 | 2 |
| APP-09 | Créer harnais headless avec temps piloté et captures d'état finies ; enregistrer les graines. | Tests rapides sans fenêtre ni GPU, nombre exact de ticks pilotable. | APP-07, GOV-09 | 3 |
| APP-10 | Définir comportement de pause, perte de focus et reprise d'onglet ; borner le rattrapage des ticks. | Après absence prolongée, pas de saut catastrophique ni de séquence d'actions répétée. | APP-07, APP-09 | 3 |
| APP-11 | Ajouter profils de test reproductibles et export de mesures ; isoler échauffement et séquence mesurée. | Même scénario fournit CSV/JSON de résultats avec métadonnées et unités explicites. | APP-08, APP-09 | 2 |
| APP-12 | Faire revue de socle : cycle de session, erreur d'asset, première image web et dépendances des crates. | J0 documenté ; failures non masquées ; scène prête pour intégration physique. | APP-03, APP-05, APP-06, APP-09, APP-10, APP-11 | 2 |

## PHY Corps rigides et contacts

Priorité P0, J1. Modules : `sim/bodies`, `sim/collision`, `sim/schedule`. Commencer avec géométrie de fixture ; les chunks et l'éditeur final ne sont pas requis.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| PHY-01 | Installer Avian dans le schedule choisi ; déclarer axes, unités m/kg/s, gravité et autorité des poses. | Corps simple tombe et repose ; un test identifie l'ordre tick-forces-contacts-publication. | APP-12 | 3 |
| PHY-02 | Créer île statique, petite coque dynamique et points d'ancrage locaux à partir de fixtures. | Deux corps indépendants se déplacent sans mélanger leurs coordonnées ; poses inspectables. | PHY-01 | 2 |
| PHY-03 | Produire collider voxel de la fixture et afficher collider/mesh superposés pour diagnostic. | Alignement centres de cellules et échelle 0,5 m vérifié sur corps tourné. | PHY-02 | 3 |
| PHY-04 | Tester coque/île, coque/coque, sphère/pont et franchissement d'arêtes internes en debug/release. | Rapport de contacts : absence de pénétration persistante, de saut injustifié et de panic observés. | PHY-03, APP-09 | 4 |
| PHY-05 | Si PHY-04 échoue, isoler le cas ≤ 4 h puis implémenter repli en boîtes fusionnées. | Même batterie passée avec le repli, défaut voxel enregistré ; sinon tâche non applicable. Les 8 h estimées incluent diagnostic et intégration du repli. | PHY-04 | 8 |
| PHY-06 | Choisir et verrouiller le chemin collision initial ; enregistrer le coût par type de paire. | D09 résolu avec preuve et limites de tailles/vitesses ; aucune promesse implicite de toutes formes. | PHY-04, PHY-05 | 1 |
| PHY-07 | Calculer masse, centre et inertie des fixtures ; éviter qu'Avian les recalcule différemment sans contrôle. | Bloc symétrique et masse décentrée correspondent à leurs valeurs analytiques. | PHY-02 | 4 |
| PHY-08 | Appliquer force et couple à un point local transformé ; exposer commandes d'essai bornées. | Force centrale translate ; force excentrée tourne ; signe et unités testés. | PHY-07 | 3 |
| PHY-09 | Tester sommeil/réveil, contact après téléportation de debug et remise au quai. | Aucune vitesse résiduelle non voulue ; réveil lors d'une commande ou nouvelle force. | PHY-06, PHY-08 | 2 |
| PHY-10 | Valider vitesse maximale initiale, petite épaisseur, chute et activation de CCD si nécessaire. | Une scène à vitesse maximale ne traverse pas la paroi ; coût de CCD enregistré. | PHY-06 | 3 |
| PHY-11 | Jouer mêmes intentions à 30/60/120 FPS et avec frames irrégulières, à ticks physiques constants. | Écart de poses sous tolérance proposée 1 cm/0,5° dans la fixture ; aucun input doublé/perdu. | PHY-08, APP-10 | 3 |
| PHY-12 | Enregistrer paramètres physiques initiaux et revue de stabilité ; fermer les alternatives non nécessaires. | Dix minutes simulées finies et contacts de référence acceptés ; D11 documenté. | PHY-06, PHY-07, PHY-09, PHY-10, PHY-11 | 2 |

## CHR Personnage et poste de pilotage

Priorité P0, J1. Modules : `game/control`, `sim/character`, `view/camera`. L'essai CHR-01 à CHR-04 est plafonné à 12 h au total, intégration comprise ; aucun contrôleur maison complet n'est lancé comme repli. Si le plafond est atteint, consigner les vérifications restantes comme non validées et passer à CHR-05, sans les déclarer réussies.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| CHR-01 | Intégrer `bevy-tnua` 0.32, adaptateur Avian3d 0.12 et couche commune ; capsule de test. | Résolution unique cohérente et marche sur sol statique, en natif et compilation WASM. | PHY-12, ENV-05 | 2 |
| CHR-02 | Éprouver support en translation, rotation et inclinaison avec Tnua pendant ≤ 4 h. | Immobilité relative au pont : dérive proposée < 5 cm/30 s sur translations 5 m/s et rotations 30°/s. | CHR-01 | 4 |
| CHR-03 | Éprouver marche, bord, saut et atterrissage sur le même pont pendant ≤ 4 h. | Vitesse héritée au saut et retour au pont cohérents ; aucun verrouillage ni lancement artificiel. | CHR-02 | 4 |
| CHR-04 | Tester interaction masse personnage/navire, escalier, plafond et variation des FPS dans le budget restant. | Effort du contrôleur ne retourne pas une coque raisonnable ; limites de pente documentées ou vérification déclarée non validée. | CHR-03 | 2 |
| CHR-05 | Décider marche embarquée ou repli après plafond ; fixer le périmètre de la démo. | D10 contient traces et résultat ; un échec ou essai incomplet n'est jamais marqué marche validée. | CHR-01, CHR-02, CHR-03, CHR-04 | 1 |
| CHR-06 | Créer état pilotage : attache locale au poste, neutralisation du contrôleur, commandes de navire. | Une seule autorité de mouvement ; rotation du navire suivie sans corps dynamique parenté deux fois. | CHR-05, APP-02 | 3 |
| CHR-07 | Définir entrée/sortie du poste et, si repli, autoriser débarquement seulement au quai immobile. | Pas de sortie dans le vide ; position et vitesses rétablies explicitement. | CHR-06 | 3 |
| CHR-08 | Ajouter intention par acteur contrôlable, focus UI et raccourcis configurables de base. | Saisir du texte ne déplace pas le joueur ; clavier maintenu après focus perdu ne reste pas actif. | CHR-06, APP-07 | 2 |
| CHR-09 | Créer caméra de suivi amortie en secondes, orbit de construction et transition de mode. | Même délai de suivi à 30/120 FPS ; aucune caméra ne réécrit la pose physique. | CHR-06, APP-04 | 3 |
| CHR-10 | Réutiliser Knight et animations utiles ; prévoir capsule visible de remplacement en cas d'asset absent. | Idle/marche/chute changent avec l'état ; modèle manquant reste jouable. | CHR-08, GOV-08 | 3 |
| CHR-11 | Créer récupération joueur hors limites et après perte de support ; dernier quai valide. | Chute ou déconnexion du pont conduit à une reprise explicite, sans sauver un état impossible. | CHR-07, APP-10 | 2 |
| CHR-12 | Rejouer scène embarquée ou poste fixe dans les deux builds ; publier limitations connues. | J1 permet de piloter une coque ; voie retenue validée, état de marche libre affiché dans la preuve, pas supposé. | CHR-05, CHR-07, CHR-08, CHR-09, CHR-10, CHR-11 | 2 |

## DAT Domaine voxel et contrats

Priorité P0, J2. Modules : `core/ids`, `core/grid`, `core/catalog`, `core/body`. Le stockage commence simple et borné ; éviter compression, octree et génération infinie sans mesure.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| DAT-01 | Définir IDs stables de corps, blocs, matériaux et composants ; séparer IDs du domaine et Entity Bevy. | Export d'un corps ne contient aucune adresse ni Entity transitoire ; références résolues explicitement. | GOV-04, ENV-05 | 2 |
| DAT-02 | Définir conventions cellule-centre, axes, rotations autorisées, mètres et conversions monde/local. | Un aller-retour sur corps translaté/tourné retrouve la même cellule, y compris coordonnées négatives. | DAT-01 | 3 |
| DAT-03 | Créer ChunkCoord et CellCoord avec division euclidienne et indexation 16³. | Cas -17, -16, -1, 0, 15, 16, 17 correctement répartis ; accès hors bornes refusé. | DAT-02 | 3 |
| DAT-04 | Implémenter chunks denses alloués à la demande dans une table sparse ; itération stable d'export. | Chunk vide libérable ; deux ordres d'insertion donnent mêmes données sérialisées. | DAT-03 | 3 |
| DAT-05 | Définir catalogue minimal bois/métal/verre avec masse, visibilité, collision et matériau distincts. | Un ID inconnu échoue proprement ; rendu et physique ne partagent pas un enum privé d'éditeur. | DAT-01 | 3 |
| DAT-06 | Définir composants fonctionnels : sustentateur, réservoir, voile, poste et harpon ; empreintes et attaches. | Placement orienté multi-cellules détecte chevauchement ; attaches identifiées hors renderer. | DAT-02, DAT-05 | 4 |
| DAT-07 | Introduire BodyDefinition, pose de session et révision ; mettre limites 10 000 cellules/128 par axe. | Dépassements rejetés avant allocation importante ; définition et état dynamique distincts. | DAT-04, DAT-06 | 3 |
| DAT-08 | Définir commandes Add/Remove/PlaceComponent et résultat transactionnel avec cellules/chunks affectés. | Commande invalide ne modifie ni données ni révision ; transaction fournit inverse et changements. | DAT-07 | 4 |
| DAT-09 | Adapter les fixtures PHY au domaine ; construire une table BodyId vers entités de simulation. | Deux corps identiques ont IDs différents et aucune référence croisée accidentelle. | DAT-07, PHY-02 | 3 |
| DAT-10 | Extraire calculs masse, centre et tenseur d'inertie des voxels et composants, avec unités. | Résultats testés sur cube plein, coque creuse et composant décentré ; masse supprimée explicitée. | DAT-05, DAT-06, PHY-07 | 4 |
| DAT-11 | Définir snapshots immuables pour rendu et calculs asynchrones ; révision corps/chunk dans chaque résultat. | Le consumer peut refuser un résultat ancien sans lire un pointeur mutable du domaine. | DAT-07, DAT-08 | 2 |
| DAT-12 | Revue des invariants et tests sans Bevy renderer ; figer contrats initiaux et échelle D05. | Grille, IDs, limites, commandes et masses vérifiés ; aucune extraction de crate supplémentaire requise. | DAT-03, DAT-04, DAT-05, DAT-06, DAT-08, DAT-09, DAT-10, DAT-11 | 2 |

## MSH Géométrie et rendu voxel

Priorité P1, J2. Modules : `core/meshing` pour résultats neutres et `view/voxels` pour Bevy. Les nombres de faces après mapping et les temps complets sont observés dès le début.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| MSH-01 | Définir prédicat de face visible selon matériau source/voisin, avec règles air/opaque/verre. | Tests des paires et même verre ; aucune disparition d'une paroi opaque vue derrière une vitre. | DAT-05, DAT-12 | 3 |
| MSH-02 | Implémenter greedy simple par masque de tranche, ordre déterministe et partition par matériau. | Cube, paroi, damier et trou produisent faces attendues, normales et winding corrects. | MSH-01 | 4 |
| MSH-03 | Fournir halo voisin de chaque chunk et invalider les faces voisines après édition sur frontière. | Deux chunks réellement distincts ne produisent ni trou ni face intérieure à leur jonction. | MSH-02, DAT-08 | 3 |
| MSH-04 | Transporter coordonnées de surface locales continues et orientation des faces indépendantes des UV finaux. | Déplacer ou tourner un corps ne fait pas glisser la texture ; raccord du halo conservé. | MSH-02, DAT-02 | 3 |
| MSH-05 | Convertir résultats en meshes Bevy avec handles réutilisés ; centraliser matériaux et textures du catalogue. | Deux éditions ne recréent pas les matériaux stables ; assets chargés depuis racine runtime. | MSH-04, APP-05 | 3 |
| MSH-06 | Séparer rendu opaque/verre ; choisir paramètres transparents simples et documenter limites de tri. | Fixture vitrage/coque lisible depuis dehors et dedans ; pas de régression sur normales/culling. | MSH-01, MSH-05 | 3 |
| MSH-07 | Créer matériaux sobres de départ et fallback ; brancher les textures bois existantes par données. | Jeu et galerie utilisent même définition ; les cartes manquantes gardent un rendu utilisable. | MSH-05, GOV-08 | 3 |
| MSH-08 | Créer file de chunks sales, dédoublonnage, priorité caméra/édition et annulation logique des anciens jobs. | Rafale de 100 éditions borne la file ; seul le dernier résultat valable est appliqué. | MSH-03, DAT-11 | 4 |
| MSH-09 | Exécuter meshing en tâches natives sans partager de mutation ; appliquer les résultats au bon tick/frame. | Aucun blocage global par édition simple ; résultat périmé rejeté dans un test de réordonnancement. | MSH-08 | 4 |
| MSH-10 | Fournir chemin web coopératif avec tranches de travail bornées, sans supposer Rayon multithread. | L'input reste traité pendant une reconstruction ; mémoire des buffers et files plafonnée. | MSH-08, APP-06 | 4 |
| MSH-11 | Instrumenter temps mesh, application/upload, triangles et latence commande-vers-image. | Rapport p50/p95 après échauffement ; temps thread principal distinct des jobs et du GPU. | MSH-09, MSH-10, APP-11 | 3 |
| MSH-12 | Tester scènes 1k/10k et stress 50k, adopter greedy sauf seuil D07 réellement dépassé. | Aucun changement de bibliothèque sans trace de profilage ; limites de la démo enregistrées. | MSH-06, MSH-07, MSH-11 | 3 |

## EDT Éditeur de construction et scission

Priorité P1, J2. Modules : `game/build`, `core/commands`, `core/connectivity`. Édition initiale au quai, simulation du corps gelée explicitement. Réparation en vol réservée à une action limitée si nécessaire plus tard.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| EDT-01 | Calculer viewport interactif depuis layout UI ; exclure pointeurs capturés par panneaux et boutons. | Placement impossible sous UI ; tailles 1280×720, 1920×1080 et petite fenêtre vérifiées. | CHR-12, CAB-01, MSH-05 | 3 |
| EDT-02 | Implémenter DDA local avec intervalle de rayon et filtre AABB des corps ; règles aux frontières. | Rayon parallèle, origine intérieure, face exacte et corps tourné sélectionnent une cellule cohérente. | DAT-02, DAT-07, EDT-01 | 4 |
| EDT-03 | Montrer aperçu d'ajout/retrait, face ciblée et raison du refus ; distinguer clic et déplacement caméra. | Un drag ne construit jamais à la relâche ; orientation et visibilité du ghost correctes. | EDT-02 | 3 |
| EDT-04 | Relier outils Add/Remove aux commandes validées ; décrire blocage du dernier élément fonctionnel requis. | Une action produit une transaction ; refus lisible sans modification partielle. | EDT-03, DAT-08 | 3 |
| EDT-05 | Ajouter placement/rotation des composants à empreinte ; afficher attaches et conflits. | Voile et poste ne se superposent pas ; références d'attaches appartiennent au corps visé. | EDT-04, DAT-06 | 4 |
| EDT-06 | Implémenter historique borné de transactions et redo ; invalider redo seulement après nouvelle édition. | 100 opérations puis undo/redo restaurent cellules/composants ; limite mémoire et nouvelle partie explicites. | EDT-04, EDT-05 | 4 |
| EDT-07 | Passer édition/pilotage au quai ; mettre à jour masse/collider/mesh en transaction avant reprise. | Le corps ne repart pas avec un collider périmé ; l'UI indique construction en préparation. | EDT-06, PHY-06, DAT-10, MSH-08, CHR-07 | 4 |
| EDT-08 | Définir graphe structurel par voisinage de faces et liens de composants ; calculer connexité exacte. | Pont fin, boucle de contournement et plusieurs îlots donnent les composantes attendues. | DAT-06, DAT-08 | 4 |
| EDT-09 | Préparer scission sur snapshot, nouvelles origines/IDs et transfert des composants/attaches. | Aucun composant dupliqué ; somme des masses des fragments égale la masse restante. | EDT-08, DAT-10, DAT-11 | 4 |
| EDT-10 | Appliquer scission et vitesses de corps rigide : v fragment = v parent + ω × décalage du centre. | Poses mondiales inchangées à la séparation ; vitesses cohérentes pour corps tournant. | EDT-09, PHY-08 | 4 |
| EDT-11 | Borner débris actifs et durée de calcul ; définir suppression d'une attache et annulation au quai. | Coupure n'entraîne ni joint orphelin ni freeze ; toute limite produit un comportement explicite. | EDT-10, EDT-07 | 3 |
| EDT-12 | Rejouer construction complète, reset récupérable, scission et undo au quai sur natif/web. | Corps reconstruit et contrôlable ; limites D15/D16 et défauts d'UI fermés ou déclarés. | EDT-01, EDT-06, EDT-07, EDT-11, MSH-12 | 3 |

## SAV Blueprints et sauvegardes

Priorité P1, J2. Modules : `core/format`, `game/storage`. Le format JSON initial favorise diagnostic et migration. Les DTO de sauvegarde restent indépendants du layout mémoire et des identifiants Bevy.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| SAV-01 | Définir blueprint v1 : unités, catalogue, cellules, composants, orientations et attaches ; ordre stable. | Un exemple asymétrique documente origine et rotation ; format sans Entity ni Handle runtime. | DAT-12 | 3 |
| SAV-02 | Définir session v1 : poses/vitesses, acteur/mode, progression, réservoir et connexions ; enveloppe versionnée. | Différence blueprint/session explicite ; champs extensibles, pas de cache de solveur sérialisé. | SAV-01, CHR-07 | 3 |
| SAV-03 | Implémenter encode/decode avec plafonds de taille, cellules, corps et valeurs finies. | 16 Mio de fichier max initial, dimensions et références invalides rejetées avant publication. | SAV-01, SAV-02 | 4 |
| SAV-04 | Construire validation sémantique transactionnelle et staging de chargement. | Un fichier corrompu ne remplace jamais la session active ni la sauvegarde précédente. | SAV-03, APP-05 | 3 |
| SAV-05 | Écrire adaptateur disque : temporaire, flush selon garantie choisie, remplacement Windows et copie de secours. | Coupure simulée à chaque étape laisse ancienne ou nouvelle version chargeable ; garantie documentée. | SAV-04 | 4 |
| SAV-06 | Créer adapter IndexedDB avec quota/permission/indisponibilité gérés ; stockage après confirmation de transaction. | Succès UI affiché seulement après commit ; refus conserve session et propose export. | SAV-04, APP-06 | 4 |
| SAV-07 | Implémenter export/import manuel JSON natif et web ; noms sûrs et messages de résultat. | Fichier téléchargé réimporte mêmes données ; annuler sélection ne modifie rien. | SAV-04, SAV-05, SAV-06 | 3 |
| SAV-08 | Définir autosave temporisé aux frontières sûres, sérialisation snapshot et limite des écritures simultanées. | Rafale d'éditions produit des saves bornées ; l'ancienne session ne remplace pas une nouvelle. | SAV-05, SAV-06, EDT-07 | 4 |
| SAV-09 | Écrire tests round-trip de corps déplacé/tourné, composants et valeurs de session ; prévoir migration testée. | Version inconnue et champ manquant ont une issue définie ; origine ne se recentre pas implicitement. | SAV-03, SAV-02 | 3 |
| SAV-10 | Restaurer session en phases : données, corps, composants, attaches, acteur, puis reprise des ticks. | Aucun joint vers corps absent ; le premier tick ne consomme pas une ancienne intention. | SAV-04, DAT-09, CHR-07, APP-07 | 4 |
| SAV-11 | Tester sauvegarde en cours de fermeture, quota saturé, fichier tronqué et recovery depuis backup. | La dernière sauvegarde valide reste accessible ; message distingue non enregistré et enregistré. | SAV-08, SAV-10 | 3 |
| SAV-12 | Valider D17/D18 sur package déplacé et navigateur rechargé ; livrer exemples versionnés. | Session à la limite de contenu autorisée sauvegardable/rechargeable ; ajuster plafond de fichier si nécessaire ; aucun chemin de compilation requis. | SAV-07, SAV-09, SAV-11, EDT-12 | 3 |

## FLT Navigation et voile simplifiée

Priorité P0, J3. Modules : `sim/flight`, `sim/wind`, `view/navigation`. Ce lot éprouve le lien entre construction et comportement avec un modèle volontairement borné et lisible.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| FLT-01 | Définir enveloppe de vol : masse, vitesses, altitude locale, rotation et limites de forces. | Paramètres nommés avec unités et valeurs dans configuration versionnée, pas dispersés en constantes. | PHY-12, DAT-10 | 2 |
| FLT-02 | Créer champ de vent constant puis variation douce déterministe par graine et temps physique. | À position/tick égaux, même vent ; intensité n'est pas liée au framerate de rendu. | FLT-01, APP-09 | 3 |
| FLT-03 | Échantillonner vent relatif à la vitesse du point d'application, incluant rotation du navire. | Une voile excentrée sur un corps tournant reçoit un vent relatif différent et explicable. | FLT-02, PHY-08, DAT-06 | 4 |
| FLT-04 | Appliquer poussée simplifiée selon aire/orientation, avec coefficients bornés et force au bon point. | Voile de profil et face au vent se distinguent ; aucune division nulle ni force infinie. | FLT-03 | 4 |
| FLT-05 | Implémenter réglage de voile et commande de direction avec autorité limitée et coût physique explicite. | Le joueur oriente sa trajectoire sans téléporter le navire ; braquage maximum contrôlé. | FLT-04, CHR-08 | 3 |
| FLT-06 | Ajouter traînée linéaire/angulaire temporellement cohérente et limites de confort. | À force coupée, ralentissement reproductible ; pas de damping différent à 30/120 FPS. | FLT-05 | 3 |
| FLT-07 | Fournir interface de sustentation au composant Aether : force verticale demandée et point d'application. | Mode fixture peut maintenir une coque ; la voile n'est pas responsable du vol stationnaire. | FLT-01, DAT-06, PHY-08 | 3 |
| FLT-08 | Ajouter indicateurs de vent, vitesse, altitude et direction réellement utiles au pilotage. | Trois orientations de voile produisent un retour perceptible sans ouvrir le debug. | FLT-05, CHR-09 | 3 |
| FLT-09 | Construire trajet simple de deux quais et contrôler départ/arrivée, freinage et accostage. | Départ sans collision, île cible visible, arrivée reconnue par un critère spatial stable. | FLT-06, FLT-07, APP-02 | 4 |
| FLT-10 | Comparer deux constructions prescrites : masse ajoutée et voile déplacée ; enregistrer effets. | Variation de comportement attendue observée et expliquée dans une fiche, pas simple changement visuel. | FLT-09, EDT-07 | 3 |
| FLT-11 | Régler confort : mouvements caméra, accélération initiale et commandes ; essai interne court. | Traversée terminable en quelques minutes ; aucune action cachée indispensable. | FLT-08, FLT-10 | 3 |
| FLT-12 | Valider première traversée technique avec sustentation de fixture puis préparer branchement Aether réel. | Navigation fonctionne ; ce résultat ne compte pas comme traversée finale tant qu'AET-12 échoue. | FLT-09, FLT-10, FLT-11 | 2 |

## AET Réservoir et sustentation Aether

Priorité P0, J3. Modules : `core/components`, `sim/aether`, `view/aether`. Une ressource simple donne un coût et une panne compréhensible sans réseau de tuyaux.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| AET-01 | Définir quantité/capacité d'Aether en unités de jeu, débit de consommation et états du sustentateur. | Unités distinctes de masse et force ; quantité bornée entre zéro et capacité. | DAT-06, FLT-07 | 2 |
| AET-02 | Relier réservoir unique du navire aux consommateurs par référence validée, sans réseau spatial. | Référence absente désactive proprement la sustentation avec cause affichable. | AET-01, DAT-01 | 3 |
| AET-03 | Calculer demande de sustentation, capacité maximale et force disponible selon masse et commande. | Une coque trop lourde ne flotte pas artificiellement ; le déficit est mesurable. | AET-02, DAT-10, FLT-07 | 4 |
| AET-04 | Débiter la consommation au tick selon commande choisie ; arrêter force à épuisement sans quantité négative. | Même durée simulée consomme la même quantité à différents FPS ; pause ne consomme pas. | AET-03, APP-10 | 3 |
| AET-05 | Ajouter recharge au quai avec déclenchement clair, débit borné et arrêt à capacité. | Départ/retour rejouables ; rester au quai n'engendre ni dépassement ni double recharge. | AET-04, FLT-09 | 2 |
| AET-06 | Donner aux composants une représentation violette et un indicateur de quantité réel. | Le joueur relie couleur/intensité et niveau ; texte/icône accompagne la couleur. | AET-04, FLT-08 | 3 |
| AET-07 | Signaler réserve basse, déficit de sustentation et panne ; ajouter action de secours cohérente. | Un novice dispose d'un avertissement avant chute ; le secours a une règle distincte documentée. | AET-06, CHR-11 | 3 |
| AET-08 | Persister quantité, état et recharge dans session ; blueprint stocke capacité/configuration sans dupliquer du carburant. | Charger, importer ou undo une coque ne crée pas d'Aether ; règles de remplissage explicites. | AET-05, SAV-10 | 4 |
| AET-09 | Définir réservoir supprimé/scindé et répartition du contenu ; borner inventaire perdu/récupéré. | Somme de quantité conservée sauf perte intentionnelle documentée ; aucun contenu dupliqué. | AET-08, EDT-10 | 3 |
| AET-10 | Tester panne en vol et retour à un quai/checkpoint ; prévenir une reprise irrémédiablement bloquée. | Après réserve nulle, une action claire permet de recommencer sans perdre toute la construction. | AET-07, AET-08, SAV-11 | 3 |
| AET-11 | Régler capacité initiale et coût du parcours ; comparer coque légère et chargée. | La ressource influe sur un choix sans interrompre constamment le pilotage ; mesures enregistrées. | AET-09, AET-10, FLT-10 | 3 |
| AET-12 | Effectuer traversée avec Aether réel, save/reload et panne volontaire ; fermer J3. | Construction, voile, consommation et récupération forment une boucle complète jouable. | AET-06, AET-08, AET-10, AET-11, FLT-12 | 2 |

## CAB Harpon et câble du vaisseau

Priorité P0, J4. Modules : `sim/tether`, `game/aim`, `view/rope`. Le câble initial n'enroule pas la géométrie ; cette limite doit être visible dans la scène et la documentation.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| CAB-01 | Dès J1, vérifier API/features DistanceJoint sur coque de fixture ; créer ancre fixe et longueur maximale. | À distance inférieure à L, pas de compression imposée ; limite supérieure active ; scène finie dix minutes. | PHY-12 | 3 |
| CAB-02 | Représenter ancrage fixe et harpon par IDs et points locaux, avec cycle attaché/détaché. | Une rotation de coque transporte le bon point ; références jamais liées au renderer. | CAB-01, DAT-01, DAT-02 | 3 |
| CAB-03 | Créer ciblage à portée et ligne de vue ; feedback valide/invalide ; limite initiale un câble actif. | Cible derrière île ou hors portée refusée ; bouton ne crée pas plusieurs joints. | CAB-02, CHR-08, EDT-02 | 3 |
| CAB-04 | Attacher au tick sans imposer instantanément une longueur incompatible ; borner paramètres autorisés. | Accrochage ne téléporte pas la coque ; mauvaise configuration échoue explicitement. | CAB-03, APP-07 | 4 |
| CAB-05 | Libérer ou couper le câble en conservant l'état physique courant ; effacer le joint une seule fois. | Trajectoire continue à la libération, sans ancienne vitesse du runner ni impulsion ajoutée. | CAB-04, PHY-08 | 3 |
| CAB-06 | Lire l'effort disponible dans le solveur ou définir un indicateur approché clairement nommé. | HUD ne présente pas une force inventée en Newtons ; tension estimée identifiée si nécessaire. | CAB-04 | 3 |
| CAB-07 | Rendre câble lâche/tendu entre points interpolés, avec couleur et son de tension. | Extrémités suivent coque/ancre sans tremblement majeur ; aucun coût physique dépend du nombre de segments visuels. | CAB-05, CAB-06, CHR-09 | 3 |
| CAB-08 | Définir câble traversant un obstacle : bloquer attache initiale, avertir ou rompre selon règle du prototype. | Comportement reproductible ; aucun enroulement promis si absent. | CAB-03, CAB-07 | 3 |
| CAB-09 | Gérer destruction/scission de la cellule d'attache et suppression du corps ; transférer ou libérer. | Pas de joint orphelin ; corps fragment receveur correct quand transfert prévu. | CAB-02, EDT-11 | 4 |
| CAB-10 | Sauver et restaurer câble après reconstruction des corps ; ID stable et longueur validée. | Reload en tension ne joint pas deux mauvais corps ; valeur hors limite rejetée. | CAB-09, SAV-10 | 3 |
| CAB-11 | Éprouver masse maximale, vitesse maximale, pause/reprise et 30/60/120 FPS ; enregistrer erreur de longueur. | Erreur p95 visée < 2 % hors accrochage ; pas d'énergie explosive ni NaN sur dix minutes. | CAB-05, CAB-08, CAB-10, PHY-11 | 4 |
| CAB-12 | Régler une manœuvre de balancier autour d'ancre ; appliquer repli borné si échec. | Le joueur change de direction par attache/libération ; D12 et limites connues enregistrées. | CAB-07, CAB-11, AET-12 | 3 |

## CUR Courant et archipel initial

Priorité P0, J4. Modules : `core/fields`, `sim/current`, `game/scenario`, `view/current`. Un archipel réalisé à la main et borné suffit ; pas de streaming ni de génération de monde.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| CUR-01 | Définir trajet 3D du courant avec segments/tube, rayon, direction, intensité et zones de transition. | Paramètres en mètres et secondes ; géométrie affichable pour debug et jeu. | FLT-02, GOV-09 | 3 |
| CUR-02 | Implémenter échantillonnage continu de vitesse cible et poids d'influence près des joints de segments. | Frontière et jonction ne changent pas brutalement la direction ; points hors courant poids nul. | CUR-01 | 4 |
| CUR-03 | Convertir champ en force de relaxation bornée sur la vitesse relative, avec constante de temps explicite. | Entrée/extraction progressives ; le courant ne téléporte pas les corps ni ne réécrit leur pose. | CUR-02, PHY-08 | 3 |
| CUR-04 | Définir quelles entités reçoivent le courant et éviter double application au personnage attaché. | Vaisseau/débris libres reçoivent force une fois ; ancre fixe reste fixe. | CUR-03, CHR-06 | 3 |
| CUR-05 | Composer un embranchement avec choix de trajectoire et destination de récupération pour branche ratée. | Les deux issues sont atteignables par la physique, sans déclencheur téléportant arbitrairement. | CUR-03, FLT-09 | 4 |
| CUR-06 | Placer ancre et obstacles lisibles autour de l'embranchement ; régler portée et fenêtre d'action. | CAB permet une sortie alternative, avec temps de réaction vérifié en essai interne. | CUR-05, CAB-12 | 3 |
| CUR-07 | Dessiner ruban violet, particules de direction et variation de densité au cœur du courant. | Direction compréhensible depuis cockpit ; éléments visuels ne modifient pas le champ physique. | CUR-02, APP-04 | 4 |
| CUR-08 | Assembler deux îles, quais et repères de navigation ; préserver lignes de vue du trajet et du but. | Joueur voit où aller et où s'ancrer sans carte volumétrique complexe. | CUR-06, CUR-07 | 3 |
| CUR-09 | Définir progression départ, première traversée, manœuvre, arrivée ; événements persistants. | Objectif ne se déclenche pas deux fois au reload ; reprise conserve étape logique. | CUR-08, SAV-10 | 3 |
| CUR-10 | Ajouter checkpoint sûr et réparation/ravitaillement entre essais ; garder coût d'échec court. | Branche ratée ne force pas à reconstruire intégralement ; boucle recommençable. | CUR-09, AET-10 | 3 |
| CUR-11 | Tester combinaisons courant/voile/câble, réserve faible et navire chargé dans le même scénario. | Toutes forces bornées ; trajectoires possibles documentées, aucun cas impossible par défaut. | CUR-10, FLT-10, CAB-11 | 4 |
| CUR-12 | Jouer scénario complet sans commandes de debug et préparer capture de référence J4. | Les décisions de construction et pilotage expliquent réussite/échec ; début et fin accessibles. | CUR-07, CUR-09, CUR-10, CUR-11 | 2 |

## UX Lisibilité et interaction

Priorité P1, J5 ; commencer les éléments utiles dès J3. Modules : `view/ui`, `view/feedback`, `view/materials`. Limiter l'art nouveau au besoin du scénario ; réutiliser les assets et thèmes existants.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| UX-01 | Définir thème partagé, hiérarchie des actions et vocabulaire français/clé de texte centralisé. | Construire, piloter, s'ancrer et recharger utilisent les mêmes noms partout. | GOV-02, APP-02 | 2 |
| UX-02 | Rendre panneaux adaptatifs au viewport et à l'échelle UI ; min-size et état compact explicites. | Aucun panneau hors écran aux tailles cibles ; zone 3D calculée reste alignée. | EDT-01, UX-01 | 3 |
| UX-03 | Afficher raccourcis contextuels, verrouillage quai/pilotage et explications de refus de commande. | Une action indisponible n'a pas l'air de fonctionner ; aucune commande Capture vide. | EDT-07, CHR-07, UX-02 | 3 |
| UX-04 | Créer tutoriel par actions réelles : bloc, voile, énergie, départ, câble ; étapes rejouables. | Progression ne dépend pas d'une durée fixe et fonctionne après save/reload. | CUR-09, UX-03 | 4 |
| UX-05 | Ajouter HUD compact de réserve, sustentation, vitesse, objectif et état de câble. | Les causes de panne sont distinctes ; information principale lisible sans overlay dev. | AET-07, CAB-06, CUR-09 | 3 |
| UX-06 | Ajouter réglages volume, sensibilité, inversion et caméra ; persister préférences séparées de la session. | Nouvelle partie ne perd pas les préférences ; valeurs extrêmes validées. | CHR-09, SAV-05, UX-01 | 3 |
| UX-07 | Préparer feedback audio utile : pose/refus, tension, réserve et arrivée, avec provenance des sons. | Sons désactivables ; aucun signal essentiel exclusivement audio ; mix non saturé. | UX-05, GOV-08 | 4 |
| UX-08 | Vérifier contrastes, texte, sens du vent et états indépendants de la couleur ; réduire mouvements caméra. | Les états restent reconnaissables en niveaux de gris et avec effets caméra réduits. | UX-05, UX-06, CUR-07 | 3 |
| UX-09 | Centraliser PBR du bois, sampler et modes debug ; supprimer paths de compilation et doublons runtime. | Package utilise mêmes textures que galerie ; end-grain éclairé ou exception artistique documentée. | MSH-07, GOV-08 | 4 |
| UX-10 | Créer galerie canonique : corps tourné, verre, vrais chunks, coque et composants, plusieurs lumières utiles. | Caméras/graines/résolution fixes ; assets attendus et timeout ; erreurs donnent sortie non nulle. | UX-09, MSH-06, APP-11 | 4 |
| UX-11 | Capturer et relire les fixtures ainsi qu'une scène de jeu à vitesse normale. | Défauts de couture, lisibilité et UI notés avec images ; renderer identique à celui du jeu. | UX-10, CUR-12 | 3 |
| UX-12 | Valider compréhension de l'interface lors d'un essai interne sans explication orale. | Blocages du tutoriel corrigés ; choix purement décoratifs reportés si hors budget. | UX-04, UX-07, UX-08, UX-11 | 2 |

## WEB Packaging et distribution

Priorité P1, J5 ; les builds web ont déjà commencé à APP-06. Modules : `game/platform`, `tools/package`, `web/`. Les tâches de préparation ne publient pas automatiquement une version.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| WEB-01 | Séparer assets runtime et sources d'auteur ; produire manifeste avec tailles et empreintes. | Aucun prompt, script source ou PNG candidat inutile dans le bundle ; actifs requis tous présents. | GOV-08, UX-09 | 3 |
| WEB-02 | Produire package Windows autonome et définir répertoires lecture seule/données utilisateur. | Démarrage depuis dossier neuf sans dépôt ; création de sauvegarde à un emplacement permis. | SAV-12, WEB-01 | 3 |
| WEB-03 | Configurer bundle Bevy CLI avec préfixe de déploiement ; fallback wrapper seulement après 4 h de blocage. | Routes et assets fonctionnent sous `/` et sous un préfixe ; aucune dépendance Bash GNU. | ENV-07, WEB-01, APP-06 | 4 |
| WEB-04 | Versionner noms/manifestes du JS, WASM et assets ; éviter mélange de versions après déploiement. | Rechargement avec ancien cache charge un ensemble cohérent ou affiche reprise contrôlée. | WEB-03 | 3 |
| WEB-05 | Afficher phases téléchargement, initialisation et première image jouable ; gérer erreurs et retry. | Barre reflète sa phase ; double clic et réseau coupé ne bloquent pas définitivement la session. | WEB-04, APP-05 | 3 |
| WEB-06 | Définir URL initiale et retour menu cohérents avec une seule application ; traiter back/forward. | Rafraîchir, revenir et relancer donnent URL et état concordants, sans moteur doublé. | WEB-05, APP-03 | 3 |
| WEB-07 | Tester clavier/souris, focus, redimensionnement, plein écran et permissions de téléchargement. | Aucun input bloqué après changement d'onglet ; session sauvegardée ou erreur signalée. | WEB-06, SAV-06, CHR-08 | 3 |
| WEB-08 | Mesurer bundle compressé, mémoire et démarrage froid/chaud ; identifier le plus gros contributeur. | Rapport sur réseau simulé 50 Mbit/s ; objectif initial première interaction < 20 s, à confirmer. | WEB-05, MSH-10, APP-11 | 3 |
| WEB-09 | Ajouter smoke Playwright sur menu, première image, construction, save/load et retour ; conserver erreurs dès le début. | Le test échoue si erreur runtime ou asset critique absent, pas seulement si canvas manquant. | WEB-07, UX-12 | 4 |
| WEB-10 | Préparer pipeline de release : réutilisation artefact validé, checksums, numéro de version et notes. | Build distribué traçable au commit testé ; sortie des laboratoires exclue. | WEB-02, WEB-08, WEB-09 | 3 |
| WEB-11 | Vérifier sauvegardes entre deux builds compatibles et préparer restauration de version précédente. | Procédure de retour testée localement ; compatibilité de format indiquée sans faux résultat live. | WEB-10, SAV-09 | 3 |
| WEB-12 | Revue de package natif/web, capacités affichées et procédure de publication pour décision ultérieure. | Archive prête à distribuer avec limitations et guide ; aucune publication déduite de la planification. | WEB-02, WEB-08, WEB-09, WEB-10, WEB-11 | 2 |

## QA Robustesse, budgets et régressions

Priorité P1, J5. Livrables : scénarios reproductibles, résultats sur la machine de référence et liste des défauts. Les tests de chaque lot existent déjà ; ce lot vérifie leurs interactions et le produit distribué.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| QA-01 | Constituer matrice courte des parcours critiques : nouvelle partie, construire, traverser, câble, échec, reprise. | Chaque parcours cite état initial, actions et résultat ; distinguer automatisation headless, navigateur et observation humaine. | CUR-12, SAV-12, GOV-10 | 2 |
| QA-02 | Exécuter scénario intégré de 30 minutes avec navigation, édition au quai et sauvegardes répétées. | Aucun crash, NaN, corps perdu ou progression irrécupérable ; anomalies conservées avec graine et dernière action. | QA-01, UX-12 | 3 |
| QA-03 | Mesurer vingt cycles menu/session et cent éditions/annulations/scissions dans une fixture bornée. | Entités, handles et files reviennent à un niveau stable ; croissance mémoire durable expliquée ou corrigée. | APP-03, EDT-12, APP-11 | 3 |
| QA-04 | Vérifier synchronisation poses physiques, meshes, picking, ancrages et interpolation après scission/reprise. | Un point repère reste cohérent à 30/60/120 FPS ; erreur tolérée explicitée en unités monde, sans masquer un tick de retard logique. | EDT-12, CAB-10, PHY-11 | 3 |
| QA-05 | Rejouer intentions enregistrées dans le même build et comparer invariants, énergie bornée et progression. | Les écarts sont mesurés ; aucune promesse de déterminisme bit à bit entre CPU, natif et WASM. | APP-09, CUR-11, QA-04 | 3 |
| QA-06 | Renforcer validation des entrées : JSON tronqué, IDs répétés, valeurs non finies, dimensions extrêmes et jobs obsolètes. | Cas refusés sans mutation partielle ni allocation excessive ; tests génératifs bornés sur codecs et coordonnées sensibles. | SAV-03, SAV-04, MSH-08, DAT-12 | 4 |
| QA-07 | Mesurer scène cible à 1080p : navire 10 000 cellules, au plus 32 corps actifs, îles et effets du parcours. | Rapporter p50/p95/p99 CPU/GPU/frame et mémoire ; objectif proposé p95 frame ≤ 16,7 ms natif, ≤ 33,3 ms web sur machine ENV-01. | APP-11, WEB-08, CUR-12 | 4 |
| QA-08 | Tester dépassement des limites : 50 000 cellules en fixture de stress, import hors borne, trop de débris et backlog de meshing. | Refus ou dégradation contrôlés ; l'essai de stress ne change pas la limite publique de 10 000 cellules par navire. | QA-07, EDT-11, SAV-03, MSH-11 | 3 |
| QA-09 | Injecter échec disque/quota web, perte de focus, fermeture pendant save et asset absent dans le package. | Dernière sauvegarde valide récupérable ; écran d'erreur permet reprise ; pas de succès affiché sur écriture ratée. | WEB-12, SAV-11, APP-10 | 4 |
| QA-10 | Exécuter matrice Windows natif, Chrome WebGPU et Edge WebGPU ; documenter pilotes/résolutions testés. | Rapport distingue vérifié, échoué et non testé ; fallback WebGL2 testé seulement s'il a été retenu. | QA-02, QA-09, WEB-12 | 4 |
| QA-11 | Sélectionner les régressions à maintenir en CI et conserver artefacts utiles sans captures inutiles à chaque commit. | Une régression représentative est détectée ; temps de CI connu ; tests visuels séparés des tests sans GPU. | QA-03, QA-05, QA-06, WEB-09 | 3 |
| QA-12 | Trier défauts et fermer le jalon technique : P0 perte/crash, P1 parcours bloqué, P2 gêne bornée, P3 cosmétique. | Aucun P0/P1 ouvert pour les cibles annoncées ; budgets dépassés ont une correction ou une réduction de périmètre explicite. | QA-07, QA-08, QA-10, QA-11 | 2 |

## PLY Validation du jeu et préparation de la démo

Priorité P1, J5. Livrables : protocole, observations et décision produit. Les tâches préparent les échanges ; Damien organise les invitations ou autorise explicitement leur envoi. Le nombre de testeurs ne garantit pas une validation de marché.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| PLY-01 | Écrire protocole de 10–15 minutes sans assistance : construire une amélioration, partir, utiliser courant/câble, revenir. | Objectif compréhensible, point de départ reproductible ; secours et arrêt de test prévus si le parcours bloque. | GOV-02, UX-12, CUR-12 | 2 |
| PLY-02 | Définir grille d'observation : temps jusqu'au départ, erreurs, compréhension de l'Aether et envie de modifier le vaisseau. | Séparer fait observé, parole du joueur et interprétation ; aucun objectif de FPS utilisé comme mesure du plaisir. | PLY-01 | 1 |
| PLY-03 | Préparer build candidat, fiche de commandes et instructions de lancement pour les personnes choisies par Damien. | Package identifié et réinstallable ; l'invitation décrit la durée et les informations recueillies, sans envoi automatique. | QA-12, WEB-12, PLY-01 | 2 |
| PLY-04 | Faire une répétition du protocole avec une personne connaissant peu la version pour déceler ses défauts. | Consignes ne révèlent pas la solution ; problèmes d'installation distincts des problèmes du jeu. | PLY-02, PLY-03 | 2 |
| PLY-05 | Observer cinq personnes séparément ; prendre notes et, avec leur accord, captures utiles du parcours. | Cinq fiches exploitables, aide apportée consignée ; arrêter et corriger si un défaut empêche tous les essais. | PLY-04 | 4 |
| PLY-06 | Consolider observations sans surinterpréter le petit échantillon ; classer fréquence, gravité et étape concernée. | Les nombres indiquent leur dénominateur ; un joueur satisfait ne masque pas les blocages des autres. | PLY-05 | 2 |
| PLY-07 | Évaluer hypothèse centrale : une modification du vaisseau produit-elle un effet compris et une nouvelle tentative volontaire ? | Rapport cite exemples et contre-exemples ; seuil indicatif de compréhension autonome 4/5, à confronter aux notes qualitatives. | PLY-06, FLT-10 | 2 |
| PLY-08 | Choisir au plus trois corrections de parcours et une hypothèse de plaisir pour un cycle court ; estimer avant de modifier. | Décision liée à des observations ; nouvelle fonctionnalité majeure devient une proposition séparée avec coût. | PLY-07 | 2 |
| PLY-09 | Réaliser le cycle de corrections sélectionné, en conservant la portée de la démo. | Limite initiale de 8 h ; tests affectés repassés ; tout dépassement provoque une nouvelle décision, pas une extension silencieuse. | PLY-08 | 8 |
| PLY-10 | Faire un second essai ciblé avec au moins deux personnes disponibles, en distinguant nouveaux joueurs et joueurs déjà formés. | L'amélioration attendue est observée ou l'hypothèse est rejetée ; limites de comparaison explicites. | PLY-09 | 3 |
| PLY-11 | Préparer notes de version, contrôles, limitations connues, procédure de récupération et archive du candidat retenu. | Les documents correspondent au build ; marche embarquée, navigateurs et limites voxel annoncés honnêtement. | PLY-10, WEB-11, QA-12 | 2 |
| PLY-12 | Décider avec Damien : démo prête, second cycle ciblé ou changement de priorité ; classer les extensions par bénéfice constaté. | Dossier J5 comprend build, preuves techniques, cinq observations, dettes et prochaine décision ; aucune publication implicite. | PLY-07, PLY-10, PLY-11 | 2 |

## FLEX Flexibles avancés — conditionnel

Priorité P2, après J5. **Activation :** le câble rigide ou la voile simplifiée ne permettent pas une interaction jugée nécessaire par PLY-12. Ce lot contient des expériences bornées, pas une promesse de simulation de tissu complète. Modules éventuels : `aether_flex` et adaptateurs `sim/flex`. Les résultats de l'ancien solveur servent de référence, pas de garantie de sûreté.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| FLEX-01 | Formuler l'interaction manquante et une scène où le modèle simplifié échoue ; accepter ou refuser l'activation. | Une observation de jeu justifie le coût ; si aucune n'existe, le lot entier est reporté avec motif. | PLY-12 | 2 |
| FLEX-02 | Isoler noyau flexible Rust avec `glam` aligné et états privés ; définir nœuds, contraintes et révisions de topologie. | Aucune dépendance à `game` ; mutation par API contrôlée ; ancienne voie parallèle unsafe absente du défaut. | FLEX-01, ENV-05 | 4 |
| FLEX-03 | Porter intégration séquentielle, masses, ancrages et amortissement en secondes ; conserver fixtures historiques utiles. | Nœud libre, fixé et pendule de référence restent finis ; changer sous-pas ne change pas arbitrairement le damping. | FLEX-02 | 4 |
| FLEX-04 | Commencer XPBD à 10 sous-pas × 1 itération ; mesurer erreur et coût sur une seule scène justifiée. | Réglage et unité de compliance consignés ; essai 20×1 seulement si seuil d'erreur prédéfini échoue, investigation totale ≤ 4 h. | FLEX-03 | 4 |
| FLEX-05 | Implémenter corde unilatérale et tension exploitable ; distinguer mou, tendu et rupture. | Corde molle ne pousse pas ses extrémités ; unités et influence du pas vérifiées sur masse suspendue. | FLEX-04 | 4 |
| FLEX-06 | Définir un seul transfert d'efforts avec Avian et l'ordre des sous-pas ; tester corps/nœud et deux corps. | Réaction aux ancrages mesurée, aucune double force ; énergie reste bornée dans une fixture sans apport externe. | FLEX-05, PHY-08 | 6 |
| FLEX-07 | Prototyper voile à grille grossière et forces de vent réparties ; rendre une surface plus détaillée sans changer la poussée. | Raffiner le rendu conserve réponse mécanique ; budget CPU comparé à la voile simplifiée sur la même traversée. | FLEX-06, FLT-04 | 6 |
| FLEX-08 | Traiter suppression et rupture de contrainte comme transaction de topologie ; conserver historique et attaches valides. | Aucun index pendant ; rupture a un seuil observable avant correction ; save/reload restitue le graphe supporté. | FLEX-05, SAV-09 | 4 |
| FLEX-09 | Tester contacts simplifiés de corde seulement si la scène FLEX-01 les exige ; borner nombre de contacts et rayon. | Une interaction définie fonctionne ou est déclarée hors portée ; aucun wrapping complet promis par ce prototype de 4 h. | FLEX-06 | 4 |
| FLEX-10 | Profiler avant toute parallélisation ; si nécessaire, établir groupes sans conflits et comparer à la référence séquentielle. | Invariants de disjonction vérifiés après mutation ; implémentation sûre privilégiée ; gain mesuré avant activation. | FLEX-08, FLEX-09 | 4 |
| FLEX-11 | Mesurer le même graphe natif/WASM et sauvegarde/reprise avec un navire mobile. | Budgets du jeu restent respectés ; mode simple utilisable si flexibles désactivés ; erreurs et capacités affichées. | FLEX-07, FLEX-08, FLEX-10, QA-07 | 4 |
| FLEX-12 | Faire essai joueur ciblé et décider intégration, maintien expérimental ou abandon. | Gain de compréhension/plaisir comparé au surcoût ; nouvelle liste de production si le prototype nécessite encore du travail. | FLEX-11, PLY-02 | 2 |

## OPT Optimisations déclenchées par mesure — conditionnel

Priorité P2, sauf dépassement bloquant un jalon. **Activation :** un budget enregistré dans MSH-12, QA-07 ou WEB-08 est dépassé sur la scène et le matériel de référence. Les tâches OPT-01 à OPT-04 forment un essai de meshing de **8 h maximum** ; les autres ne s'exécutent que sur leur propre diagnostic. Aucun gain annoncé par une bibliothèque n'est repris comme objectif garanti.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| OPT-01 | Capturer un profil d'édition lente et attribuer le coût : extraction, meshing, allocation, upload, collider ou attente. | Une mesure identifie le poste dominant ; si ce n'est pas le mesher, OPT-02 à 04 sont non applicables. | MSH-12, APP-11 | 2 |
| OPT-02 | Si meshing dominant, brancher `binary-greedy-meshing` 0.5.2 sur fixture via adaptateur isolé. | Coût de conversion/padding et granularité 16³ inclus ; aucune modification globale du format de chunks pour satisfaire le benchmark. | OPT-01 | 2 |
| OPT-03 | Vérifier sorties de l'adaptateur : faces, matériaux, halos, UV, transparence et cellules de coordonnées négatives. | Fixtures de correction de MSH réutilisées ; incompatibilité ou travail hors budget autorise le rejet de l'alternative. | OPT-02 | 2 |
| OPT-04 | Mesurer gain complet et choisir conserver greedy simple ou intégrer adaptateur. | Comparaison mêmes données/build/machine ; budget de latence atteint ou prochaine cause identifiée ; décision prise à 8 h cumulées. | OPT-03 | 2 |
| OPT-05 | Si mémoire des chunks domine, mesurer occupation puis prototyper palette locale ou stockage uniforme des chunks pleins/vides. | Gain net incluant décompression/jobs ; sauvegardes inchangées ou migrées ; pas de régression des éditions mesurées. | QA-08, DAT-04 | 4 |
| OPT-06 | Si l'occlusion améliore la lecture observée, ajouter AO par sommet avec règle de fusion compatible. | Coutures, diagonales et changement de voisin testés ; coût et hausse de triangles comparés ; désactivation possible. | UX-11, MSH-12, PLY-07 | 4 |
| OPT-07 | Si textures/draw calls dominent, essayer une seule évolution d'atlas ou d'array de textures sur le renderer partagé. | Pas de couture/mipmap incorrect sur fixture tournée ; pas de shader propre à la galerie ; gain GPU mesuré. | QA-07, UX-09, UX-10 | 4 |
| OPT-08 | Si le travail coopératif web dépasse durablement les budgets, évaluer workers/threads avec contraintes d'hébergement réelles. | Essai ≤ 4 h ; coût de transfert et configuration documentés ; fallback coopératif conservé si gain insuffisant. | WEB-08, MSH-10, QA-07 | 4 |
| OPT-09 | Si les colliders dominent les éditions, regrouper reconstructions et réutiliser résultats par révision de corps. | Aucune collision périmée après transaction ; mesure inclut upload et temps jusqu'à corps de nouveau éditable. | OPT-01, EDT-07, PHY-06 | 4 |
| OPT-10 | Si le démarrage web est trop long, réduire le contributeur dominant identifié : features, compression ou assets initiaux. | Nouvelle mesure froide/chaude comparable ; chargement différé conserve erreurs et reprise ; intégrité du package validée. | WEB-08, WEB-04, QA-10 | 4 |
| OPT-11 | Si trop de débris actifs coûtent cher, éprouver sommeil, regroupement visuel ou expiration annoncée des fragments secondaires. | Règle n'efface pas le navire principal ni une ressource requise ; budgets tenus et conséquences produit approuvées. | QA-08, EDT-11, AET-09 | 4 |
| OPT-12 | Mettre à jour baseline de performance et décision de chaque optimisation tentée ; retirer prototypes inutiles du chemin distribué. | Comparaison avant/après et régressions pertinentes repassées ; pas d'obligation de réaliser les optimisations non activées. | OPT-01 | 2 |

## EXT Extensions de produit — hors première démo

Priorité P3. **Activation :** décision explicite issue de PLY-12, avec une interaction attendue, une limite de coût et une cible. Ces tâches cadrent des prototypes et études limités ; leurs heures ne couvrent pas leur industrialisation. Le multijoueur, les fluides volumiques et le monde infini ne sont pas des promesses du reboot initial.

| ID | Travail et livrable | Acceptation et preuve | Prérequis | h |
| --- | --- | --- | --- | --- |
| EXT-01 | Choisir une extension à partir des observations et rédiger hypothèse, scène minimale, plafond et condition d'abandon. | Une seule extension produit active ; coût de maintenance et influence sur sauvegardes identifiés avant prototype. | PLY-12 | 2 |
| EXT-02 | Si import `.vox` utile, intégrer `dot_vox` 5.2 avec limites et mapping palette/axes ; traiter modèles et origine explicitement. | Fichier asymétrique de référence importé correctement ; matériaux inconnus signalés ; le codec ne devient pas format de session. | EXT-01, SAV-03, DAT-05 | 6 |
| EXT-03 | Si échange externe utile, exporter `.vox` puis faire aller-retour réel dans MagicaVoxel sur fixture asymétrique. | Axes, couleurs, dimensions et origine vérifiés ; pertes de composants/état annoncées à l'utilisateur. | EXT-02, SAV-07 | 4 |
| EXT-04 | Prototyper réseau Aether par segments connectés avec capacité commune et débits bornés, sur un navire de test. | Ajout/retrait de tuyau relie/isole correctement réservoir et consommateur ; conservation mesurée sans simulation volumétrique. | EXT-01, AET-12 | 6 |
| EXT-05 | Étudier coupure, fuite et scission du réseau ; définir distribution de quantité lors des changements de topologie. | Deux fixtures conservent quantité moins fuite/consommation explicites ; transaction compatible avec EDT et SAV. | EXT-04, EDT-11, AET-09 | 4 |
| EXT-06 | Seulement si l'eau volumique a une interaction nécessaire, prototyper automate borné dans un bassin statique. | Conservation et frontières mesurées ; budget et limites connus ; couplage complet aux navires reste une étude séparée. | EXT-01 | 6 |
| EXT-07 | Prototyper un seul état de matériau motivé par le jeu, par exemple mouillé ou endommagé, avec effet visible. | Coût de stockage, sauvegarde et effet mécanique identifiés ; pas de simulation combinée feu/eau/chaleur sans nouvelle décision. | EXT-01, DAT-05, SAV-09 | 4 |
| EXT-08 | Étudier streaming d'îles si le nombre d'îles nécessaire dépasse la scène chargée ; charger/décharger deux régions de fixture. | Attaches, checkpoints et corps actifs restent cohérents ; cache borné ; aucune revendication de monde infini. | EXT-01, CUR-08, SAV-10 | 6 |
| EXT-09 | Si erreur de précision observée loin de l'origine, quantifier seuil puis essayer repère flottant et version compatible de `big_space`. | Ne pas intégrer 0.12 directement à Bevy 0.19 ; dérive avant/après et impact Avian/joints/picking mesurés ; rejet possible. | EXT-01, QA-04, PHY-11 | 4 |
| EXT-10 | Prototyper génération reproductible d'une petite île utile au parcours, avec contraintes d'atterrissage et ressources lisibles. | Graine et version du générateur enregistrées ; partie rechargeable malgré évolution ultérieure du générateur. | EXT-01, CUR-08, SAV-02 | 6 |
| EXT-11 | Si coopération demandée, rédiger étude réseau bornée : autorité serveur, commandes d'édition, IDs, reconnexion et coût physique. | Comparer besoins concrets à la simulation existante ; aucune supposition de lockstep déterministe ; prototype réseau fait l'objet d'un autre plan. | EXT-01, QA-05, DAT-08 | 4 |
| EXT-12 | Pour l'extension choisie, conclure conserver/abandonner, chiffrer mise en production et mettre à jour décisions actives. | Preuve d'utilité et risques restants ; prototype n'est pas déclaré fini pour la production ; autres extensions restent reportées. | EXT-01 | 2 |

## Règles de clôture et premiers travaux

Une dépendance conditionnelle n'exige pas d'implémenter son alternative : son statut « non applicable » doit citer la mesure ou la décision qui la rend inutile. OPT-12 et EXT-12 exigent la clôture de toutes les tâches réellement activées de leur lot, même si seules leurs entrées communes figurent dans la colonne des prérequis. FLEX-12 n'est pas une étape obligatoire pour J5.

Les premiers travaux sont GOV-01 à GOV-05, GOV-08 à GOV-10 et ENV-01/02. Dès que les frontières et l'environnement sont prêts, ENV-03 à ENV-12 puis APP peuvent commencer. Les tâches documentaires restantes se regroupent avec le travail technique concerné ; elles ne justifient pas une semaine de réunions préalable.

À la fin de J0, réestimer PHY et CHR avec les temps de compilation réels. À la fin de J1, confirmer la marche embarquée ou afficher le repli, puis réestimer le reste. À la fin de J3, jouer la traversée avant d'ajouter des systèmes. À la fin de J5, décider sur les observations si des flexibles ou un réseau d'Aether apportent plus qu'une amélioration des contrôles.

Le développement ultérieur doit commencer par ces travaux de socle. Ce document ne constitue ni une livraison du reboot ni une validation technique de ses hypothèses.
