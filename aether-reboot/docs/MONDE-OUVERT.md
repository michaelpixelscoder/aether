# Monde ouvert — périmètre 0.2

La demande de Damien du 2 octobre active un monde plus vaste, beaucoup plus proche des planches et plus facile à parcourir. Les 41 images historiques ont été relues. Les planches 25–30 définissent les biomes extérieurs, 33–36 la composition du ciel habité et 40–41 les profondeurs. Les choix techniques restent en Rust / Bevy / Avian.

## Géographie

Le catalogue contient 204 nouvelles îles et les trois quais historiques. Il s'étend sur plusieurs kilomètres horizontalement et plusieurs étages, avec une altitude de pilotage comprise entre −3000 et +2200 m. Les îles sont déterministes et leurs identifiants sont sauvegardables. Ce n'est ni une planète sphérique ni un monde infini.

| Région | Centre, mètres X / Y / Z | Décor et identité | Marchandise locale |
| --- | --- | --- | --- |
| Couronne de l'Aube | −260 / 45 / −680 | Cité à dômes, tours, jardins, moulin, terrasses et cascades | Bois |
| Récifs de Cristal | 1500 / 480 / −1750 | Grands cristaux, sanctuaires et roche minérale | Cristal |
| Routes du Soleil | −1800 / 160 / −1400 | Architecture chaude et escales marchandes | Bois |
| Forges des Braises | −2600 / −120 / −3400 | Ateliers, fours, fumées et cendres | Fer |
| Monastères du Givre | 0 / 1250 / −3650 | Monastères en altitude, neige et glace | Eau |
| Racines Verdoyantes | 2600 / 100 / −3400 | Canopées, racines, végétation et ruines | Plantes |
| Frontière des Tempêtes | 1300 / 740 / −5000 | Tours, orages et courant ascendant en spirale | Cristal |
| Profondeurs Oubliées | −600 / −1400 / −1700 | Caverne, ruines, cristaux, gardiens et créatures ailées | Reliques |
| Cité des Sous-forges | 1350 / −2200 / −3200 | Caverne industrielle, ateliers et grands engrenages | Fer |

Les capitales et les satellites possèdent un quai. Les routes se prolongent entre les régions, et deux puits dans les voûtes permettent une descente réelle. Les grottes ne sont pas des scènes séparées chargées après un écran noir.

## Interactions effectivement implémentées

| Système | Action du joueur et effet | Vérification |
| --- | --- | --- |
| Propulsion | Avancer même sans vent, reculer lentement avec K, freiner progressivement ; la masse et l'inertie restent physiques | Tests de navigation : calme, vent debout/travers, bonus du vent, charge, frein et différentes fréquences |
| Vent et voiles | Voiles automatiques à l'avance, déploiement manuel, rafales et déformation du tissu selon le vent apparent | Tests de forces et mesure des morph targets avec pause/reprise |
| Courants | Quinze grandes routes et deux courants historiques ; accélération, spirales et descentes, sorties possibles sous moteur | Couloirs dégagés, quais calmes, trace aux touches dans le monde |
| Carte | M ouvre l'atlas, choix d'une destination, distance et régions consultables | Parcours navigateur avec sélection et reprise |
| Découverte | L'arrivée au quai ajoute le lieu aux découvertes et accorde une récompense unique | État persistant, validation et absence de double récompense |
| Collecte | B près d'un cristal ou bassin visible ajoute cinq unités, dans la limite de la cale | Portée, visibilité, identifiant de ressource et test de récolte en jeu |
| Commerce | Acheter/vendre une unité au port ; prix liés à la région, limite de cargaison et crédits contrôlés | Tests transactionnels et parcours navigateur |
| Contrats | Livrer huit unités demandées à une capitale, une fois par contrat | Débit exact, récompense unique et sauvegarde |
| Raffinage | Consommer un cristal pour restaurer 240 unités de réserve | Capacité, quantité et atomicité des refus |
| Cargaison | Chaque marchandise ajoute son poids et son inertie au navire | Propriétés de masse et ralentissement sous charge |
| Portails | Rejoindre un lieu déjà découvert depuis une porte proche, contre 80 unités d'énergie et avec un délai de 15 secondes | Destination valide, collision d'arrivée, cooldown et reprise |
| Anciennes constructions | Installer deux hélices depuis un port sur une ancienne sauvegarde, contre matériaux | Migration réelle du format 1 vers 2 et conservation après rechargement |
| Créatures | Gardiens et créatures ailées animés, observables avec B à portée et en vue ; récompense une fois par espèce | Trajectoires, pause/reprise, volumes et parcours souterrain |
| Trafic | Cinq dirigeables et trois navires à voile sur six circuits, mouvements physiques et reprise avec l'horloge sauvegardée | Trajectoires continues et enveloppes dégagées ; 90 boîtes par dirigeable, cellules et équipements réels pour les voiliers |
| Météo | Vent régional, jour/nuit, pluie, neige, cendres, brume et orage | Échantillons déterministes, horloge suspendue en pause |
| Machines | Moulin et cinq engrenages animés autour des pivots exportés | Pivots des deux LOD vérifiés, horloge partagée |
| Marche et caméra | Marcher sur le pont, la dunette et les îles ; caméra repoussée par les murs | Escaliers/paliers, montée et descente des deux accès de dunette par intentions normales, volumes partagés et balayage de caméra |
| Construction | Grand gréement et moteurs constructibles ; suppression, masse, scission et sauvegarde cohérentes | Invariants du domaine et QA de l'éditeur |

