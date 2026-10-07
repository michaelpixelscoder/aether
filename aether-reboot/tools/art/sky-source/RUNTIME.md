# Ciel R57 — stockage runtime exact

Le panorama est un rendu original 8192 × 4096. Le HDR original et la référence RGBA16F `reference/sky-world-rgba16.ktx2` restent dans les sources hors distribution. Le runtime utilise `assets/textures/sky-world-linear.ktx2` au format RGB9E5 : 134 217 728 octets de texels au lieu de 268 435 456. Le fichier complet fait 134 218 032 octets ; son SHA est `7efec756222aeaeffcbfe1591c28397d2f9ae97a503081a2b0548144eadb0b16`.

L'encodage .25 est conservé explicitement pour stabiliser le gain du shader à 2,1435469. Radiance HDR possède un exposant partagé et huit bits de mantisse par canal. Pour ce rendu, les neuf bits du runtime reproduisent exactement la référence fp16. `pack_sky.py` refuse un pixel perdu ou un alpha différent de un. La comparaison indépendante Node décode les deux formats : 100 663 296 composantes égales, erreur maximale zéro, huit corruptions refusées. Ce résultat ne garantit pas que tous les rendus futurs seront exactement packables.

L'IBL vient d'un original 1024 × 512 / 128 de la même scène. Seul le disque est omis aux rayons caméra ; le champ chaud partagé et l'illumination des nuages restent actifs. La convolution Lambert/GGX est un dérivé d'éclairage documenté, distinct d'un rendu original ou d'une retouche. Le diffuse est un cube 32², le spéculaire un cube 256² et neuf niveaux de rugosité ; sept corruptions sont refusées.

Depuis la racine du paquet, après `build_sky.py --final` et `--ibl` :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/pack_sky.py --self-test
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/pack_sky.py --source tools/art/sky-source/reference/sky-world-rgba16.ktx2 --output assets/textures/sky-world-linear.ktx2 --report tools/art/sky-source/runtime-packing.json
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/build_ibl.py
node tools/art/verify-sky.mjs
node tools/art/verify-sky-field.mjs --self-test
node tools/art/verify-sky-packing.mjs --self-test
node tools/art/verify-ibl.mjs --self-test
```

Le runtime répète U et borne V, avec `U=fract(.75-u), V=v`. La convolution conserve les conventions cubemap/WebGPU/Bevy existantes ; aucun décodeur personnalisé n'est ajouté au shader du ciel. Les gates gardent les seuils R55 de chromaticité, coutures et pôles. Ils vérifient un format et sa provenance, pas une note artistique ou une performance.

L'intensité IBL et la lumière clé doivent être calibrées dans la production après revue native de ce 8K. Les anciens résultats de mémoire/performance R55 et les captures diagnostiques 2K ne décrivent pas la qualité finale R57.

Références primaires : [Khronos EXT_texture_shared_exponent](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt), [DFD Khronos](https://github.com/KhronosGroup/KTX-Software/blob/main/external/dfdutils/createdfd.c), [capacités WebGPU](https://gpuweb.github.io/gpuweb/#texture-format-caps), [Bevy environment map](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/light_probe/environment_map.wgsl).
