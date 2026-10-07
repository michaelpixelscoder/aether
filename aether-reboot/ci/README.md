# Validation distante à activer ultérieurement

Le modèle `windows-validation.yml` reprend les commandes locales sur un runner Windows. Il reste dans `ci`, sans dépôt distant associé ni exécution automatique pendant le reboot. Après création ou choix explicite d'un dépôt, le placer dans `.github/workflows` à la racine du workspace. Adapter le répertoire courant si le workspace reste imbriqué.

Les actions officielles checkout et upload-artifact sont fixées à v7.0.1, versions vérifiées dans leurs releases le 1er octobre 2026. Le job ne possède que la lecture du contenu et ne conserve pas d'identifiants de checkout. Il ne publie pas le jeu. Le format, Clippy, les tests et les builds native/WASM bloquent la validation en cas d'échec. Les journaux sont conservés même lors d'un échec.

Les tests avec GPU restent séparés et doivent tourner sur une machine disposant de WebGPU réel. Un runner standard sans ce matériel peut vérifier l'écran d'erreur de capacité, mais ne doit pas être annoncé comme une validation du rendu ou des performances. La procédure locale de référence est dans le README du projet.
