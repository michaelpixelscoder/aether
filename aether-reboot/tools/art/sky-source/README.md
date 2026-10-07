# Ciel original R57 — source portable

Ce paquet isolé contient un panorama original 8192 × 4096 et son éclairage source original 1024 × 512, rendus à 128 échantillons par Cycles OPTIX. Le panorama a pris 367,19 s et l'IBL 11,41 s. Il est prêt pour une revue native ; aucune capture de la galerie ne constitue sa validation artistique finale.

Les 98 champs continus, les 4 013 placements, les dix sculptures OpenVDB et tous les matériaux de R53 sont conservés. Après réouverture du fichier portable, les matrices, sommets, polygones, octets VDB et graphes des cinq matériaux sont exactement identiques. Les 98 chemins actifs sont relatifs et résolus dans le paquet. La recette a reconstruit les 98 fichiers avec les mêmes empreintes en 51,017 s. L'air conserve g=.90 et la densité 3e−7. L'hypothèse A modifiant l'air a été rejetée à cause de son voile blanc massif ; elle reste conservée dans le dossier d'étude.

Le changement porte sur la radiance incidente du World : les pôles bleus et un disque stellaire nominal de 1,20°, puis un champ chaud RGB linéaire (7,2,.3) dans un cône de rayon 4,5°. Le champ utilise un poids smoothstep angulaire commun aux rayons caméra, nuages et éclairage ; hors cône, la source bleue B est exacte. Le transport et la convolution peuvent naturellement propager une différence hors cône. C'est une illumination d'auteur, pas une couronne stellaire mesurée ni une simulation complète de diffusion multiple atmosphérique.

Le seuil FLOAT effectivement enregistré donne un diamètre de 1,20018955°. L'intégrale normale rouge du cœur est 21,93409451 ; la différence du champ chaud par rapport à B ajoute .06590561. Leur somme vaut 22,00000012. La fraction du cœur est .99700429595. Le gate indépendant calcule cette somme depuis les vraies connexions et valeurs enregistrées, puis refuse 13 corruptions ciblées. Il vérifie aussi que le seul masque par type de rayon sert à enlever le disque des rayons caméra de l'IBL. Les nuages et le champ incident restent éclairés.

Le shader des nuages locaux change seulement deux valeurs de radiance pour rester cohérent avec la source. Le vérificateur annule ces deux modifications et retrouve exactement les octets du shader R55 archivé ; les densités, l'intégration, les silhouettes et l'alpha restent donc protégés.

L'encodage linéaire .25 est fixe, avec le multiplicateur runtime 2,1435469. Le rendu vérifie l'égalité de toutes les composantes HDR après conversion fp16, puis le packing RGB9E5 vérifie l'égalité de 100 663 296 composantes, erreur maximale zéro. Les gates de provenance, chromaticité R55, couture et pôles passent sans relâchement des seuils. Voir [RUNTIME.md](RUNTIME.md).

Les essais natifs diagnostiques ont utilisé env×.5 et key×1.25. Ils servent à choisir une calibration, pas de preuve finale. La calibration de production doit être décidée et inscrite dans Rust après revue des actifs 8K finaux. La fraction du cœur ci-dessus permet de distinguer le cœur directionnel du champ chaud conservé dans l'IBL.

Depuis la racine du paquet, avec Blender 5.2 :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background -t 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --verify-scene
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background -t 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --rebuild-volumes
node tools/art/verify-sky-field.mjs --self-test
node tools/art/verify-sky.mjs --self-test
node tools/art/verify-sky.mjs
```

Après une modification volontaire de la scène, `--refresh` marque la source comme nécessitant un nouveau rendu. Actualiser aussi ses paramètres d'auteur vérifiés. `--final` produit le panorama original, `--ibl` son éclairage source ; terminer avec le packing et la convolution du guide runtime. Une reconstruction VDB modifie la preuve et le manifeste : refaire alors `build_ibl.py` pour associer la convolution au nouveau SHA du manifeste.

Les originaux R53 sont préservés dans `history/r53-original-source.blend`, `history/r53-original-linear.hdr`, `history/r53-original-ibl-linear.hdr` et `reference/sky-r53-original-rgba16.ktx2`. Le fichier .blend historique est byte exact : pour retrouver ses chemins relatifs, le restaurer sous `sky-source/` à son emplacement d'origine. La scène R57 active s'ouvre directement depuis son emplacement portable. Les historiques précédents, les scripts réellement exécutés et les descripteurs des deux nouveaux rendus restent conservés.

Sources primaires : [PBRT, radiance incidente directionnelle des lumières infinies](https://pbr-book.org/4ed/Light_Sources/Infinite_Area_Lights), [Bevy 0.19.1, bloom et poids Karis](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_post_process/src/bloom/bloom.wgsl), [Bevy, exposition PBR](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr_functions.wgsl). Ces références justifient le modèle de source et le diagnostic ; elles ne constituent pas une preuve de fidélité visuelle. Aucune image externe, retouche, rotation corrective, nouvelle densité ou mise à l'échelle n'intervient.
