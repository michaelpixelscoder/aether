# Protocole de test joueur

Ce protocole prépare la validation humaine de la démo. Aucun entretien ni essai de cinq personnes n'est déclaré réalisé par les tests automatisés. Damien choisit les participants et le mode de remise du package ; aucune invitation n'a été envoyée.

## Hypothèse

Retour spontané de Damien, 2 octobre 2026, pendant l'essai de la version Windows proposée `0.1.0-784a02ffedfb` : la navigation sur le pont est appréciée, ainsi que la voile en voxels ; il souhaite des variations de vent pour voir la toile bouger. Ce sont les appréciations exprimées dans la conversation, sans chronométrage, observation directe complète ni mesure de compréhension. La demande a ouvert D46 (rafales douces et déformation de voile). Elle ne clôture pas le protocole de cinq personnes ci-dessous.

Une modification de construction doit produire un effet que le joueur comprend et qui lui donne envie d'essayer une autre traversée. La stabilité technique permet cette observation ; un bon framerate ne suffit pas à établir le plaisir.

## Préparation

Utiliser le package identifié dans `dist/latest.json`, un répertoire de sauvegarde neuf via `AETHER_DATA` ou un nouveau profil de navigateur, un clavier et une souris. Noter machine, navigateur, résolution, identifiant du package et expérience préalable. Vérifier le lancement avant la séance, sans montrer le parcours à la personne.

Faire d'abord une répétition avec une personne connaissant peu la version. Corriger un éventuel blocage d'installation avant de recruter cinq participants. Les cinq séances se font séparément. Prévoir 10–15 minutes de jeu, puis quelques questions ouvertes. Demander l'accord avant d'enregistrer écran ou voix ; des notes suffisent.

## Consigne à lire

« Vous disposez d'un vaisseau au chantier. Préparez-le à votre idée puis essayez de rejoindre les jardins suspendus. Explorez les commandes affichées et les possibilités du courant et du harpon. Vous pouvez recommencer et modifier la construction. Dites ce que vous cherchez à faire lorsque vous en avez envie. »

Ne pas décrire la séquence gagnante. Noter chaque aide fournie, son instant et sa raison. Si la partie bloque durablement, proposer le secours déjà affiché ; distinguer ce soutien d'une réussite autonome. Arrêter si un défaut technique empêche le test ou si la personne le souhaite.

## Fiche par personne

| Champ | À renseigner lors de l'observation |
| --- | --- |
| Identifiant anonyme et date | |
| Package, machine et disposition clavier | |
| Expérience préalable de cette version | |
| Accord pour capture éventuelle | |
| Temps jusqu'à la première modification | |
| Temps jusqu'au premier départ | |
| Modification choisie et effet attendu | |
| Effet observé et explication donnée | |
| Compréhension de la réserve et de la panne | |
| Usage autonome du courant et du harpon | |
| Arrivée, secours ou abandon | |
| Nombre et nature des aides | |
| Nouvelle tentative ou modification volontaire | |
| Citation courte autorisée | |
| Faits observés | |
| Interprétation de l'observateur | |

Après le jeu : « Qu'est-ce qui a changé quand vous avez modifié le vaisseau ? », « Qu'essayeriez-vous ensuite ? », « À quel moment ne saviez-vous plus quoi faire ? ». Ne pas demander seulement si le jeu plaît.

## Décision après cinq séances

Consolider les observations avec leurs dénominateurs, par exemple 3/5, et garder séparées les personnes ayant déjà joué. Le seuil indicatif de compréhension autonome est 4/5, accompagné d'exemples et contre-exemples. Classer les défauts : perte ou crash, parcours bloqué, gêne, cosmétique.

Choisir au plus trois corrections et une hypothèse de plaisir. Prévoir un cycle initial de huit heures avec les tests concernés, puis deux essais ciblés en distinguant nouveaux joueurs et joueurs formés. Le dossier de décision contiendra les cinq fiches, les changements, les limites et la recommandation à Damien. La demande directe de monde ouvert ci-dessous active ce lot sans attendre ces cinq séances ; elle ne constitue pas cinq observations indépendantes.

## Retour direct et nouveau périmètre — 2 octobre 2026

Damien apprécie la marche sur le pont et le matériau de la voile. Il juge le monde « beaucoup trop basique, 5/10 au mieux » et demande de reprendre les planches, d'agrandir fortement le monde et d'en développer les interactions. Il signale aussi une navigation très difficile : il veut pouvoir choisir sa direction grâce à une force motrice suffisante sans vent, bénéficier des vents favorables et conserver l'inertie à grande vitesse.

Conséquences actives : moteurs constructibles et marche arrière lente, neuf régions et profondeur souterraine, courants plus lisibles, nouveaux modèles détaillés, carte et découvertes persistantes, récolte/commerce/contrats/portails, créatures observables et circulation de dirigeables. Les décisions et limitations sont consignées dans D47 et suivantes. Un script aux touches vérifie les parcours, mais la sensation de pilotage et la satisfaction visuelle devront être réévaluées par Damien sur ce candidat ; aucun avis favorable n'est présumé.
