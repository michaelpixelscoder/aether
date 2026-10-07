# Préparer et restaurer une livraison

Le workspace produit un package Windows et un package web autonomes. La version de produit est 0.2.0 ; une empreinte des sources et actifs complète son identifiant. La compilation et les tests se font localement. Aucun dépôt distant ni hébergeur n'a été modifié.

## Commandes

```powershell
./tools/verify.ps1 -Web -Release
./tools/package.ps1
```

`-SkipBuild` réutilise des builds existants et refuse des sources Rust plus récentes ou des assets web différents. Le script ne remplace pas un package du même identifiant. `dist/latest.json` indique les chemins ; `MANIFEST.json` donne tailles et SHA256 ; `BUILD.json` identifie les sources. Les archives ZIP conservent ce dossier identifié.

Les archives de cette livraison ont aussi été relues entrée par entrée et comparées à leur manifeste : [archive-integrity.json](evidence/archive-integrity.json). Un fichier `.zip.sha256` accompagne chaque ZIP retenu. Pour vérifier une copie, comparer `Get-FileHash -Algorithm SHA256 <archive.zip>` à cette empreinte avant extraction.

## Vérifier le package

Copier le dossier Windows vers un répertoire neuf hors workspace puis lancer l'exécutable depuis un autre répertoire courant. Utiliser `AETHER_DATA` vers un dossier temporaire lors d'un test. Nouvelle partie, édition, sauvegarde, fermeture et reprise doivent fonctionner sans accès aux sources. Le test `--qa` exerce aussi les codecs, scissions, attaches et cycles de sessions ; son script contient des déplacements de préparation et ne remplace pas la traversée physique séparée.

L'archive contient le programme et ses actifs ; le runtime système Visual C++ v14 x64 reste un prérequis Windows. L'inspection des imports est conservée dans `evidence/windows-dependencies.txt`. Les essais hors workspace ont lieu sur la machine de référence, pas dans une image Windows vierge. La page Microsoft liée dans les commandes permet d'installer le redistribuable sans installer les outils de développement.

Servir le dossier web sur HTTP local pour la vérification, HTTPS pour une distribution réelle. `node tools/serve.mjs <dossier-web> 4174` sert les fichiers `.br` ou `.gz` lorsque le navigateur les accepte. Tester l'entrée à la racine et sous un préfixe, par exemple `/demo/`. Le WASM doit être servi avec `application/wasm` et la compression avec le bon `Content-Encoding`.

## Déploiement cohérent à effectuer ultérieurement

Conserver chaque ensemble sous `r/<identifiant>/` avec ses JS, WASM, assets et manifeste. Déposer entièrement ce répertoire avant de remplacer la page d'entrée. Cette page doit avoir `Cache-Control: no-store` ; un répertoire d'empreinte peut être immuable. Ne jamais remplacer un WASM ou un asset dans un répertoire déjà publié.

Pour revenir en arrière, rétablir la page d'entrée qui pointe vers un ancien répertoire encore présent. Les sauvegardes restent attachées à la même origine IndexedDB, mais 0.2 migre le format 1 vers le format 2. L'ancien jeu refuse le format 2 : un retour complet à 0.1 demande une copie des données datant d'avant la migration. Le backup automatique n'est pas une archive permanente de cette version antérieure. Le test de package exerce la migration réelle 0.1 → 0.2, la motorisation, le refus par 0.1 puis la reprise intacte dans 0.2. Cela n'établit pas une migration depuis le prototype historique.

## Limites de la validation

Les mesures publiées concernent la machine et les navigateurs identifiés dans `VALIDATION.md`. Elles ne sont pas une certification de tous les matériels WebGPU. Le mode web peut refuser un adapter ou un device indisponible et propose de réessayer. Le modèle CI est fourni sous `ci` et n'a pas été connecté à une plateforme distante. La publication reste une opération distincte de ce reboot.
