# Jouer à Aether Isles

Vous partez du chantier avec l'Alcyon — Expédition : coque de 16,5 m, deux voiles, deux hélices motrices, six sustentateurs, deux réservoirs et un harpon. L'atlas propose la Couronne de l'Aube comme première capitale. Les jardins et le refuge restent accessibles pour apprendre les commandes sur une courte traversée.

Sous Windows, conserver le dossier `assets` près de l'exécutable. Si Windows signale `VCRUNTIME140.dll` absent, installer le [Visual C++ v14 Redistributable x64 officiel](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170). Rust et Visual Studio servent uniquement au développement.

## Commandes

| Action | Commande |
| --- | --- |
| Commencer depuis le menu | Entrée ou bouton |
| Reprendre une sauvegarde depuis le menu | L ou bouton |
| Quitter le quai ou s'amarrer | F |
| Atelier au quai | Tab |
| Gouvernail | Q ou A, D, flèches gauche et droite |
| Avancer avec les hélices | Z, W ou flèche haut, maintenir |
| Marche arrière lente des hélices | K, maintenir |
| Freiner et fermer la voile | S ou flèche bas, maintenir |
| Orienter la voile | J et X, maintenir |
| Régler l'altitude cible | E et C, maintenir |
| Accrocher l'ancre visible la plus proche ou libérer | G |
| Marcher ou reprendre le poste | T |
| Marche | ZQSD, WASD ou flèches |
| Saut en marche | Espace |
| Secours au dernier quai | R |
| Atlas, soute et comptoir du port | M |
| Récolter, observer une créature ou activer un portail | B |
| Orbite de caméra | Glisser avec clic droit |
| Zoom | Molette |
| Pause et reprise | Échap |
| Sauvegarder ou charger | Ctrl+S ou Ctrl+O |
| Ajouter une cellule | Clic gauche bref sur une face |
| Retirer une cellule | Maj+clic gauche ou clic milieu |
| Bois, métal, verre | Touches de la rangée 1, 2, 3 ou boutons |
| Choisir un composant | Bouton dans l'atelier |
| Tourner le composant avant placement | V |
| Annuler ou rétablir | Ctrl+Z ou Ctrl+Y, dans l'atelier |
| Diagnostic local | F3 |
| Navigation des boutons au clavier pendant le jeu | F6, puis Tab/flèches et Entrée/Espace |
| Masquer ou rétablir l'interface pour une photo | F8 |

Les lettres utilisent la disposition logique du clavier ; les flèches restent disponibles. Aucun texte à saisir n'intercepte les commandes de vol. L'interface empêche les clics de construction sous ses panneaux.

## Construire puis partir

L'aperçu vert signale un ajout valide, orange un retrait, rouge un refus avec explication. Une voile réserve plusieurs cellules. Retirez un composant avant de supprimer son support. Les cellules peuvent être négatives et les textures restent attachées à la coque.

La coque doit être connexe et posséder poste, réservoir et sustentateur avant le départ. Une masse excessive peut tout de même empêcher le vol : le poids et le déficit sont affichés. Reliez les morceaux isolés ou utilisez « Séparer les fragments ». La séparation est annulable immédiatement au quai. Une édition après une séparation commence un nouvel historique de construction ; elle efface l'annulation globale de cette séparation. L'historique conserve au plus 128 transactions et 16 Mio estimés.

## Naviguer

Maintenez la commande d'avance pour les hélices. Elles permettent de progresser sans vent et de remonter un vent contraire. `K` inverse doucement leur poussée pour reculer au quai ou se dégager d'un obstacle ; les voiles sont alors repliées. Les voiles se déploient automatiquement sur un navire motorisé en marche normale et ajoutent leur poussée ; le vent de face est atténué par une réduction automatique de voilure. La dérive est amortie progressivement. À haute vitesse, tourner la coque ne fait pas pivoter instantanément sa trajectoire : anticipez et freinez avant les bâtiments.