Les fichiers de preuve retenus figurent dans [VALIDATION](VALIDATION.md). Le script de QA court prépare certaines poses artificiellement ; le parcours de collecte et de caverne utilise les touches et une sonde en lecture seule. Ces preuves ne remplacent pas un essai humain de la sensation de pilotage.

Pour reproduire le parcours complet depuis la racine, servir le bundle avec `tools/serve.mjs`, puis définir `AETHER_TEST_URL` sur son adresse et `AETHER_WASM` sur son fichier WASM. `tools/verify.ps1` exporte le catalogue nécessaire ; `node tools/world-tour.mjs` pilote alors une vraie session et écrit `docs/evidence/world-tour.json` avec la trace, les erreurs et l'empreinte du binaire. L'option `--harvest-only` est un contrôle court et ne vaut pas validation des cavernes.

## Lecture précise des planches

Les dômes, corniches, escaliers, ponts, bassins, cascades, cristaux, voûtes et navires sont des objets rendus par le jeu. Le panorama ne contient aucun bâtiment ou navire peint. Les modèles détaillés et leurs LOD partagent des repères avec la collision ; aucun décor navigable n'est validé sur la seule présence d'un fichier GLB.

Certains thèmes des illustrations correspondent à une représentation ou à une interaction bornée, et non à un simulateur complet :

- L'eau coule visuellement et se récolte ; elle ne remplit pas dynamiquement les volumes et ne pousse pas les navires.
- La distribution d'Aether est installable au quai : ports réels, branches, vannes, débits bornés, fuites et réparation de réservoir, sauvegarde V3. Une branche vide coupe réellement ses moteurs/sustentateurs. Les conduites 3D routées, pompes jouables, dommages et transferts entre vaisseaux restent ouverts ; le modèle n'est pas un solveur volumique.
- Les plantes, reliques et lingots participent à l'économie ; il n'y a pas de faim, santé, arbre de recettes ou agriculture.
- Les créatures se déplacent et s'observent ; aucun combat, dialogue ou recrutement n'est livré.
- Les navires marchands circulent ; les échanges utilisent les interfaces des ports.
- Le moulin et la forge s'animent ; leurs engrenages ne transmettent pas un couple à une machine constructible.
- La coque peut être éditée et scindée ; les îles ne sont pas destructibles.
- La toile se déforme par trois morph targets ; ni tissu XPBD ni corde s'enroulant sur les obstacles.
- Le monde est étagé et borné ; les représentations planétaires des planches 37–39 ne sont pas une planète parcourable.

Ces points restent explicitement ouverts. Ils ne sont pas marqués terminés dans le registre initial par simple rapprochement avec les illustrations. La comparaison de qualité visuelle et ses écarts figurent dans [RENDU](RENDU.md).
