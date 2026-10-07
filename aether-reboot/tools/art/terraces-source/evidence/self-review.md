# Auto-revue R56 B

Comparaison des mêmes caméras Cycles CPU4, avant/après ; aucun shader de jeu ni
éclairage changé pour cette étude.

La première étude A conservée dans `study-a/` donnait des massifs trop ovoïdes,
proches de blocs verts. B remplace chaque volume par trois épaules inégales et
des embranchements. Les pousses courbées utilisent l'atlas botanique original,
les coeurs réutilisent le feuillage dense existant ; pas de sphère ajoutée.

Sur Watch, les massifs s'insèrent entre les arbres et les murs et rompent mieux
les grandes surfaces plates. La marche centrale, les seuils, les chemins latéraux
et le portail restent lisibles et ouverts. Sur Dawn, l'effet est plus discret :
les exclusions physiques et les grandes surfaces pavées gardent beaucoup d'air
autour du portail et de l'escalier. Quelques nouveaux feuillages sont masqués par
les maisons ou les arbres depuis la caméra de détail ; ce n'est pas un remplissage
uniforme de tout le terrain.

Les masses de bâtiment, fenêtres R54, pierres, étagement et cadre des quais sont
visuellement inchangés ; ce constat est renforcé par le contrôle exact de tous
les anciens octets d'attributs et indices. Les ajouts aux corniches ont de vraies
racines mais sont petits à la distance de la vue globale. Le global Dawn coupe
encore légèrement les plus hautes tours : il est un contrôle de terrain, pas une
capture de composition finale contre la planche 33.

Limites : cette étude locale ne résout pas les grandes limites de l'éclairage,
de la silhouette du navire ou de la densité de tous les biomes. Elle ne suffit
pas à déclarer une hausse de score dream-loop. Une capture native Bevy et une
mesure de coût MASK restent nécessaires à la décision d'intégration.

Gate : 12 batches existants, 0 nouveau node/matériau ; tous les anciens coins et
attributs exacts ; AABB global exact, LOD/collisions/landmarks exacts ; aucune
nouvelle normale non finie, triangle inversé ou dégénéré ; 6 corruptions rejetées.
Reconstruction portable complète hors workspace : les deux GLB byte-exacts.