Les sustentateurs maintiennent l'altitude demandée tant que la réserve et la puissance suffisent. Un courant montant ou descendant entraîne aussi cette consigne ; `E/C` permet d'en sortir. Le frein amortit fortement la vitesse horizontale sans empêcher la descente. Ajouter du métal ou de la cargaison augmente la masse, l'inertie et la consommation ; la surcharge peut empêcher le vol. Les hélices consomment de l'Aether uniquement lorsqu'elles poussent.


Pour une première traversée, gagnez progressivement 20 m d'altitude. Suivez la branche principale du courant. À environ 35 m des jardins, maintenez S. Le quai accepte l'amarrage à moins de 9 m et sous 3,5 m/s. Le HUD indique distance, cap et état de câble.

Le harpon accroche une seule ancre visible à moins de 70 m. Le câble est lâche ou tendu selon sa longueur ; il ne pousse jamais le navire. Libérer conserve la vitesse. Un obstacle sur son segment le détache. Aucun enroulement autour des îles n'est simulé. L'ancre à gauche du premier courant permet un balancier vers le refuge ; relâchez quand votre mouvement vous emporte vers sa branche, puis freinez à proximité du petit quai.

## Explorer et ravitailler

Ouvrez l'atlas avec `M`, choisissez une capitale et relevez l'altitude de son quai. Les quinze grands courants relient ou contournent les régions ; deux courants plus courts desservent les escales d'apprentissage. Les arrivées sont apaisées près des ports. Les Profondeurs Oubliées sont centrées à −1 400 m et les Sous-forges à −2 200 m : suivez leurs puits ouverts ou leurs entrées latérales. La plage de consigne est de −3 000 à +2 200 m.

La découverte des îles complète l'atlas et rapporte une prime unique. `B` récolte cinq unités au voisinage des cristaux ou des bassins visibles ; un même point ne se récolte qu'une fois dans cette partie. À pied, il faut s'approcher davantage. L'eau et les cristaux ont des usages et des quantités distincts dans la soute.

Au quai, `M` ou `B` ouvre le comptoir : six marchandises, prix régionaux, achat/vente et un contrat de huit unités par capitale. La capacité est de 400 unités ; chaque marchandise possède sa masse. Le raffinage consomme un cristal pour restituer jusqu'à 240 unités d'Aether. Les anciennes coques sans hélice peuvent être motorisées au comptoir contre quatre bois, quatre fers et deux cristaux, si deux emplacements sont disponibles.

Dans les cavernes, approchez les gardiens ou les planeurs et appuyez sur `B` avec une ligne de vue dégagée. Chaque espèce observée rapporte une prime unique de 75 crédits. Les dirigeables marchands suivent leurs circuits et possèdent une collision ; les transactions se font aux ports.

Pour utiliser un portail de capitale, sélectionnez dans l'atlas un autre port déjà découvert, puis approchez le navire de l'arche et appuyez sur `B`. Le passage coûte 80 Aether, libère le câble, remet la vitesse à zéro et vous place devant le quai de destination. Un délai de quinze secondes de simulation sépare deux passages. La marche à pied ne déclenche pas ce transfert.

## Sauvegarde et secours

Pour parcourir rapidement le monde, ouvrez **Échap → Voyage rapide** ou **M → Voyage rapide**. Les onglets proposent les lieux remarquables, toutes les îles, les souterrains et les courants aériens. Les pages donnent accès aussi aux lieux encore inconnus. Cliquez sur une carte pour y aller avec votre navire, sans coût de voyage ; vous arrivez au poste de pilotage, à l'arrêt, devant le quai ou près du lieu choisi. Votre construction, circuit, réserve et cargaison sont conservés. Le câble est libéré, le voyage est sauvegardé et le dernier quai de secours ne change qu'à l'amarrage. Un navire trop grand pour l'arrivée est refusé sans déplacer le joueur. Échap ferme le menu.

Les sauvegardes V1/V2 sont migrées au chargement vers le format V3. Les anciens lecteurs refusent les sauvegardes V3. Les nouveaux archipels, la soute, les connexions, les quantités des réservoirs, les fractions de débit et les fuites persistent avec la coque. L'export de construction conserve le plan des connexions, sans exporter la réserve ou la progression.

