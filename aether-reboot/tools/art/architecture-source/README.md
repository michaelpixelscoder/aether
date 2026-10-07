# R54 — embrasures de maçonnerie Dawn / Watch

Un seul changement de structure visuelle : les façades pleines des maisons reçoivent de vraies baies creusées dans leur volume, avec joues de pierre, arc à grands voussoirs et vitrage au fond. La profondeur est comprise entre 0,945 et 1,2 m selon la maison. Les maisons gardent exactement leurs emprises, fondations, toits, corniches, coins et positions. L'ancien bandeau horizontal traverse toujours les baies et les partage visuellement en deux niveaux.

Le but est de produire de vraies ombres et un relief lisible en s'approchant, au lieu du simple rectangle posé sur un mur. Ce passage ne densifie pas les quartiers, n'ajoute aucun bâtiment et ne prétend pas résoudre les pelouses vides ou les tours trop semblables.

| Île | Maisons | Triangles avant | Après | Écart | Meshes / matériaux / nodes |
|---|---:|---:|---:|---:|---|
| Dawn | 18 | 184 368 | 194 376 | +10 008 | 12 / 12 / 12, inchangés |
| Watch | 7 | 139 763 | 143 655 | +3 892 | 12 / 12 / 12, inchangés |

Pas de nouvelle entité ni de nouveau batch matériel. Les LOD restent byte-exact. Les sept batches non architecturaux — roche, strates, bois/racines, les deux feuillages, cristaux et sol — sont repris directement des GLB d'origine, sans réexport de leurs attributs. Les propriétés des douze matériaux et leurs images sont inchangées.

Les volumes physiques sont inchangés. Chaque baie commence au-dessus de 2,5 m et reste fermée par son fond : elle ne représente pas une porte ou un accès jouable. La découpe du mur est exclusivement visuelle et rentre dans la masse existante. Les collisions, landmarks et chemins Watch sont comparés aux entrées archivées ; le volume global XYZ et les pivots sont également contrôlés.

## Reconstruire le candidat

Blender 5.2.1 et Node. Depuis cette racine :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python-exit-code 1 --threads 4 --python tools/art/build_facades_r54.py
node tools/art/finish_facades_r54.mjs
node tools/art/verify_facades_r54.mjs
```

`build_world.py` et `build_world_variants.py` restent inchangés. Le wrapper installe temporairement la recette de façade puis ne compose que les deux îles en HD. L'assemblage reprend les sept batches intacts des GLB archivés. Il refuse un double assemblage. Les entrées historiques sont dans `tools/art/architecture-source/archive` ; la provenance du patch et de l'assemblage est inscrite dans chacun des GLB.

Les `.blend` de Dawn et Watch contiennent les murs éditables et les images empaquetées. Leurs batches hors architecture peuvent avoir les minuscules différences de tangente d'un réexport Blender ; le GLB final reprend explicitement les données historiques exactes. Ne pas exporter le `.blend` seul en prétendant avoir reproduit l'assemblage final.

## Validation et limites

Les preuves durables se trouvent sous `tools/art/architecture-source/evidence/` ; les exemples de noms ci-dessous sont relatifs à ce dossier. Les scripts exécutables écrivent eux aussi dans ce dossier, y compris après promotion.

- `facades-build.json` : comptages, dimensions de chaque baie, collisions/landmarks contrôlés pendant la génération.
- `facades-assembly.json` : empreintes finales et sept batches préservés.
- `facades-gate.json` : normales/tangentes, triangles non dégénérés/non inversés, géométrie intacte hors architecture, extents/pivots, matériaux, images et LOD.
- `portable-rebuild.json` : résultats de la reconstruction dans un autre dossier, et identité des buffers entre l'étude visuelle et le paquet final.
- `negative-gate.json` : rejet d'une source modifiée, d'une archive altérée et d'un sommet hors emprise, même si son SHA de fichier a été actualisé.
- `watch-close-*` et `dawn-quarter-*` : mêmes caméras et éclairage CPU après réimport des GLB. Ces rendus étudient la géométrie ; ils ne prouvent ni le rendu natif ni la performance.

Une comparaison native avant/après reste nécessaire avant promotion. Le paquet et tous ses changements restent isolés. Aucun fichier canonique ni aucune donnée physique n'ont été modifiés par cette délégation ; aucune action Git.
