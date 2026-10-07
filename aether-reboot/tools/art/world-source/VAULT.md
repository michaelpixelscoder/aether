# Voûte régionale Hollow / Underforge

`build_vault.py` construit `arch-vault.glb`, `arch-vault-lod.glb`, `vault-collisions.json` et `vault-manifest.json` indépendamment des îles. Le `.blend` contient uniquement le décor détaillé; les éclairages et caméras des aperçus sont ajoutés après l'enregistrement et ne font pas partie du GLB.

La région est centrée en `[0,0,0]`, en mètres Y-up. Les parois occupent l'anneau de 1 500 à 1 650 m, entre Y−600 et Y400. L'enveloppe intérieure du toit reste comprise entre Y350 et Y450. Le relief extérieur ajouté en r11 atteint Y650. Deux passages de 500 m restent ouverts vers Z positif et négatif. Deux puits, centrés sur X−700 et X700, Z0, conservent un diamètre libre d'au moins 400 m. Il n'y a aucun plancher.

Décision r11 : conserver la navigation et le dessous du toit, puis superposer sept massifs inégaux en strates, avec onze éperons périphériques décalés. Les volumes supérieurs partent de Y418 pour recouper la roche existante et ne laisser aucune plaque flottante. L'enveloppe totale mesure 1 250 m; les crêtes culminent à Y650. Le lichen sobre est porté par les faces des strates, sans grille de plaques superposées. Les stalactites périphériques ne pénètrent pas le rayon navigable de 1 400 m sous Y300. Les bords des puits restent évidés sur au moins 400 m de diamètre.

Les colliders sont des boîtes métriques dans un tableau JSON. Les boîtes du toit intérieur sont inscrites entre les surfaces sculptées; elles sont fusionnées lorsqu'elles partagent la même hauteur. Chaque cellule et éperon extérieur possède sa collision congruente. Le validateur compare les hauteurs de collision aux triangles réels sur 148 points. Les petits éclats des parois et le lichen restent décoratifs. Les parois ont une épaisseur minimale suffisante pour éviter les fentes diagonales créées par la discrétisation sur grille.

Les trois roches partagent `assets/textures/dressed-limestone-r11.png`. Leurs facteurs PBR linéaires sont écrits explicitement dans le GLB : roche `[.21,.23,.24]`, fracture `[.135,.15,.16]`, strate `[.29,.30,.29]`. Les deux lichens sont des matériaux unis. Budget final : 93 944 triangles et 5 matériaux en HD, 10 766 triangles et 5 matériaux en LOD, 3 318 colliders. Les aperçus proviennent du GLB réimporté, afin d'inclure le comportement réel de l'export.

Validation : `node .dream-loop/world-art-vault-verify.mjs`. Le script contrôle les accès dans les triangles GLB, l'absence de sol, la couverture du toit, la fermeture des directions de fond, les dépendances de texture et les SHAs inchangés des îles. Ce contrôle géométrique ne remplace pas le voyage et l'éclairage dans Bevy.

Aperçus : `.dream-loop/world-art-vault-high-interior.png`, `-high-under.png`, `-high-top.png`, `-lod-interior.png` et `-lod-top.png`. Les éclairages de contrôle sont réservés aux aperçus; le moteur fournit l'ambiance Hollow ou Underforge.
