# Création des décors

La compilation du jeu utilise les GLB fournis et ne nécessite pas Blender.

La voile contient trois shape keys (`WindPressure`, `RippleSin`, `RippleCos`) sur la toile et l'emblème. Elles sont exportées après application des chanfreins ; `export_apply=False` conserve les morph targets. Les limites de la toile annulent le déplacement aux attaches. `view::sail` anime les poids par instance selon le vent apparent fourni par `game::scenario`. Aucun vertex buffer CPU n'est reconstruit pendant l'animation. `node tools/art/verify.mjs` contrôle aussi ces cibles et l'absence de déformation du mât et du gréement.

Pour reconstruire les modèles après un changement du terrain :

```powershell
cargo run -p aether_core --example art_scene --locked --quiet | Set-Content -Encoding utf8NoBOM tools/art/scene.json
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_art.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 --python tools/art/build_sail_r52.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 --python tools/art/build_sail_rig.py
node tools/art/merge_sail_rig.mjs
node tools/art/verify.mjs
```

`scene.json` est un export du domaine : centres, dimensions et catégories des boîtes physiques. Le générateur ajoute strates, chanfreins, normales pondérées, petits brins d'herbe et feuillage. Les maillages sont groupés par matériau. Les sources éditables sont dans `source`, les résultats consommés dans `assets/art`.

La géométrie de l'Alcyon reste dans `core::fixtures`. Les ferrures de coque, les lanternes et les cordages sont recalculés par `view::craft` lorsqu'une construction change. Les coques identiques partagent leurs batches ; une modification ou une rotation de voile crée une géométrie distincte. Les entrées sans instance vivante sont libérées. Ces décors ne servent jamais aux calculs de masse ni aux collisions. La limite de 1024 cellules décorées borne leur coût sur la construction de 10k ; le maillage et toutes les cellules physiques restent présents.

Les panoramas ImageGen originaux, leur résolution réelle et leurs prompts sont documentés dans `PROMPTS.md`. Aucun traitement d'image n'est requis à la compilation. Les nuages de proximité sont rendus en volume dans `view::clouds`, avec une texture de densité calculée une seule fois.

La capture `node tools/art-capture.mjs` se lance depuis le dossier tools (`node art-capture.mjs`) avec le serveur de développement sur 4173. Elle pilote le jeu par les touches et la souris, place la pause et masque l'interface avec F8. La sonde est lue, jamais utilisée pour téléporter ou modifier la scène.

## Ciel du monde ouvert

Le panorama actif `sky-world-linear.ktx2` provient d'un rendu original 8192 × 4096, avec 98 champs continus, 4 013 placements et dix sculptures OpenVDB. Une seule source angulaire éclaire les nuages et l'air ; l'IBL est rendue depuis la même scène. Le générateur, Blender, les grilles, le HDR et la référence RGBA16F restent sous `tools/art`. Le fichier distribué utilise RGB9E5 : ses 100 663 296 composantes sont exactement égales à celles de la référence, pour 128 Mio GPU au lieu de 256. Aucun pixel n'est redimensionné ni retouché. Encodage .25, gain runtime 2.1435469. Le [guide du format runtime](sky-source/RUNTIME.md) précise les contrôles et la reconstruction.

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --verify-scene
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --rebuild-volumes
node tools/art/verify-sky.mjs --self-test
node tools/art/verify-sky.mjs
# Après le rendu --final, produire le format runtime puis les cartes d'éclairage :
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/pack_sky.py --source tools/art/sky-source/reference/sky-world-rgba16.ktx2 --output assets/textures/sky-world-linear.ktx2 --report tools/art/sky-source/runtime-packing.json
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/build_ibl.py
node tools/art/verify-sky-packing.mjs --self-test
# Après un changement auteur volontaire : --refresh, puis nouveau rendu --final.
```

Le rendu doit rester séparé des mesures de performance. La reconstruction des 98 grilles utilise seulement le CPU et vérifie leur identité binaire. `verify-sky` contrôle la référence originale et sa provenance ; `verify-sky-packing` compare séparément chaque composante du fichier distribué. Le soleil de la scène auteur et du jeu partage maintenant une direction basse, à −2,92°. L'ouverture est visible, mais sa fidélité dorée et le modelé des nuages locaux restent en revue.

## Préfiltrage de l'éclairage indirect

`build_ibl.py` prépare deux cubemaps à partir de l'original HDR d'éclairage du ciel : Lambert 32² pour le diffus et GGX 256² avec neuf niveaux de rugosité pour les reflets. Cette convolution produit des données d'éclairage dérivées ; elle ne modifie pas le panorama visible. Le filtre suit les conventions du [glTF IBL Sampler de Khronos](https://github.com/KhronosGroup/glTF-IBL-Sampler) et celles des shaders Bevy 0.19.1 : diffuse déjà divisée par π, rugosité linéaire entre niveaux et inversion Z du lookup cubique.

Le générateur exige un manifeste auteur : disque solaire absent des rayons caméra de l'IBL, mais source toujours active sur les nuages. L'intensité du soleil direct n'est donc pas comptée deux fois. Il applique l'exposition du manifeste, contrôle l'empreinte du HDR et conserve celle du décodeur et du générateur. Le réglage d'intensité du moteur reste une calibration artistique distincte des unités du rendu auteur.

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/build_ibl.py --self-test
# Le dossier choisi doit contenir tools/art/sky-source et son manifeste IBL.
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/build_ibl.py --root <dossier-candidat>
node tools/art/verify-ibl.mjs --root <dossier-candidat> --self-test
```

