# Aether Isles

Jeu Rust de construction et d'exploration aérienne. Le monde comporte 204 îles réparties entre neuf régions, auxquelles s'ajoutent les trois escales d'apprentissage. Naviguez dans les courants, explorez les capitales et les cavernes, récoltez, commercez et transformez votre navire au chantier. Le reboot est dans `aether/aether-reboot/` ; le prototype historique reste dans le dossier parent `..`. Les deux workspaces Rust sont indépendants.

L'extension 0.2 décrite ici est en cours dans les sources. `dist/latest.json` pointe encore vers la distribution historique 0.1 ; elle ne contient pas ce monde étendu. La qualité visuelle et les validations finales sont suivies dans [RENDU](docs/RENDU.md) et [VALIDATION](docs/VALIDATION.md).

[Notes de version](docs/NOTES-VERSION.md) · [Aperçu du chantier](docs/evidence/atelier-final.png) · [Direction artistique](docs/RENDU.md) · [Résultats de validation](docs/VALIDATION.md)

## Jouer

Le package Windows est indiqué par [`dist/latest.json`](dist/latest.json). Dans son dossier `windows`, lancer `Aether-Isles.exe` en conservant `assets` à côté. Aucune installation de Rust n'est nécessaire pour jouer au package.

Le binaire utilise `VCRUNTIME140.dll`, vérifié par inspection de ses imports. Sur une autre machine Windows, installer si nécessaire le [Visual C++ v14 Redistributable x64 de Microsoft](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170). Il est déjà disponible sur la machine de validation.

Depuis les sources, dans PowerShell ouvert dans ce dossier :

```powershell
./tools/run.ps1
# Build optimisé
./tools/run.ps1 -Release
```

`F` quitte le quai. Maintenez `Z/W/↑` pour les moteurs, `Q/D` ou les flèches pour diriger, `E/C` pour l'altitude et `S/↓` pour freiner. Les voiles prennent automatiquement le vent ; `J/X` ajuste leur orientation. `M` ouvre l'atlas et les comptoirs, `B` récolte ou interagit. La Couronne de l'Aube est la première capitale proposée. Freinez à l'approche, puis appuyez sur `F` à moins de 9 m du quai et sous 3,5 m/s. `Tab` ouvre l'atelier au quai ; `T` permet de marcher. `R` ramène au dernier quai en conservant la construction principale. [Commandes et règles complètes](docs/CONTROLS.md).

Les sauvegardes V1/V2 sont migrées au chargement. Les sources actuelles utilisent le **format V3**, qui conserve aussi les circuits d'Aether ; les anciens exécutables refusent ce format. Le comptoir propose une motorisation pour les anciennes coques qui en sont dépourvues.

Pour essayer la distribution, restez amarré, ouvrez `M`, puis **Atelier des circuits d'Aether → Installer la distribution**. Sélectionnez deux équipements pour ajouter une connexion, utilisez leurs vannes pour isoler les branches, puis sauvegardez. Une branche sans réserve coupe réellement ses moteurs ou sustentateurs. Annuler/rétablir change le plan sans restaurer le carburant consommé. Les conduites visibles en 3D et les pompes jouables sont encore en développement.

`K` commande une marche arrière lente pour les manœuvres. La réserve, le poids de la cargaison, les découvertes et les interactions sont décrits dans [MONDE-OUVERT](docs/MONDE-OUVERT.md).

## Compiler et vérifier

Les assets et sources graphiques binaires sont versionnés avec **Git LFS**. Après un clone du dépôt parent, installer Git LFS et exécuter `git lfs pull` à sa racine avant de compiler : les petits pointeurs LFS seuls ne sont pas des textures ou modèles utilisables. Les builds `target/`, distributions `dist/`, dépendances locales et études temporaires `.dream-loop/` sont ignorés ; ils restent disponibles localement et se reconstruisent avec les outils du projet. Les fichiers auteur et les baselines nécessaires aux vérificateurs sont conservés.

Prérequis de développement : Windows avec outils C++ MSVC et SDK Windows, Rust installé par rustup. `rust-toolchain.toml` fixe Rust 1.98. Node 24 sert uniquement au packaging et aux tests navigateur. Les bibliothèques sont verrouillées dans `Cargo.lock`.

```powershell
./tools/setup.ps1 -Browser
./tools/verify.ps1
./tools/verify.ps1 -Web -Release
./tools/run.ps1 -QA
./tools/run.ps1 -Gallery
./tools/run.ps1 -Release -Benchmark
```

Le navigateur cible est Chrome ou Edge desktop avec WebGPU. Pour un build local :

```powershell
./tools/bin/bevy.exe build --yes --locked -p aether_game web --bundle --wasm-opt=false
node tools/serve.mjs
# Dans un second terminal
cd tools
$env:AETHER_BROWSER='chrome' # ou 'msedge'
npx playwright test
```

Le serveur affiche son URL locale. Le HTML ne se lance pas par `file://`. Pour produire les deux archives autonomes, utiliser `./tools/package.ps1`. Les artefacts versionnés et leurs empreintes vont dans `dist` ; aucune publication n'est effectuée.

Les tests utilisent Edge par défaut, ou le navigateur choisi dans `AETHER_BROWSER`. Pour rejouer l'endurance rendue complète, garder le serveur ouvert puis lancer `node soak.mjs` depuis `tools` : ce scénario dure au moins trente minutes. `node edit-latency.mjs` mesure vingt éditions avec lecture d'une région de l'image, en séparant préparation et capture. Les résultats vont dans `docs/evidence`.

## Organisation

```text
crates/
  aether_core/  données, commandes, grille, meshing, codecs, terrain et champs
  aether_sim/   Avian, Tnua, vol, personnages, collisions et joints
  aether_view/  matériaux, meshes, personnage, environnement et galerie
  aether_game/  états, commandes, atelier, sauvegardes, interface et composition
assets/        actifs consommés par le jeu
web/           page de lancement WebGPU
tools/         commandes Windows, serveur, packaging et tests Playwright
ci/            modèle de validation à activer ultérieurement
docs/          décisions, formats, preuves et suivi des 240 tâches
dist/          packages et archives versionnés
```

Commencer par [l'architecture](docs/ARCHITECTURE.md), [les décisions](docs/DECISIONS.md), [la validation](docs/VALIDATION.md) et [le suivi du plan](docs/TASKS.md). Les limites sont 10 000 cellules par corps, 128 cellules par axe, 32 corps de construction et 16 Mio par sauvegarde. Le monde étendu et son chargement par proximité sont activés par la demande du 2 octobre. Les observations de cinq joueurs restent une validation humaine distincte ; le [protocole](docs/PLAYTEST.md) est prêt.

Aucune commande Git n'a été exécutée pour ce reboot. La traçabilité locale repose sur les empreintes des sources, des actifs et des packages.
