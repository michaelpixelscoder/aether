# Nuages locaux R57 â€” preuve CPU distincte et compatibilitÃ© historique

Le shader R57 conserve exactement tous les octets R55 sauf deux radiances : le remplissage ambiant reÃ§oit RGBÃ—(4,4.5,5), et le direct devient RGB(3,1.86,.96). Le bounce conserve sa couleur et sa force .35. L'intÃ©gration, les densitÃ©s, les ombres et les normales restent identiques.

La preuve est un nouveau calcul de 24 champs CPU : analytique, cache et densitÃ© cache avec normale analytique, pour les deux bancs, deux camÃ©ras et deux cadences d'Ã©clairage. Elle garde les rayons 240Ã—150, 128 Ã©tapes, bruit fixÃ©, phase et transform d'affichage R55. Aucun pixel R55 n'est multipliÃ© ou relabellisÃ©. Les fichiers PNG sont des inspections mathÃ©matiques CPU nouvelles ; ils ne sont pas des captures du moteur. Les floats et leurs empreintes sont conservÃ©s.

Les huit comparaisons passent les seuils hÃ©ritÃ©s : RMSE opaque maximale .009915231 < .025 ; opacitÃ© de bord p99 maximale .075714256 < .08. Toutes les distributions d'erreur d'opacitÃ© restent exactement R55, elles-mÃªmes protÃ©gÃ©es par R47. Le calcul a durÃ© 327,238 s avec au maximum quatre threads, pendant les builds/tests du parent ; ce temps mural ne permet aucune conclusion de performance du jeu.

`verify-cloud-lighting.mjs` enchaÃ®ne maintenant deux contrats distincts. L'adaptateur R55 vÃ©rifie son shader et son vÃ©rificateur archivÃ©s byte exact, ses sources originales et ses anciens rayons, sans Ã©crire ses rÃ©sultats historiques. Le gate indÃ©pendant R57 vÃ©rifie la nouvelle radiance, le nouveau renderer, ses rayons et ses artefacts. Il reconstruit aussi l'adaptateur R55 depuis la source historique et n'autorise que les adaptations de chemins/commentaires. Les scripts R55, sources.json, contract.json et leurs preuves sont conservÃ©s sans modification. Le comparatif du renderer neutralise seulement les fins de lignes CRLF/LF produites par Python sous Windows, puis retrouve exactement son texte R55 en annulant les deux radiances et les mÃ©tadonnÃ©es de destination.

Depuis la racine du projet ou du paquet portable :

```powershell
$env:OMP_NUM_THREADS='4'; $env:OPENBLAS_NUM_THREADS='4'; $env:MKL_NUM_THREADS='4'; $env:TBB_NUM_THREADS='4'
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' --background -t 4 --python tools/art/compare_cloud_light_r57_rays_cpu.py
& 'C:/Program Files/Blender Foundation/Blender 5.2/5.2/python/bin/python.exe' tools/art/cloud-r57/freeze-proof.py
node tools/art/verify-cloud-lighting.mjs
node tools/art/cloud-r57/test-gate.mjs .dream-loop/cloud-r57-negative-tests
```

Les vingt corruptions couvrent seuils, camÃ©ras, cadences, recettes, soleil/bounce, opacitÃ©, radiance CPU, densitÃ©/intÃ©gration, floats stockÃ©s et archives. Le contrÃ´le rÃ©gional ancien .035 reste explicitement Ã©chouÃ©, sans Ã©largissement rÃ©troactif. Limites : bruit fixÃ©, rayons orthographiques, fond et transform d'affichage historiques ; pas de profondeur opaque, advection dynamique, reprojection, ACES/bloom natifs ou simulation exacte de diffusion multiple. La calibration surface keyÃ—1.25/envÃ—.5 appartient Ã  la production du parent et n'est pas simulÃ©e par cette preuve des nuages locaux.
