# Direction artistique et contrôle du rendu

Les 41 planches historiques guident le monde ouvert : cèdre brun, laiton, voiles indigo brodées, architectures en terrasses, cristaux, cascades et courants lumineux. La planche 33 sert à comparer la Couronne de l'Aube, la 25 les récifs et les 40–41 les cavernes. Les bâtiments, navires et reliefs sont des géométries du jeu ; le panorama ne contient aucun objet jouable peint.

## Réalisation 0.2

- Onze GLB du socle et 34 nouveaux GLB : douze types d'îles HD/LOD, voûte HD/LOD, quatre modules navals, deux espèces et un dirigeable HD/LOD. Sources Blender, générateurs et manifestes sont conservés. Le Knight historique reste l'avatar CC0.
- La coque de la nouvelle expédition contient 526 cellules éditables et quatorze équipements, dont deux moteurs et deux voiles de tailles différentes. Une dunette basse remplace la haute cabine ; ses deux accès et ses garde-corps suivent les cellules physiques. Les sauvegardes existantes conservent leurs constructions. Bordages, ferrures, fenêtres et lanternes suivent les cellules présentes. Les six lanternes au maximum éclairent réellement le pont.
- Trois morph targets animent la toile et son emblème selon le vent apparent. Mâts et points d'attache restent fixes ; pause et réglage de mouvement réduit sont respectés. Aucun vertex buffer CPU n'est refait à chaque image.
- Dômes, tours, moulin, terrasses, ponts, escaliers, racines, bassins et minéraux proviennent de modèles Blender. La collision et les repères de ressources sont exportés avec eux. Les modèles distants sont rédigés séparément pour conserver les volumes fermés et les paliers.
- Matériaux PBR, textures de cèdre/toile/pierre, normales du bois, rugosités distinctes, éclairage d'environnement, ombres, SSAO, TAA et bloom. Les mipmaps sRGB sont calculées en lumière linéaire. Les couleurs de pierre sont écrites explicitement dans les exports glTF.
- Rubans de courant courbes et cascades animées, nuages de proximité volumétriques avec ombres propres, précipitations et transitions de lumière. La salle souterraine garde son ambiance jusqu'aux ouvertures ; les lanternes et cristaux éclairent ses chemins.

Provenance : [ASSETS](ASSETS.md). Instructions auteur et prompts : [tools/art/README](../tools/art/README.md), [PROMPTS](../tools/art/PROMPTS.md). Contrats d'export : [socle](evidence/art-assets.json), [monde étendu](evidence/open-world-assets.json).

## Revue de l'extension en cours

**État courant R72 :** courants R66 en production avec leur carte dédiée ; [capture actuelle](../.dream-loop/house-r71-control-wide.png), sans override. Une maison R71 a été reconstruite entièrement dans un stage isolé : [comparaison proche](../.dream-loop/house-r71-candidate-watch.png). Les murs continus améliorent sa lecture, mais le toit reste trop lisse ; elle n’est pas promue. Les cartes chargées correspondent aux originaux, le diagnostic examine donc la sculpture et ses échelles de détail. Aucun nouveau score ne remplace R47 4/10. Les captures et études suivantes restent historiques.

La [capture R60 intégrée](../.dream-loop/world-r60-final-wide.png) réunit le ciel original 8K R57, les courants historiques cyan/violet, la voile creuse, les fenêtres encastrées, la dunette basse et les nouvelles falaises R59. Le [sol et les terrasses Watch à proximité](../.dream-loop/world-r60-final-watch-close.png) montrent le nouveau matériau de terre et les massifs R56. Les passages, collisions et repères restent exacts. La calibration active renforce la clé de surface de 25% et réduit le remplissage IBL de moitié, à exposition constante ; elle revient aux valeurs antérieures dans les cavernes. Les [Profondeurs de production](../.dream-loop/world-r57-production-hollow.png) et [Sous-forges](../.dream-loop/world-r57-production-underforge.png) sont recapturées sans sliders. Les variantes rejetées du ciel restent archivées. Les courants R58–R62 restent des études : leurs crêtes doivent encore former des chemins lumineux cohérents. Aucune nouvelle note indépendante n'est attribuée. Le ciel conserve toutes les valeurs originales et occupe 128 Mio GPU. Les captures historiques ci-dessous décrivent leurs versions respectives.

