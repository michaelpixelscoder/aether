# Candidat naval r44 — isolé, non intégré

La capture examinée est `../world-r43b-dawn.png`, comparée à la planche 33. Le bateau possède déjà des bordés éditables, corniches, rails et haubans dérivés des cellules. Les remplacer par une coque décorative monolithique détruirait ce contrat sans corriger le principal manque observé : la construction des mâts et vergues.

## Changement concret

Le candidat remplace uniquement les quatre meshes rigides de `assets/art/sail.glb`. Les vergues ont désormais des sections octogonales effilées, un léger cintre et des embouts. Le mât s'affine vers son mâtereau, avec un joint visible, un cap tourné, un croisillon soutenu par deux genoux courbes, des colliers et un racage. Les points de manœuvre portent de petites poulies à joues séparées, réas, axes et œils ; les saisines entourent réellement les vergues. Les balancines, ralingues et palans courts restent rattachés au gréement de cette même pièce.

Les haubans vers le pont restent générés par `craft.rs` à partir des cellules réelles. Le candidat n'ajoute aucun câble entre un GLB et une position de pont présumée. Les ferrures de coque `build_ship.py`, les garde-corps/corniches existants et les décisions de transmission lumineuse de la toile sont conservés.

## Contrats et validation

- Aucun fichier canonique modifié. Tout le prototype est dans ce dossier.
- Les cinq meshes animés sont repris du GLB canonique : mêmes noms, matériaux, poids par défaut, UV, indices, positions et normales ; tous leurs bufferViews et octets restent identiques. La toile conserve ses 1 600 carreaux et son emblème.
- `WindPressure`, `RippleSin`, `RippleCos` sont repris intégralement, sans réexport ni recalcul. Aucun morph ajouté aux espars.
- Les nouveaux volumes restent dans l'enveloppe XYZ du GLB d'origine. Les cellules, pièces, empreintes, masse et collisions restent exclusivement celles du code canonique inchangé.
- 9 meshes et 7 matériaux comme avant ; aucune nouvelle passe de matériau. Les textures originales sont réutilisées ; les copies d'images du travail Blender ne sont pas ajoutées au candidat.
- 71 996 → 78 744 triangles par voile, soit +6 748 (+9,37 %). Partie rigide : 7 988 triangles. Taille : 22 407 308 → 23 004 264 octets (+596 956).
- Normales exportées finies et unitaires à ±0,000001 ; zéro triangle dégénéré et zéro désaccord de sens entre normale et winding dans les nouveaux meshes. Tangentes exportées pour les quatre matériaux rigides.
- `validation.json` contient les SHA, les bornes, les coûts et les hashes des fichiers canoniques lus. `merge_verify.mjs` exécute ces assertions, y compris l'identité binaire des trois morphs.

## Revue visuelle CPU

Les GLB ont été réimportés dans Blender puis rendus avec Cycles CPU, quatre threads, 12 échantillons, 800 × 800. `canonical-cpu.png` et `candidate-cpu.png` utilisent les mêmes caméra, lumières et poids `[0.58, 0.27, 0.12]`. `candidate-masthead-cpu.png` montre les liaisons à plus grande échelle. Les facteurs de teinture suivent ceux du jeu ; la transmission diffuse Bevy n'est pas reproduite dans cette scène d'inspection.

Auto-revue : la silhouette du mât et les extrémités des vergues ont une construction navale plus crédible ; les pièces sont reliées, les poulies lisibles en gros plan, sans flottement ou cassure apparente. Le cube du mât disparaît. La toile reste volontairement identique et représente toujours l'essentiel de la surface à l'écran. À la taille du navire dans la galerie, les saisines et réas seront surtout des accents ; ce candidat seul ne rapproche pas suffisamment la composition générale de la planche 33 pour justifier une nouvelle note de monde. La brillance/lecture du tissu et le dessin des haubans sur la silhouette doivent être revus dans la galerie réelle avec SailWind. Aucun playtest d'édition ou profil FPS n'est revendiqué.

## Reproduction et revue parent

Depuis la racine du projet :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 --python .dream-loop/ship-r44-candidate/build_rig.py
node .dream-loop/ship-r44-candidate/merge_verify.mjs
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 --python .dream-loop/ship-r44-candidate/review_cpu.py
```

Le seul fichier à substituer dans une copie de runtime pour la revue est `assets/art/sail.glb` de ce dossier. Aucun patch Rust n'est nécessaire. Une intégration canonique éventuelle doit aussi intégrer une source reproductible et adapter la provenance du valideur d'art, qui attend actuellement un générateur unique `build_art.py` ; il ne faut pas simplement recopier ce GLB sous un hash de générateur ancien.
