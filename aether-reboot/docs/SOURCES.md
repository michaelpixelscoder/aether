# Sources techniques utilisées

- [Factorio — Friday Facts #416, Fluids 2.0](https://www.factorio.com/blog/post/fff-416), relu le 3 octobre : segments regroupant stockage et tuyaux, réserve partagée, débits de consommateurs et pompes dirigées. Le laboratoire R67 adapte ce principe au réseau d'Aether avec conservation en unités entières et transactions de topologie ; aucun code ni performance de Factorio n'est repris ou extrapolé.

Consultations du 1er octobre 2026, complétées par les sources exactes des crates résolues dans le cache Cargo. Les décisions locales et mesures du reboot se trouvent dans `DECISIONS.md` et `VALIDATION.md` ; les documents ci-dessous expliquent les API et choix étudiés.

| Source primaire | Utilisation dans le reboot |
| --- | --- |
| [Avian 0.7 Collider](https://docs.rs/avian3d/0.7.0/avian3d/collision/collider/struct.Collider.html) | Colliders voxel, formes composées et propriétés. L'alignement réel avec le domaine est vérifié par test, pas supposé. |
| [Tnua 0.32 README](https://docs.rs/crate/bevy-tnua/0.32.0/source/README.md) | Compatibilité et contrôle de personnage sur supports mobiles. Le scheduling précis est recoupé avec la source de l'adaptateur 0.12. |
| [Bevy 0.19](https://docs.rs/bevy/0.19.0/bevy/) | États, assets, rendu et animation. Les exemples `animated_mesh` et les sources `bevy_input` ont guidé les changements d'API et les touches logiques. |
| [Bevy CLI web](https://thebevyflock.github.io/bevy_cli/cli/web.html) | Page `web/index.html`, génération des bindings et bundle des assets. |
| [Binaire officiel Bevy CLI](https://github.com/TheBevyFlock/bevy_cli/releases/tag/cli-v0.1.0-alpha.2) | Téléchargement HTTPS du binaire verrouillé ; empreinte dans `evidence/bevy-cli-sha256.json`. Aucun clone. |
| [MDN IndexedDB transaction](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/transaction) | Mode `strict` et confirmation au commit. La durabilité du navigateur ne protège pas contre l'effacement de ses données. |
| [Chrome WebGPU troubleshooting](https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips) | Diagnostic adapter/device et différence de capacités entre exécutables d'automatisation. |
| [Microsoft Visual C++ v14 Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) | Prérequis runtime x64 du package Windows, après observation de VCRUNTIME140.dll dans les imports PE. |
| [KayKit Adventure Characters](https://github.com/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0) | Provenance du Knight repris du projet, licence CC0 incluse. |
| [Noto Sans](https://github.com/google/fonts/tree/main/ofl/notosans) et [Cormorant Garamond](https://github.com/google/fonts/tree/main/ofl/cormorantgaramond) | Polices françaises, fichiers OFL inclus. |

Le greedy, la scission, les champs de courant et les codecs sont implémentés dans ce workspace. Le réglage à 60 Hz, les forces, le rayon de courant et les corrections du personnage sont des choix de projet soumis aux tests locaux. Aucun résultat de benchmark externe n'est présenté comme une performance mesurée ici.

Compléments consultés pendant la fermeture :

- [MikkTSpace — interface de référence](https://github.com/mmikk/MikkTSpace/blob/master/mikktspace.h) : soudure interne et orientation tangentielle ; convention de signe recoupée dans `bevy_mesh` 0.19.1 local. Le calcul spécialisé pour faces planes est comparé à cette implémentation, sans modifier le chemin des GLB.
- [Bevy — Morph Targets](https://bevy.org/examples-webgpu/animation/morph-targets/) et [Blender — glTF](https://docs.blender.org/manual/id/5.0/addons/import_export/scene_gltf2.html) : support des morph targets, poids par instance et export des shape keys. API exacte `MorphWeights` / `MeshMorphWeights::Reference` recoupée dans les sources Cargo 0.19.1. Les rafales et amplitudes sont des choix propres au jeu, soumis aux tests locaux.

- [Cycle de vie des pages Chrome](https://developer.chrome.com/docs/web-platform/page-lifecycle-api) : différence entre onglet visible, caché et gelé. Le test retient une suspension JS vérifiable.
- [Chrome DevTools Protocol — Debugger](https://chromedevtools.github.io/devtools-protocol/tot/Debugger/) : événements pause et reprise utilisés par le scénario de 65 secondes.
- [W3C — contraste minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) : luminance sRGB et seuil 4,5:1 pour les textes ; calcul reproductible dans `tools/contrast-check.mjs`.
- [Bevy — RenderDiagnosticsPlugin](https://docs.rs/bevy/0.19.1/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html) : mesures CPU/GPU par passe quand le backend expose les horodatages. Sources exactes recoupées dans le cache Cargo.
- [Blender — export glTF](https://docs.blender.org/manual/en/5.2/addons/scene_gltf2.html) : maillages, UV, Principled BSDF et textures exportables ; matériaux procéduraux non supposés transférables.
- [Bevy — AtmosphereEnvironmentMapLight](https://docs.rs/bevy/0.19.1/bevy/light/struct.AtmosphereEnvironmentMapLight.html) et [exemple Atmosphere](https://bevy.org/examples/3d-rendering/atmosphere/) : génération de l'éclairage ambiant/spéculaire depuis l'atmosphère ; résolution explicitement bornée à 64² par face dans le reboot.
- Sources locales Avian 0.7 et `bevy_transform_interpolation` 0.5 : activation par composant, position physique distincte de la pose interpolée, remise à zéro des états lors d'une téléportation.
- [Bevy — StandardMaterial](https://docs.rs/bevy/0.19.1/bevy/pbr/struct.StandardMaterial.html) : émission, poids de l'exposition et transparence. Le shader local `pbr.wgsl` confirme que la branche `unlit` retourne la couleur de base ; les filaments utilisent donc le chemin émissif PBR pour conserver leur halo.
- [Bevy — RelationshipTarget](https://docs.rs/bevy/0.19.1/bevy/ecs/relationship/trait.RelationshipTarget.html) : `LINKED_SPAWN` contrôle la suppression des entités sources. Recoupé dans le code Cargo résolu : `TnuaSensorsSet` 0.13 n'active pas cette option, `Children` Bevy l'active. Le correctif de durée de vie est limité aux capteurs de nos personnages et protégé par une régression exécutée avant/après.

- [Khronos — glTF 2.0, matériaux](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#materials) : une texture d'albédo est décodée de sRGB vers le linéaire puis multipliée par le facteur du matériau et par `COLOR_0` s'il est présent. L'absence de `baseColorFactor` ne prouve donc pas l'absence de teinte. Ce contrat guide l'étude de sol R60 ; les données du GLB chargé restent la preuve de l'export réel.
- [Bevy 0.19.1 — tonemapping partagé](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_core_pipeline/src/tonemapping/tonemapping_shared.wgsl) : l'étude R57 compare les radiances avant et après le tonemapping exact du moteur ; une hausse HDR peut saturer les couleurs au lieu de renforcer leur teinte. La revue dans Bevy reste nécessaire avec bloom, TAA et volumes locaux actifs.
- [Valve — Alex Vlachos, Water Flow, SIGGRAPH 2010](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf) : advection de faible amplitude, deux phases temporelles décalées d'une demi-période, bruit pour réduire les pulsations et intervalle de déformation des couleurs centré sur zéro. Ces principes guident l'étude R62 ; ses crêtes/alpha sont des choix de projet encore non promus, pas une simulation de fluide. Les résultats de gameplay et coûts de 2010 ne sont pas extrapolés au reboot.
- [NVIDIA — GPU Gems, Mark Finch, Effective Water Simulation from Physical Models](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-1-effective-water-simulation-physical-models) : séparation des ondulations géométriques et du détail de surface. Référence de méthode pour distinguer corps continu et petites crêtes ; aucune équivalence physique ni performance moderne n'est déduite de ce chapitre historique.

- [Red Blob Games — implémentation d’A*](https://www.redblobgames.com/pathfinding/a-star/implementation.html) : états pouvant inclure une direction et file de priorité. R72 inclut aussi une distance droite minimale et une reprise coopérative bornée ; grille, marges et plafonds sont nos décisions vérifiées localement.
- [Khronos — normalTexture glTF2](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#materialnormaltextureinfo) : normales tangentielles, coordonnées UV et facteur d’échelle. Le diagnostic R71 distingue contrat exporté, cartes effectivement chargées et détail réellement visible dans Bevy.
