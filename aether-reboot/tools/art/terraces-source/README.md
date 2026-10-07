# Terrasses R56 — ajouts limités et reconstructibles

Le candidat conserve les GLB R54 exacts et ajoute seulement des massifs bas à
trois épaules asymétriques, leur charpente ligneuse, et quelques racines et
retombées fixées aux vraies limites du sol. Les deux atlases de feuillage et tous
les paramètres de matériaux sont ceux du monde existant. Aucun asset externe.

- Dawn : 20 arbustes dans 7 massifs + 4 jardins de corniche ; +4 880 triangles,
  199 256 triangles HD au total.
- Dawn Watch : 22 arbustes dans 9 massifs + 3 jardins de corniche ; +5 116
  triangles, 148 771 triangles HD au total.
- Seulement les primitives 06 (bois/racines), 08 et 09 (feuillage) s'allongent.
  Les anciens attributs, UV, normales, tangentes et coins des triangles sont
  conservés exactement. 12 meshes, 12 matériaux et 12 nodes par île.
- Sol, architecture R54, escalier, quais, points d'entrée, bassins, collisions,
  landmarks, routes et GLB LOD : aucune mutation. Les ajouts végétaux décoratifs
  ne créent pas de collision. Placement sur raycasts du sol, avec exclusion des
  surfaces pavées, chemins centraux, volumes physiques et bassins.

## Construire depuis les sources

Dans le paquet portable, les scripts se trouvent dans
`tools/art/terraces-source/` et les sorties dans `assets/world/`.

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --factory-startup --threads 4 --python tools/art/terraces-source/build_terraces.py
node tools/art/terraces-source/assemble_terraces.mjs
node tools/art/terraces-source/verify_terraces.mjs --self-test
```

Les baselines R54, leurs textures et l'implémentation contrôlée des helpers sont
embarquées dans `baseline/`, toutes vérifiées par `baseline-manifest.json`.
Les deux `.blend` contiennent la référence complète, les ajouts séparés avec
propriété `r56_role` et les images packées. L'assemblage Node ajoute les nouvelles
données dans les primitives existantes sans réexporter les objets historiques.
La gate indépendante vérifie l'intégrité, la préservation de ces données,
normales/tangentes, triangles non dégénérés/non inversés et budget de 200k.
Six corruptions de GLB sont rejetées. La reconstruction à partir de ce paquet
est comparée par SHA au candidat livré.

Pour les comparaisons illustratives CPU4 avec caméra/lumière identiques :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --factory-startup --threads 4 --python tools/art/terraces-source/review_terraces.py
```

Ces rendus Cycles ne remplacent pas une capture Bevy ni une mesure du coût MASK
dans le jeu. L'amélioration est locale : elle anime les surfaces libres sans
refaire la disposition de la ville ni remplir chaque carré de gazon. Les plus
petits feuillages ne changent pas les LOD ; ils disparaissent avec le HD.

## Intégration

Promouvoir seulement `dawn.glb`, `dawn-watch.glb`, les deux entrées HD +
`terraces_r56` de `manifest.json`, et le dossier de sources. Les autres entrées
du manifeste du projet ne doivent pas être remplacées par celles de la baseline
figée. `promotion-plan.json` fournit SHA et destinations. Les textures, LOD,
collisions et landmarks sont des dépendances exactes, pas des mutations.

La gate R54 qui exigeait la longueur exacte de tous les anciens batches de
feuillage doit désormais vérifier leur préfixe exact lorsque `terraces_r56` est
présent ; sa preuve architecture reste stricte sur les batches 03/04/05/07/10.
La gate R56 valide ensuite l'ensemble des appendices et leur provenance.
