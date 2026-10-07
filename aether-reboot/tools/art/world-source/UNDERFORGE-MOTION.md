# Engrenages visuels Underforge

Les cinq roues déjà modélisées sont extraites du lot statique de laiton dans `underforge.glb` et `underforge-lod.glb`. Elles conservent leurs triangles en coordonnées du monde, à 1 mm près après transformation des flottants glTF, ainsi que tous les colliders et repères de ressources. Aucun mécanisme de simulation n'est ajouté.

Chaque nœud possède une translation au pivot ci-dessous, une orientation initiale identité et l'extra `aether_animation_axis: "local Z"`. Le parent peut donc appliquer une rotation autour de Z local pour une animation purement visuelle. Les valeurs sont en mètres, Y vertical, dans le repère de l'île.

| Nœud | Pivot X, Y, Z |
| --- | --- |
| `UnderforgeGearWestForge` | −42, 19.25, 39 |
| `UnderforgeGearEastForge` | 40, 43.79, −37 |
| `UnderforgeGearEastWorkshop` | 43, 19, 37 |
| `UnderforgeGearWestDrive` | −47, 15, 0 |
| `UnderforgeGearEastDrive` | 47, 15, 0 |

Les cinq nœuds partagent le matériau `05 | Brushed antique brass`. L'extraction ajoute cinq meshes/draws mais aucun matériau ni triangle. Les supports, conduits, chaînes et cheminées restent statiques. La direction et la vitesse d'animation sont choisies côté présentation; aucune vitesse de simulation n'est inscrite dans l'asset.

Preuve : `node .dream-loop/world-r11-motion-verify.mjs` compare les triangles transformés et les matériaux avant/après, les deux niveaux de détail, les pivots, l'axe, tous les colliders et tous les landmarks. Résultat dans `.dream-loop/world-r11-motion-validation.json`.
