# Structure navale R56

Le navire réel est construit en Rust par `fixtures::explorer`, puis décoré par
`aether_view::craft`. `naval_deck` classe les marches, ponts supérieurs et appuis
de mât depuis les cellules et le repère du poste de pilotage. `naval::fittings`
réunit les poses que le rendu et les colliders consomment ensemble.

La géométrie n'est pas un GLB monolithique : chaque édition reconstruit les
surfaces dérivées de la nouvelle structure. La dunette, ses marches et le mât
sont des voxels autoritaires. Les cellules des sauvegardes existantes restent
autoritaires ; seul le modèle proposé aux nouvelles parties change.

Les tests actifs se reproduisent depuis la racine :

```powershell
cargo test -p aether_core naval_deck --locked
cargo test -p aether_sim --test naval_deck --locked
cargo test -p aether_sim --test navigation --locked
cargo test -p aether_view craft --locked
```

`archive/r56` conserve les blueprints avant/après, les sources Rust antérieures,
la scène Blender d'étude avec images packées, les quatre rendus Cycles et la
preuve initiale de domaine. `r56-archive-inventory.json` fixe les empreintes de
ces archives. Le `study-manifest.json` historique décrit le dossier d'étude
avant intégration ; il n'est pas le manifeste des fichiers de production actuels.
Les instructions de ce dossier historique sont archivées comme telles.

La validation du passage utilise le vrai contrôleur Tnua et les colliders Avian.
Les essais Cycles ne prouvent ni les contacts ni l'apparence finale dans Bevy.
Les captures natives et les décisions d'intégration sont décrites dans D83 et
`docs/RENDU.md`.
