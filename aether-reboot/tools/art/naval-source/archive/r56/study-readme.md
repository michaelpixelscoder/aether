# R56 — étude intégrable de la poupe de l’Alcyon

**Statut : candidat de structure, non promu et sans essai natif.** Le dossier ne modifie aucun fichier canonique. Aucun changement de voile, d’équipement, de langage, de format de sauvegarde ou de type de bloc.

La variante retenue remplace la haute cabine par un rangement fermé sous une dunette basse. La silhouette arrière perd son aspect de bâtiment à trois étages. Le petit mât conserve son appui, désormais lisible comme une charpente. Le volume de rangement n’est pas présenté comme une pièce visitable : tous ses voxels sont effectivement occupés.

## À regarder

- `evidence/baseline-hero.png` / `candidate-hero.png` : deux navires complets, même caméra et même éclairage Blender.
- `evidence/baseline-stern.png` / `candidate-stern.png` : détail comparable de la poupe et des accès.
- `source/candidate-study.blend` : scène complète, équipements/voiles originaux et textures embarquées. Les matériaux du rendu d’étude approchent les paramètres du jeu ; seule une capture native pourra valider leur réponse exacte.
- `source/candidate-blueprint.json` : la structure réelle proposée, au format `Blueprint` existant.
- `source/domain-gate.json` : métriques et assertions exécutées contre le vrai `aether_core`.
- `probe/src/raised_deck.rs` : propositions de prédicats purs et de poses de garde-corps, avec propriétaire de cellule explicite dans l’étude.

## Géométrie et passage

La carène et le pont principal (toutes les cellules Y ≤ 0, matériaux compris) restent identiques. L’emprise des cellules reste X ±4, Z −18…14, Y −2…7 ; les 14 équipements conservent IDs, cellules et rotations. La grande voile et la petite voile utilisent le GLB R52 canonique, sans retouche.

La dunette occupe X −2…2, Z 8…11, Y 1…2 ; sa surface est à 1,25 m. Deux marches, X −1…1, Y 1, Z 7 et 12, donnent un accès de 1,50 m à l’avant et à l’arrière. La surface du pont principal est à 0,25 m : chaque marche monte ou descend réellement de 0,50 m. La capsule du joueur devra être testée en mouvement sur ces marches avant promotion. Les dégagements statiques ne constituent pas cet essai.

Le pied du petit mât reste en `(0,3…7,10)`, avec appui de la voile inchangé en `(0,7,10)`. Le passage contourne ce support. Le rétrécissement arrière a été repoussé à Z=12 après qu’un test de capsule a détecté un étranglement lorsque la dunette se rétrécissait dès Z=11.

La masse passe de 10 890 à 10 464 kg (−3,91 %), et le centre de masse descend de 12,3 cm et avance de 20,3 cm. Aucun lest artificiel n’a été ajouté pour dissimuler cette différence. Vérifier accélération, freinage, virage, équilibre de portance et sortie du poste de pilotage ; le `boarding_cell` passe de `(0,0,7)` à `(0,0,6)`.

## Intégration minimale proposée

1. Porter la construction de `candidate()` dans `fixtures::explorer`, en conservant le nom de production. Ne pas remplacer automatiquement les vaisseaux déjà sauvegardés : leur blueprint édité doit rester autoritaire.
2. Porter les prédicats de `raised_deck.rs` dans le core. Ajouter les 16 poses `Kind::Rail` à `naval::fittings`, sans introduire une récursion entre le calcul de budget et la liste finale. Le helper d’étude appelle la liste canonique existante uniquement pour connaître le budget restant de 192 fittings.
3. Le `Kind::Rail` existant alimente déjà le rendu et `sim::vessel::spawn_equipment` : même pose, collider de 0,50 × 0,60 × 0,064 m, centre décalé de 0,30 m vers le haut. Aucun nouveau collider ou asset naval nécessaire.
4. Reporter les **quatre différences de conditions** entre `craft_snapshot.rs` et `craft_candidate.rs` : planches sur le niveau de dunette ; suppression de la mini-toiture sur les vrais supports de petite voile ; suppression de leur corniche sommitale ; suppression des anciennes balustrades décoratives sur les deux marches. Cette dernière condition apparaît à deux endroits du fichier.
5. Le cache `GeometryKey` existant inclut cellules et pièces. Toute édition doit donc recalculer ces prédicats, les fittings, les colliders correspondants et les surfaces ; aucun mesh monolithique de bateau n’est introduit.
6. Vérifier dans le moteur la silhouette avec la caméra R55, monter/descendre les marches des deux côtés, contourner le mât, quitter/reprendre la barre, puis retirer une marche, un voxel de garde-corps et un voxel de mât dans l’éditeur. Vérifier la scission et la sauvegarde/recharge de cette édition.

## Courbes et autorité physique

Les moulures, couples courbes et panneaux existants restent construits par tronçons de 0,50 m rattachés aux faces de bois exposées ; les profils et les coordonnées figurent dans `craft_candidate.rs`. Ces petites pièces n’effacent pas les cellules carrées. La structure basse rend déjà la silhouette plus navale sans changer cette règle.

Une carapace de coque lissée qui masquerait les coins de cellules conservées a été écartée. Un fût octogonal nécessitant des colliders spécifiques a également été écarté : le gain de cette étude vient du véritable changement des cellules et du chemin sur la dunette. Une future coque entièrement courbe nécessiterait un contrat de formes physiques partagé, décision de gameplay distincte de cette passe artistique.

## Vérification exécutée

- `Body::from_blueprint` accepte le candidat : pièces soutenues, réservations libres, identifiants valides ; un seul composant connexe.
- Pont/carine Y ≤ 0 et toutes les pièces comparés exactement au modèle d’origine.
- Six marches/accès et niveaux intermédiaires disposent du dégagement spécifié ; route autour du mât explicitée dans `domain-gate.json`.
- Suppression de chacun des 12 propriétaires des 16 nouveaux garde-corps : aucun garde-corps ne survit avec ce propriétaire.
- Poses et classification inchangées sous les quatre rotations de quart de tour et une translation incluant Y négatif.
- Une capsule de rayon 0,25 m garde au moins 0,095 m aux OBB des nouveaux garde-corps sur le trajet échantillonné. Ce test horizontal ne remplace ni un balayage continu ni le contrôleur physique.
- Projet Rust isolé compilé ; `cargo clippy --offline --all-targets -- -D warnings` passé. `cargo fmt --check` passé après formatage.

Les fittings totaux restent 84. Ils se regroupent dans les mêmes batches/entités que les modèles existants. Le détail des triangles est dans `domain-gate.json` ; la géométrie craft diminue par rapport aux 149 016 triangles du bateau d’origine. Aucun gain FPS n’est revendiqué avant capture native.

## Reproduire

Depuis la racine `aether-reboot` :

```powershell
cargo run --offline --manifest-path .dream-loop/naval-r56-candidate/probe/Cargo.toml -- .dream-loop/naval-r56-candidate/source
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background --python-exit-code 1 --threads 4 --python .dream-loop/naval-r56-candidate/source/review_naval.py -- all
node .dream-loop/naval-r56-candidate/source/package_study.mjs --verify
```

`study-manifest.json` donne les SHA des sources, des preuves, du fichier Blender et des entrées canoniques. Les dépendances restent les sources/GLB/textures déjà présentes dans le dépôt ; aucun asset téléchargé. Le dossier `probe/target` n’appartient pas au paquet.

L’essai A, qui ne faisait que raccourcir et baisser la cabine, reste dans `trial-a-*` comme preuve de l’alternative rejetée ; il ne doit pas être intégré.
