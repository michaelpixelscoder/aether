# Rafales et voile voxel — 2 octobre 2026

La version `0.1.0-27bffa9ee363` ajoute le mouvement demandé pendant l'essai de Damien. Le caractère voxel de la toile et son emblème sont conservés.

## Essayer

Lancer `dist/windows/0.1.0-27bffa9ee363/Aether-Isles.exe`, puis Nouvelle partie ou Reprendre. La toile réagit dès le quai. Approcher avec la molette et tourner avec le clic droit ; F8 masque l'interface. Échap fige le mouvement. Le réglage de mouvement réduit atténue les ondulations.

[Vidéo capturée dans le jeu](evidence/sail-wind.webm) · [Première image](evidence/sail-close-a.png) · [Seconde image](evidence/sail-close-b.png).

## Comportement

Le vent dominant reçoit des rafales progressives, bornées à ±13,5 % de sa vitesse, et une oscillation supplémentaire de direction de ±2,6°. Graine, position et tick déterminent le champ ; aucune nouvelle suite aléatoire ne se relance à chaque image ou chargement. La physique utilise ce champ pour la poussée.

Le renderer reçoit le vent apparent à la voile, après soustraction de la vitesse du navire et de la vitesse due à sa rotation. Trois cibles de déformation animent pression et ondulation. La toile et l'emblème utilisent le même déplacement ; mât, vergues et cordages restent fixes. Les voxels sont conservés dans le modèle. Aucun mesh CPU n'est reconstruit pendant l'animation.

Ce sont des déformations visuelles bornées, sans simulation de tissu. La surface aérodynamique simplifiée reste la référence mécanique. La phase suit le temps physique, donc la pause la gèle et la sauvegarde conserve son repère temporel. Le mouvement réduit conserve la pression et réduit l'ondulation à 25 %.

## Vérifications

- 84 tests Rust, format et Clippy natif/WASM ; builds Windows/Web optimisés.
- 24 scénarios physiques release, dont les traversées, le décalage de voile, les pentes, l'interpolation et la matrice de câble.
- 121 contrôles Windows et archive extraite hors workspace avec sauvegarde effective.
- 15 tests Chrome et 15 Edge ; le test de voile vérifie poids variables, pause figée, reprise et nombre de meshes constant.
- Export GLB contrôlé : cibles indépendantes et amplitudes ≤18 cm / 6,5 cm. La première exportation cumulée a été rejetée ; les journaux avant/après sont conservés.

L'endurance rendue passe 30 min 23 s et 69 cycles sans erreur ni augmentation d'entités, meshes, matériaux ou mémoire. Les mesures et la réserve de performance native figurent dans [VALIDATION](VALIDATION.md). Le package intermédiaire `0.1.0-6ab290499f34` est un essai rejeté ; utiliser le candidat corrigé ci-dessus. Le chantier global, la fidélité aux planches et les essais joueurs restent suivis dans [FINISHING](FINISHING.md), [RENDU](RENDU.md) et [PLAYTEST](PLAYTEST.md).
