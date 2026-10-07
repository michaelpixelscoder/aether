# Règles du reboot

Avant un changement : quel comportement du joueur ou quel risque mesurable permet-il de valider ?

- Rust 1.98, Bevy 0.19, Avian 0.7 ; versions résolues dans Cargo.lock.
- Ne pas exécuter de commande Git. Le prototype historique est dans `..` : ne pas modifier ses fichiers hors du dossier `aether-reboot` sans instruction explicite.
- `core` : données et algorithmes purs ; `sim` : autorité physique ; `view` : présentation ; `game` : composition et stockage.
- `view` ne dépend pas de `game` ou `sim`. Pas de données autoritaires dans les meshes.
- Pas d'unsafe. Entrées validées avant mutation. IDs persistants distincts des Entity Bevy.
- Une tâche terminée possède une preuve. Ne pas assimiler un test automatisé à un playtest humain.
- Validation : `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, build natif et WASM.
- Documenter décisions dans `docs/DECISIONS.md`, résultats dans `docs/VALIDATION.md` et états des 240 tâches dans `docs/TASKS.md`.
