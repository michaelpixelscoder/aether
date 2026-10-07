# R59 — étude de substitution des falaises

Ce paquet R59 est autonome et sa livraison demeure isolée. Il contient les sources, helpers et inputs R56 figés, les GLB reconstruits, les Blends éditables packed, les contrôles CPU et les prototypes historiques. Il ne modifie aucune source Rust, aucun collider et aucun asset externe. Il n'est pas une preuve de qualité finale du jeu. Les images Cycles CPU4 servent au contrôle de structure ; une capture Bevy comparable reste obligatoire avant toute promotion.

## Diagnostic

Le R46 comporte déjà un substrat et des plaques nommées « continuous », mais les groupes restent découpés par plans de cellule et par fenêtres d'altitude. Les 539/688 petits couronnements, totalisant17 248/22 016 triangles, forment une seconde couche de pierres dont la lecture reproduit des cours horizontaux. Ajouter davantage de bruit ou de petits cailloux ne traite pas la silhouette des grandes parois.

Le concept et les directives R47 demandent des groupes de plaques d'environ20–60px et des fissures de2–6px. Cette étude remplace l'émetteur décoratif01 : champ commun de grandes fractures obliques/verticales, facettes larges inclinées avec une vraie épaule géométrique, joints rentrants de0,58–1,18m. Les plaques atteignent31,04m Dawn et29,45m Watch ; les surfaces des terrasses restent aux mêmes altitudes. Le calepinage ne suit pas des rangées horizontales fixes.

Les trois véritables affleurements fermés par île qui définissaient les extrema X/Z sont conservés complets. Aucun sommet factice ne sert à maintenir un bound. Les autres petits couronnements sont remplacés.

Le retrait65cm de l'ancien substrat descendait sur toute la profondeur des cellules hautes bordant une terrasse basse. Le prototype ne recule que la portion réellement exposée, en conservant un substrat connecté en dessous. Dawn reste une composante principale. Watch conserve ses deux composantes d'origine : massif principal et fondation détachée de neuf cellules au quai. Aucun pont rocheux supplémentaire n'est inventé.

## Chiffres CPU vérifiés

| Île | R56 total | R56 roche01 | Prototype C total | Prototype C roche01 |
|---|---:|---:|---:|---:|
| Dawn |199256|39836|181520|22100|
| Watch |148771|48836|124521|24586|

Les11 autres batches, notamment toute architecture R54, les ajouts végétaux R56, le sol12, les strates physiques02, les escaliers et le bois des quais, sont repris avec chaque attribut et chaque index byte exact. Les252/180 triangles01 des racines physiques sont conservés avec tous attributs de corner, via signatures orientées de triangles ; la triangulation Blender peut intercaler des triangles et ne doit pas être auditée par un simple préfixe d'indices supposé.

- Les12 nodes, meshes, matériaux, textures et dépendances sont conservés.
- Bounds globaux exacts, collisions, landmarks et LOD inchangés.
- La régénération du01 actuel avant substitution reproduit exactement positions, normales, tangentes, UV et indices.
- Aucune face dégénérée ou inversée ; erreur des normales<1,4e−7, tangentes<6,9e−5 et UV métriques<2,4e−7.
- Toutes plaques sont des volumes fermés, avec chaque arête dans exactement deux faces opposées. Le vérificateur indépendant retrouve chaque face des281/386 noyaux et1252/1351 plaques dans les véritables streams de triangles du GLB. Il recalcule la connectivité des noyaux et vérifie leurs249/318 cellules d'origine contre les colliders réels ; il ne déduit pas la topologie du seul metadata. Seule la correspondance double→fp32 des sommets utilise une tolérance de30micromètres, alors que tous payloads d'attributs sont parallèlement exigés byte exact.
- Les12 corruptions sémantiques du vérificateur indépendant sont rejetées au contrat visé, après recalcul du checksum : sol/architecture, NaN, tangentes nulles, bounds et taille du GLB.

## Reproduire