Au quai, `M → Atelier des circuits d'Aether` ou `Tab → Distribution d'Aether` ouvre la distribution. L'installation est explicite et raccorde les équipements déjà présents ; un nouvel équipement reste isolé jusqu'à son raccordement. Choisissez deux équipements pour ajouter une liaison. Les boutons de vannes ouvrent/ferment réellement leurs branches. Une branche sans réservoir rempli ne produit aucune poussée ou sustentation. Le panneau permet aussi de réparer un réservoir fuyant. Ctrl+Z/Y et les boutons Annuler/Rétablir changent le plan, sans restituer l'énergie consommée. Ctrl+S et Sauvegarder enregistrent la session V3. Les tuyaux routés en 3D et les pompes jouables restent à développer.

Ctrl+S et l'autosave toutes les 30 secondes enregistrent la session. L'amarrage et le retour au menu déclenchent aussi une sauvegarde. Attendez le message « Sauvegarde enregistrée ». La fermeture normale Windows attend la dernière écriture ; une erreur maintient la fenêtre ouverte et explique comment quitter. Une fermeture forcée ou d'onglet peut interrompre l'écriture courante : la précédente version valide reste le recours.

Windows utilise le répertoire de données utilisateur fourni par `directories::ProjectDirs` pour `AetherIsles/Reboot`, généralement `%LOCALAPPDATA%\AetherIsles\Reboot\data`. `AETHER_DATA` peut le remplacer. Le navigateur utilise IndexedDB, par origine du site. Effacer les données du navigateur efface ces sauvegardes. L'export JSON de l'atelier permet de conserver la construction indépendamment ; il ne contient pas le carburant, la position ou la progression.

Le chargement tente la copie de secours si la principale est invalide. Un fichier incompatible est refusé sans remplacer la session. Le secours R remet le navire principal au dernier quai avec une réserve pleine, conserve sa construction et abandonne les fragments secondaires. La marche tombée loin du pont revient au poste. Les préférences de volume et de caméra restent séparées de la partie.

Les dix-huit commandes de déplacement et d'action se personnalisent dans Préférences : cliquez sur l'action, puis appuyez sur une lettre A–Z, Maj, une flèche, Espace ou Tab. Maj gauche et Maj droite déclenchent la même commande. Une touche réservée ou non prise en charge affiche un message et laisse l'attribution ouverte ; Échap l'annule. F8 ne masque pas l'écran pendant cette saisie.

La commande sélectionnée reste en surbrillance après l'attribution. Les boutons et le focus restent en place pendant la saisie et lors d'un conflit.

Si la touche appartient déjà à une autre action, la précédente attribution reste intacte. Cliquez sur « Échanger avec… » ou appuyez sur Entrée pour confirmer l'échange des touches principales ; l'ensemble est refusé sans modification si une troisième commande entrerait en conflit. « Rétablir les touches » remet les raccourcis par défaut. Les choix sont conservés après redémarrage. Échap, Entrée, F3, F6, F8 et les raccourcis Ctrl restent réservés pour les raccourcis de jeu.

Dans les menus, Tab ou les flèches sélectionnent un bouton, puis Entrée/Espace l'active. Pendant le jeu, F6 active cette navigation et interrompt les intentions de pilotage ; Échap ou F6 en sort. Le bouton sélectionné porte un contour clair. F8 masque/rétablit l'interface pour les captures. La simulation continue ; Échap peut la mettre en pause avant la photo. Le bouton du navigateur permet le plein écran.

Les huit conseils d'apprentissage progressent quand vous réalisez les gestes correspondants. « Rejouer les conseils » dans la pause les remet à zéro, sans effacer la partie.
La réaction de la toile est visuelle : le mât, les attaches et la surface aérodynamique de référence conservent leur rôle. Le vent physique présente des rafales douces ; le gonflement et l'ondulation suivent le vent apparent, donc également la vitesse du navire et l'orientation de la voile. Au quai, la toile reste vivante. Échap fige la simulation et la toile ; « Réduction des mouvements » atténue l'ondulation.
