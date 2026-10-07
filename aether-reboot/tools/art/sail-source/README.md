# Voile voxel et gréement — source R52

Blender 5.2.1, Node, aucune dépendance téléchargée. Le tissu conserve les 1 600 voxels, l'emblème et les images de R44. R52 déforme l'archive Blender sans recréer les polygones ni les UV : bords latéraux concaves, pied légèrement relevé et ventre de repos approfondi de −0,24 à −0,45 m. La pression reste orientée vers +Z et garde ses trois morphs existants.

Reconstruction depuis la racine du projet :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python-exit-code 1 --threads 4 --python tools/art/build_sail_r52.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python-exit-code 1 --threads 4 --python tools/art/build_sail_rig.py
node tools/art/merge_sail_rig.mjs
node tools/art/verify.mjs
# Contrôle indépendant des polygones/UV dans les sources Blender :
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python-exit-code 1 --threads 4 --python tools/art/compare_sail_r52_source.py
```

Si la bibliothèque entière est reconstruite avec `build_art.py`, exécuter ensuite ces trois étapes de voile. Le générateur commun est volontairement inchangé afin de ne pas invalider la provenance des autres modèles.

- `archive/r44-canvas.blend` : source de départ exacte, SHA-256 vérifié avant toute déformation.
- `archive/r44-sail.glb` : référence exportée exacte de comparaison ; le vérificateur refuse une autre empreinte. `verify.mjs` exécute aussi les douze états extrêmes via `verify_sail_r52.mjs` et produit `evidence/r52/current-gate.json`.
- `../source/sail.blend` : tissu déformé, 5 surfaces animées, 3 morphs par surface, images empaquetées.
- `sail-rig.blend` : 4 surfaces statiques avec cordes de bord courbes et ferrures repositionnées ; mâts/vergues inchangés.
- `static-manifest.json` et `evidence/sail-provenance.json` : empreintes de toute la chaîne exécutée, archive comprise ; preuve que l'assemblage conserve intégralement le nouveau tissu.
- `evidence/r52/source-uv-topology.json` : polygones et UV des boucles identiques à l'archive.
- `evidence/r52/r52-gate.json` : mêmes textures/matériaux, 12 états de vent extrêmes sans triangle inversé ou dégénéré, sens de pression conservé, limites de déformation et dégagement du pont.
- `evidence/r52/portable-rebuild.json` : reconstruction dans un autre dossier ; GLB final byte-exact.

Le résultat contient 9 meshes, 7 matériaux et 79 084 triangles (+340 dans les cordes par rapport à R44). Les images CPU reimportées du paquet de travail sont des études de forme ; les captures moteur et les mesures de performances sont consignées séparément par l'intégration.