Les revues [r34, 2,5/10](../.dream-loop/art-judge-r34.md), [r39, 2,8/10](../.dream-loop/art-judge-r39.md), [r41, 3/10](../.dream-loop/art-judge-r41.md), puis [r47, 4/10](../.dream-loop/art-judge-r47.md), évaluent l'extension 0.2. Le barème plafonne la note à 3 tant que les masses principales ne correspondent pas à la planche. En r47, la réduction de la capitale nomade lève ce dernier blocage : le juge valide la disposition générale et conserve les autres positions acquises. La lumière dorée, la séparation ciel bleu/ombres et l'éclat des courants empêchent encore de franchir le palier suivant. Les surfaces et la finition navale restent également ouvertes. Cet état n'est pas accepté comme qualité définitive.

Les [captures intégrées r43](../.dream-loop/world-r43b-dawn.png) comprennent la capitale nomade réduite dans la géographie réelle, la roche et la maçonnerie PBR, les cascades en filets irréguliers, le tourbillon incliné et une toile éclairée par transmission diffuse. Elles utilisaient un original HDR linéaire 4096×2048, depuis remplacé par le 8K décrit plus haut. Les anciens rectangles pâles provenant des toits des cavernes n'y étaient plus visibles entre les nuages bas. Les [Profondeurs](../.dream-loop/world-r43b-hollow.png) et les [Sous-forges](../.dream-loop/world-r43b-underforge.png) possèdent des arcades et des matériaux lisibles à proximité. Aucune de ces captures n'a encore reçu une nouvelle note indépendante. Les succès des tests ne valent pas validation visuelle.

Les [revues 2/10](../.dream-loop/judge-1.md) et [2,5/10](../.dream-loop/judge-2.md) portent sur l'ancienne version 0.1 et son navire de 151 cellules. Elles restent archivées ; elles ne sont ni la note ni l'approbation de l'extension 0.2. Aucune note finale n'est annoncée avant la nouvelle comparaison.

La révision r44 intègre des rameaux feuillus texturés et des espars effilés. Les vues de contrôle de la ruine à [180 m](../.dream-loop/foliage-r44-focus-180.png), [800 m](../.dream-loop/foliage-r44-focus-800.png) et [1 480 m](../.dream-loop/foliage-r44-focus-1480.png) couvrent la minification et la transition HD/LOD ; la caméra vise explicitement les terrasses et omet le bateau de présentation. La couverture du masque est mesurée sur les textures réellement chargées. Ces vues montrent un progrès du feuillage, mais aussi des cristaux trop uniformes, des façades trop semblables et un ciel encore éloigné de la planche. Elles ne constituent pas une validation artistique finale.

La révision r47 réunit les [falaises, cristaux et nuages](../.dream-loop/world-r47-combined-dawn.png), avec une [vue proche de la capitale](../.dream-loop/world-r47-combined-close.png). Les [Profondeurs](../.dream-loop/world-r47-combined-hollow.png) et [Sous-forges](../.dream-loop/world-r47-combined-underforge.png) ont également été recapturées. Les fichiers de preuve associés identifient le binaire, tous les actifs et les paramètres de galerie. L'[auto-revue r47](../.dream-loop/self-review-r47.md) et la revue indépendante maintiennent les défauts de lumière, de densité végétale et de répétition architecturale.

## Reproduction

Le voyage de collecte et de caverne se reproduit avec `node tools/world-tour.mjs` depuis la racine, `AETHER_TEST_URL` pointant sur le jeu servi et `AETHER_WASM` sur son binaire. Toutes ses actions passent par les touches ; la sonde ne permet pas de modifier une pose. Ses captures sont des preuves de jeu effectif.

La galerie artistique utilise le même renderer mais prépare explicitement sa caméra, son heure et les poses de la faune. Elle sert à comparer des angles identiques et ne prouve pas un trajet de navigation :

```powershell
cargo run -p aether_view --example world_gallery --locked -- --world dawn --capture .dream-loop/dawn.png
cargo run -p aether_view --example world_gallery --locked -- --world hollow --interior --capture .dream-loop/hollow-interior.png
```

F8 masque l'interface dans le jeu. Les performances et leurs limites figurent dans [VALIDATION](VALIDATION.md), le périmètre fonctionnel exact dans [MONDE-OUVERT](MONDE-OUVERT.md).