Les tests analytiques contrôlent les six orientations, la conservation d'un champ constant et l'intégrale d'un champ directionnel. Le vérificateur indépendant inspecte les valeurs fp16, les six faces, les niveaux GGX, les plages de fichiers et les empreintes ; sept corruptions ciblées doivent être refusées. Les deux textures demandent 4 243 440 octets GPU. L'IBL auteur R53 est active et contrôlée par `verify-open-world`, avec les textures critiques chargées avant l'entrée en partie.

## Monde ouvert

Les sources sont rangées dans `world-source`, `ship-source`, `fauna-source` et `traffic-source`. Les modèles fournis suffisent pour compiler et jouer ; Blender 5.2 est seulement un outil auteur. Depuis la racine du reboot :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_world.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_world_variants.py -- --publish
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --threads 4 --python tools/art/build_facades_r54.py
node tools/art/finish_facades_r54.mjs
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --factory-startup --threads 4 --python tools/art/terraces-source/build_terraces.py
node tools/art/terraces-source/assemble_terraces.mjs
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_vault.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_ship.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_fauna.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_trader.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/build_trader_collisions.py
node tools/art/verify.mjs
node tools/art/verify-open-world.mjs
```

`build_world.py -- dawn crystal` après le séparateur Blender `--` limite la reconstruction à ces deux biomes. Un changement du générateur commun exige aussi de régénérer les variantes et la voûte : leurs manifestes référencent son empreinte. Les scripts produisent des rendus de contrôle ; ils ne doivent pas tourner en même temps qu'un benchmark du jeu.

Les collisions et repères sont exportés dans `assets/world/*.json`, puis consommés par le domaine. Le LOD est rédigé séparément ; il conserve les volumes et surfaces praticables au lieu de décimer globalement des blocs disjoints. Les validateurs contrôlent les positions, les couloirs, les escaliers, les pivots, les textures et leurs empreintes. Le mouvement des cinq engrenages de la forge et des hélices du dirigeable provient des pivots des GLB. Les géométries ne sont pas déplacées pour masquer une erreur d'export.

Les embrasures R54 se reconstruisent après les îles de base ; leur [guide](architecture-source/README.md) décrit l'assemblage conservant exactement les batches hors architecture. Les [terrasses R56](terraces-source/README.md) s'assemblent ensuite depuis leur entrée R54 figée : elles ajoutent seulement bois et feuillage, sans réexporter les surfaces historiques. Si le terrain ou les façades changent, cette entrée doit être revue et reconstruite avant d'assembler les terrasses. Les courants texturés et leurs onze mipmaps ont leur [source et contrôle portable](flow-source/README.md). Le validateur global exécute ces familles. La [dunette R56](naval-source/README.md) est générée par le domaine Rust et la présentation depuis les vraies cellules éditables ; ses sources et preuves historiques sont archivées séparément.

La galerie du monde utilise le renderer de production mais prépare artificiellement la caméra et les habitants ; c'est une fixture artistique, distincte des parcours aux touches :

```powershell
cargo run -p aether_view --example world_gallery --locked -- --world dawn --capture .dream-loop/dawn.png
cargo run -p aether_view --example world_gallery --locked -- --world underforge --interior --capture .dream-loop/underforge.png
```
