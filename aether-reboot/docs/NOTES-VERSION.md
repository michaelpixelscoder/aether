# Aether Isles 0.2.0 — Les neuf archipels

Le monde s'étend à neuf régions et 207 escales, avec des capitales détaillées, des satellites, deux ensembles de cavernes et des voies de courant sur plusieurs kilomètres. La version 0.2 est en cours de validation ; [dist/latest.json](../dist/latest.json) pointe encore sur la distribution 0.1 tant que la nouvelle archive n'est pas vérifiée.

## Contenu jouable

- Construction au quai en bois, métal ou verre ; équipements orientables, aperçu des refus, annulation/rétablissement et séparation de fragments.
- Navigation physique : deux hélices pour avancer même sans vent, marche arrière lente, vent favorable accélérateur, dérive maîtrisée et inertie conservée à haute vitesse. Sustentation Aether, consommation, surcharge, freinage, recharge et secours au dernier quai.
- Rafales douces et voile voxel animée : toile et emblème réagissent au vent apparent, avec pause et mouvement réduit respectés.
- Quinze grands courants, boucles et spirales ; deux parcours courts vers les jardins et le refuge conservés. Quais apaisés, ancres physiques et harpon à câble unilatéral.
- Atlas des régions et destinations ; découverte persistante, primes, six marchandises, soute de 400 unités avec masse réelle, commerce régional et neuf contrats de livraison.
- Collecte des cristaux et de l'eau, raffinage en Aether ; portails entre ports découverts avec coût et délai de recharge.
- Gardiens et planeurs des cavernes animés et observables ; cinq dirigeables marchands et trois navires à voile suivent six circuits continus avec collision.
- Pluie, neige, cendres, rafales régionales, cycle lumineux, nuages volumétriques traversables, cascades et bassins animés. Moulins et engrenages tournent avec l'horloge du jeu.
- Personnage animé, marche et saut sur le navire ; entrée au poste seulement depuis une place disponible.
- Sauvegarde complète Windows/IndexedDB, copie de secours, export/import de construction et préférences séparées.
- Interface française, volume et caméra réglables, erreurs de chargement récupérables, galerie utilisant le même renderer.
- Dix-huit commandes remappables, navigation des boutons au clavier, huit conseils rejouables et mode photo F8. La caméra se rapproche devant une paroi ou un plafond.
- Grande coque entièrement éditable, carène effilée, cabine, deux voiles indigo, ferrures et lanternes. Douze modèles d'îles détaillées avec LOD, voûtes sculptées, textures partagées et sources Blender fournies.
- Distribution d'Aether installable au quai : équipements connectés par branches, vannes effectives, réparation de fuites, historique conservant la réserve actuelle. Conduites 3D et pompes jouables encore ouvertes.
- Migration des sauvegardes V1/V2 ; motorisation des anciennes coques au comptoir. Les sources actuelles sauvegardent en V3, refusé par les anciens lecteurs.

## Première partie

Extraire tout le ZIP Windows puis lancer `Aether-Isles.exe` avec son dossier `assets`. Entrée commence une partie ; `F` libère les amarres, `Z/W/↑` avance, `Q/D` dirige, `E/C` règle l'altitude et `S` freine. `K` recule doucement, `J/X` oriente la voile. `M` ouvre l'atlas ; `B` interagit à proximité. `Tab` ouvre l'atelier au quai et `R` ramène au dernier quai. Les [commandes complètes](CONTROLS.md) décrivent le commerce, les portails et la marche.

## Portée de cette livraison

Windows et WebGPU desktop sont les cibles vérifiées. Les limites sont 10 000 cellules par navire, 128 cellules par axe, 32 corps et 16 Mio par session. La construction est limitée au quai. Le verre utilise une transparence simple ; la voile se déforme par morph targets sans solveur de tissu ; le câble ne s'enroule pas autour des obstacles.

Le monde est déterministe et borné ; les collisions sont chargées par proximité et les modèles passent à leur LOD distant. Les îles ne sont pas destructibles. Les bassins et cascades représentent des surfaces d'eau animées, sans solveur hydraulique ; les machines des décors sont animées sans réseau de transmission simulé. L'observation de la faune n'est pas un système de combat. Les comptoirs portent le commerce, les dirigeables assurent le trafic visuel et physique.

Les résultats de compilation, tests, performance et lancement portable figurent dans [VALIDATION](VALIDATION.md). Le [registre exhaustif](TASKS.md) conserve les critères encore partiels. Les cinq essais joueurs restent distincts de la validation technique. Aucun service distant, publication ou opération Git n'a été effectué.
