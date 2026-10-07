# Constructions et coûts mesurés

Mesures du 2 octobre 2026. Commande reproductible : `cargo test -p aether_sim --test envelope --test scenarios --release --locked -- --nocapture`. Les 24 tests passent ; [journal complet](evidence/physics-final-release.log).

## Construire modifie la simulation

| Variation | Résultat | Interprétation |
| --- | --- | --- |
| Coque initiale en cèdre, 10 s de sustentation | 6,958 unités consommées | Valeur de comparaison dans la fixture prescrite. |
| Même test, carène inférieure en métal | 9,796 unités consommées | +40,8 % : la force nécessaire augmente avec la masse. L'apparence ne décide pas du coût. |
| Voile déplacée de −0,5 m latéralement | Cap −1,343° après 2 s | Le point d'application de la force produit un couple. |
| Voile déplacée de +0,5 m latéralement | Cap +1,404° après 2 s | Le couple et le changement de cap s'inversent. |

Le déplacement de voile est réalisé sur le blueprint, puis le corps est recréé par les fonctions de production. Les deux essais partent du même état, hors courant, avec la même intention à 0,8. Les valeurs sont celles du profil release ; il n'y a pas de promesse de déterminisme binaire entre compilateurs ou profils.

## Fenêtre d'action du harpon

Le navire part du quai et se déplace par forces. Après détection d'une occasion valide, le test attend avant d'accrocher ; la portée ≤70 m et la ligne de vue sont vérifiées à l'instant réel de l'action. Le seuil d'arrivée reste 9 m du quai.

| Retard avant accroche | Distance minimale au refuge | Réserve à l'arrivée |
| --- | ---: | ---: |
| 0 s | 8,981 m | 165,354 / 180 |
| 0,5 s | 8,980 m | 166,011 / 180 |
| 1,5 s | 8,991 m | 165,916 / 180 |

Les trois trajets réussissent, sans téléportation du navire. Ce test borne une fenêtre technique ; il ne mesure pas la réaction d'une personne qui découvre le jeu.

## Stabilité de la physique

Les 54 combinaisons du câble croisent trois masses, trois vitesses, deux longueurs et trois cadences de rendu. Le pire p95 relatif de dépassement mesuré est 0,0000008742. La pause et la reprise figurent dans chaque cas. Les scénarios comprennent aussi pentes 15/30/39°, jonctions, sommeil/réveil et panne du navire chargé.

Le retard des repères interpolés atteint 0,047203 m à 60 Hz et 0,023409 m à 120 Hz dans la fixture release, rafales D46 actives ; il reste dans la borne d'un tick physique. Une téléportation réinitialise cette interpolation. Il ne s'agit pas d'une erreur accumulée entre la physique et les points d'équipements.

## Historique de performance

La scène contient toujours 10 000 cellules sur le corps principal et 31 autres corps dynamiques. Les mesures intermédiaires Edge à 1080p ont attribué le dépassement aux contacts Avian.

| Étape intermédiaire | Frame p50 / p95 / p99, ms | Tick p50 / p95 / p99, ms |
| --- | --- | --- |
| Avant corrections | 49,6 / 77,2 / 105,1 | 13,3 / 16,7 / 18,0 |
| WASM vitesse et avatar du seul navire actif | 19,2 / 36,0 / 41,3 | 11,7 / 14,4 / 15,9 |
| Fusion exacte des boîtes de terrain coplanaires | 14,3 / 17,7 / 20,2 | 7,9 / 9,4 / 10,5 |

[Avant](evidence/performance-before.json), [première correction](evidence/performance-avatar.json), [fusion](evidence/performance-merged.json). Ces étapes précèdent les derniers détails graphiques ; la [validation finale](VALIDATION.md) donne la mesure à retenir pour la livraison. Le test de volume et 9 000 intersections de segments protège l'équivalence des colliders fusionnés. Aucun navire physique n'a été retiré pour améliorer le chiffre.

Le meshing garde son implémentation greedy et ses formats. Les diagnostics séparent maillage CPU, application et attente. La mesure [entrée→PNG](evidence/edit-latency.json) est une borne haute avec polling, IPC et encodage ; elle ne doit être ni confondue avec l'upload GPU, ni vendue comme mesure commande→photon.

La première mesure d'édition interrogeait le rapport complet, publié toutes les 200 ms. Ce délai d'observation a été isolé : une sonde légère, en lecture seule, publie maintenant le nombre de cellules et de jobs à chaque image. Le rapport final utilise cette sonde, trois éditions d'échauffement et vingt mesures. Les anciens chiffres incluant la cadence de 5 Hz ne servent pas à attribuer un coût au mesher.

## Nettoyage à l'usage

Le profil CPU CDP a également isolé une reconstruction décorative lente : MikkTSpace recalculait les tangentes de toutes les petites faces des bordages. Le calcul analytique spécialisé est comparé à MikkTSpace sur les meshes réels et sur des UV inversés/tournés. Le greedy reste inchangé. Les mesures antérieures sont conservées dans `edit-latency-before-flat-tangents.json` et `edit-latency-diagnostic-before-flat-tangents.json` ; le profil brut est `edit-cpu-before.cpuprofile`. Le coût de décoration est publié séparément.

La mesure finale entrée→PNG sur 20 éditions donne p50/p95/p99 66,73 / 80,56 / 80,56 ms, contre 157,29 / 168,34 / 168,34 ms avant ce calcul spécialisé. Les mêmes étapes et la même zone de capture sont utilisées ; le candidat final inclut en plus la voile animée. Le délai jusqu'aux meshes prêts est généralement de 17–23 ms, avec un échantillon à 38,96 ms. Cette comparaison borne le chemin instrumenté, toujours distinct de la latence écran/photon.

La première endurance rendue complète a échoué : chaque retour de la marche laissait un capteur Tnua vivant. [Essai en échec conservé](evidence/rendered-soak-before-sensor-fix.json). Le test ciblé de vingt suppressions est [rouge avant](evidence/sensor-before.log), [vert après](evidence/sensor-after.log) le rattachement explicite du capteur au personnage. Le rapport final exige une nouvelle endurance complète du binaire corrigé et vérifie son empreinte ainsi que celles des actifs distribués.