Depuis la racine du paquet autonome, ou du workspace après promotion par le responsable, sans Git :

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/cliff-source/build_cliff.py
node tools/art/cliff-source/assemble.mjs
node tools/art/cliff-source/verify.mjs --self-test
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python tools/art/cliff-source/review_cpu.py
```

L'assemblage écrit seulement les deux GLB et les entrées correspondantes du manifest dans `assets/world`, relativement à ce paquet. Il ne recherche pas un workspace externe. `baseline-inventory.json` fige les40 inputs. Les quinze PNG réellement utilisés sont conservés, sans les sept images sources inutiles. Les sources de roche ne packent que leurs trois images PBR ; les sauvegardes automatiques `.blend1` ne sont pas livrées.

`build-evidence.json`, `assembly-evidence.json` et `gate-evidence.json` décrivent les preuves et les SHAs actuels. `source/*candidate.blend` contient les falaises éditables avec leurs trois textures packed ; `source/*candidate-review-full.blend` contient l'île complète du contrôle CPU historique C, avec ses quinze textures packed. `portable-rebuild-evidence.json` prouve que ses matériaux, nodes, images et tous streams de géométrie sont exacts au paquet actuel ; seul le metadata ultérieur R59 de source/assemblage/topologie a changé. Les fichiers historiques C, images originales et preuves ne sont pas réécrits.

Une reconstruction autonome a été exécutée dans `C:/Users/Damien/AppData/Local/Temp/aether-cliff-r59-rebuild-20261003`, hors workspace : les deux GLB complets obtenus sont identiques byte exact au paquet. Les quatre Blends candidats, complets et roche seule, ont été rouverts avec toutes leurs images packed. Les logs/proofs sont sous `evidence/` et `source/reopen-evidence.json`.

Les prototypes A/B sont archivés sous `archive/`, avec leurs scripts/GLB/échecs. A avait changé les bounds et produit quelques tangentes nulles : ces échecs ne sont pas remplacés par une preuve de succès. B avait corrigé ces deux problèmes, mais pas encore reconnecté la portion profonde des cellules de terrasses différentes.

## Gates incompatibles à faire évoluer avant promotion

Aucune gate canonique n'a été modifiée. Des overlays de compatibilité R54/R56 et du vérificateur global sont proposés séparément, dans le dossier voisin `overlays/`. Les scripts originaux restent figés sous `archive/gates-before-r59/`. Les deux overlays ont passé leurs contrôles hors workspace ; R56 a également rejeté ses six corruptions existantes. La validation globale canonique appartient à la promotion après capture Bevy.

1. `verify_facades_r54.mjs` exige aujourd'hui01 byte exact. Une exception strictement limitée à01, à ces deux îles et à un R59 authentifié est nécessaire. Tous autres batches historiques doivent rester comparés réellement aux archives R54, avec le traitement append-only existant06/08/09. Le budget historique R54 se calcule en retranchant les ajouts R56 puis le delta signé R59 ; le plafond architectural12000 ne change pas. L'ancien script et ses preuves doivent être archivés.
2. `terraces-source/verify_terraces.mjs` exige tous batches historiques exacts, dont01, et compte aujourd'hui toutes différences de triangles comme ajouts R56. Il faut vérifier01 séparément par R59 et compter comme ajouts R56 seulement06/08/09. Les attributs et corners historiques des11 autres batches restent exacts ; les metadata `cliff_r59` sont exclus du contrôle « non-HD R56 unchanged » uniquement parce qu'ils décrivent une étape ultérieure distincte. `authored.combined_triangles` reste l'ancien résultat R56 ; le total actuel doit être égal à cet historique plus le delta signé R59. Aucune preuve passée ne doit être réécrite.
3. `verify-open-world.mjs` doit invoquer le nouveau vérificateur R59 indépendant, qui compare01 au payload réellement construit, les racines physiques par signatures de tous corners, les11 autres batches aux GLB R56 figés, les bounds et données autoritaires, normales/tangentes/UV/triangles. Ne pas remplacer les comparaisons par une confiance dans le seul metadata.
4. `verify-islands.mjs`, `verify-variants.mjs`, `verify-rock-materials.mjs` en mode ordinaire et les gates matériaux n'exigent pas d'ancien nombre HD fixe et doivent garder leurs plafonds/contrats existants. Les modes historiques `--before` des passes matériau uniquement resteront légitimement incompatibles avec une substitution de géométrie ultérieure ; leurs preuves passées restent figées.

## Composition avec le sol R60

R60 reste une promotion distincte, après R59. Les overlays R54/R56 et le vérificateur R59 prennent en charge sa seule modification de matière12 via l'API indépendante `ground-r60-source/validate-ground-delta.mjs`, dont le SHA est exigé exact : `6ff191df9ff34a8d5c5283ce111cb5db1c95088ba78cbb1827abc465130f6c8c`. L'API doit d'abord authentifier le GLB R59 archivé complet avec le SHA d'assemblage R59, puis comparer le BIN complet et tout JSON sans rapport avec ce patch. Pour les deux LOD, le SHA d'entrée provient des fichiers historiques exacts. Seuls baseColorTexture/baseColorFactor12, le PNG ajouté, sa texture et KHR_texture_transform sont projetés vers R59 après ces contrôles. Les comparaisons géométriques continuent de lire les buffers actuels : aucun buffer de référence ne les remplace.

Les trois gates composées ont passé hors workspace, y compris les deux LOD R60. Sept corruptions sémantiques ont produit21 rejets sur ces trois gates après recalcul des checksums : géométrie sol/roche HD, géométrie sol LOD, autre matière, roughness12 HD/LOD et échelle UV. Les preuves R59 seules et R59+R60 restent séparées sous `evidence/`. Le vérificateur global conserve tous contrôles historiques, invoque R59 et invoque R60 si son paquet est installé.

Le paquet portable et la reproduction hors workspace sont vérifiés. Restent les captures Bevy avec les SHAs portables définitifs, le coût rendu/MASK et la validation globale après intégration. Un gain de qualité ne se déduit pas de ces chiffres géométriques. Le remplacement R59 porte exclusivement sur01.
