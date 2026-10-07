# Registre des défauts et limites

Gravité : P0 perte de données ou crash, P1 parcours bloqué, P2 gêne bornée, P3 cosmétique. Les preuves finales sont liées dans [VALIDATION](VALIDATION.md). Une limite non éprouvée n'est pas transformée en réussite.

## Corrections fermées par un scénario reproductible

| Défaut corrigé | Risque | Protection |
| --- | --- | --- |
| Clignotement et perte de sélection après attribution d'une touche | Impression de refus et focus clavier perdu | Actualisation des composants existants, lignes de taille stable et sélection conservée après réussite ; tests Bevy et navigateur R75, D109. |
| Première frappe perdue après clic d'attribution ; Maj ignorée et conflit sans échange | Raccourci inchangé malgré la saisie | Collecte après les boutons, capture de la demande en file, Maj persistante, refus expliqué et échange confirmé atomique. Tests Bevy et parcours navigateur R74c : D108 et VALIDATION. |
| Résultat de maillage ancien appliqué après une nouvelle révision | Collision et présentation incohérentes | Révisions, priorité du corps édité, test de fin inversée dans le vrai consommateur. |
| File encore vide dans l'image du remplacement de session | Départ/amarrage avant reconstruction | Préparation annoncée immédiatement ; QA attend le mesh avant d'amarrer. |
| `Entities::len()` interprété comme nombre d'entités vivantes | Faux diagnostic de nettoyage | `World::entity_count()`, compteur d'indices séparé, handles suivis sur 20 cycles. |
| Knight absent bloquant tout le chargement | Repli personnage inutilisable | Asset optionnel, représentation de secours conservée ; marche et sauvegarde testées dans les navigateurs. |
| Troncature ou interruption du stockage | Perte du seul état chargeable | Validation, backup, douze frontières de panne injectées, transactions IndexedDB strictes. |
| Fermeture native annonçant un ancien succès d'écriture | Dernières modifications perdues | Attente de la nouvelle sauvegarde ; tests de résultat et fermeture du programme compilé. |
| Commande encore tenue après perte de focus | Navigation involontaire | Libération d'entrée et pause ; vrai blur du canvas éprouvé. |
| Plus de 70 ms par frame sur Edge | Budget web dépassé | Profilage Avian, optimisation WASM et fusion exacte du terrain ; nouvelles mesures sur la scène complète. |
| Textures fines sans mipmaps | Moiré et scintillement | Mips en lumière linéaire, dimensions impaires testées, filtrage trilinéaire et anisotrope. |
| Filtrage appliqué à des atlas de texte modifiables | Niveaux réduits périmés après ajout de glyphes | Traitement limité aux images chargées depuis un asset ; test d'exclusion d'un atlas RGBA sRGB dynamique. |
| Capteur Tnua survivant au personnage | Une entité par cycle marche/pilotage, accumulation longue durée | Parentage de durée de vie, test de vingt suppressions initialement rouge puis vert, endurance complète relancée. |
| Rapport publié à 5 Hz utilisé pour une latence d'édition | Jusqu'à 200 ms de faux retard de reconstruction | Sonde légère par image, mesure PNG séparant état prêt et lecture du navigateur. |
| Identifiant importé proche de la borne du format | Ajout refusé ou annonce de scission sans mutation | Réservation d'un ID libre, conservation des IDs présents, régressions d'édition et de flotte maximale. |
| Bouton HTML plein écran devant la télémétrie | Titre partiellement masqué à 1280×720 | Placement dans le bandeau supérieur, contrôle plein écran/redimensionnement rejoué sur les deux navigateurs. |

## Limites connues

**Extension courante R72 :** recherche spatiale des conduites testée mais sans pose, raccords ni collisions en jeu ; maison R71 non promue, toiture trop uniforme ; autres bâtiments encore répétitifs. Benchmark natif au calme p95 18,51 ms au-dessus de la cible 16,7 ms. Endurance finale et distribution0.2 restent ouvertes. Les anciennes preuves du tableau ci-dessous concernent leurs versions respectives.

| Niveau | Cas | Comportement et portée |
| --- | --- | --- |
| P2 | Plusieurs surfaces transparentes superposées | Le tri alpha peut présenter un ordre imparfait ; le verre reste séparé des surfaces opaques. |
| P2 | Variabilité du benchmark Windows avec voile animée | Première passe p95 frame 26,17 ms ; deux répétitions isolées du même binaire donnent 8,62 et 7,66 ms. Première pointe de prépasse GPU conservée, cause exacte non isolée ; pas de certification du premier lancement à froid. [Trois mesures](VALIDATION.md#répétition-de-la-mesure-windows). |
| P2 | Fermeture forcée d'un onglet pendant une écriture | Seule une transaction déjà commitée est garantie ; autosave, sauvegarde manuelle et export restent disponibles. |
| P2 | Préparation du benchmark Windows | Quatre avertissements Bevy `ChildOf` vers un parent absent figurent au démarrage du benchmark natif. Ils ne figurent pas dans la QA native ni la galerie de cette passe ; la cause exacte n'est pas isolée. Le test ciblé d'un résultat de mesh suivi d'une destruction de session passe déjà sans correction. Aucun correctif spéculatif ou simple filtrage du journal n'est livré ; [journal](evidence/native-benchmark.stderr.log), [contrôle ciblé](evidence/deferred-owner-probe.log). |
| P3 | Voile et câble | Toile orientable et déformée par morph targets, sans simulation de tissu ; câble sans enroulement autour des obstacles. Aucun second solveur caché. |
| P3 | Fines garnitures | Ferrures et bordages dérivés des cellules, jusqu'à 1024 cellules décorées par coque ; les 10 000 cellules restent visibles et physiques. |

Les coupures électriques réelles, la veille Windows, le bfcache, la VRAM isolée et la latence physique écran/photon ne sont pas certifiés. L'absence de timestamp GPU web est signalée. L'essai source propre partage les caches locaux ; l'essai portable n'est pas un Windows vierge. Les observations de cinq joueurs et la décision de plaisir restent dans [PLAYTEST](PLAYTEST.md). Aucun résultat humain n'est simulé.
