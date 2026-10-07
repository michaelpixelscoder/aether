# Conduites — fondation R72 et intégration restante

Le réseau logique V3 est jouable dans l'atelier. Le module `core::conduit_route` prépare la recherche d'un passage pour des conduites visibles. Il n'est appelé ni par l'atelier ni par la simulation. Les modèles industriels R69 et la recherche R72 restent séparés : aucun tuyau posé, aucune pompe commandable et aucun nouveau collider ne sont annoncés.

## Contrat actuellement vérifié

La recherche travaille dans le repère local d'une coque validée, sur une grille de125 mm. Les cellules occupées de500 mm et les empreintes réelles des équipements deviennent des obstacles fermés. Le voisinage testé est borné à27 cellules. Une marge de280 mm couvre l'enveloppe transversale des modules R69, notamment le volant de vanne, dont le coin extrême est à258 mm. Elle ne constitue pas un collider plein remplaçant le passage creux des tuyaux. La longueur du tube droit est couverte par le parcours de son axe.

Les faces des obstacles se trouvent sur des multiples de125 mm. Sur chaque pas cardinal de125 mm, la distance à une boîte ne peut donc avoir un minimum strict entre ses extrémités : l'intervalle projeté est soit monotone, soit constant. Ce contrat dépend de cette grille et de ces boîtes ; il ne couvre pas une future géométrie libre ou des colliders obliques. Un test indépendant échantillonne également les segments trouvés tous les5 mm contre les vraies boîtes, sans réutiliser le prédicat de recherche.

Chaque état comprend position, direction d'approche et distance depuis le dernier coude. Retenir seulement la position effacerait des approches valides. Les raccords sont cardinaux, sans demi-tour, avec500 mm droits aux extrémités et entre deux coudes. Le coût entier vaut10 par pas et8 par virage ; l'heuristique Manhattan×10 reste admissible. Le départ va dans l'axe sortant, l'arrivée contre l'axe sortant du raccord cible.

Un appel effectue au plus256 extractions de file, même si elles sont périmées. Plafonds :16384 états, six fois ce nombre dans la file,65536 obstacles,1025 points avant compression et une enveloppe de80 m par axe. Détruire le job annule la recherche sans modifier construction ni réseau. `Limit` distingue l'épuisement d'un budget de l'absence de chemin. La sortie contient les extrémités, les virages et la longueur de la polyligne ; cette longueur n'est pas celle des arcs d'un assemblage final.

Les six tests vérifient parcours direct, déterminisme entre lots de travail, contournement d'une vraie coque, empreinte et dégagement d'un réservoir réel, coordonnées et budgets refusés, axe sortant bloqué et limites terminales. Ils passent dans le workspace complet de158 tests. Ils ne prouvent pas des raccordements en jeu.

## Ordre d'intégration et critères

1. Rédiger les raccords des réservoirs, sustentateurs et hélices à partir des modèles réellement installés : position, axe, diamètre, pose relative et ID d'équipement. Vérifier chaque quart de tour.
2. Définir le court passage autorisé dans le volume de l'équipement propriétaire. Aucun obstacle étranger ne doit être exempté pour libérer un raccord.
3. Ajouter des clés persistantes de raccord distinctes des entités Bevy et valider leur résolution après rechargement.
4. Définir un plan de conduite attaché à une arête logique : raccords, segments, modules et révision de construction.
5. Invalider une recherche si la construction, le raccord ou le plan change ; afficher un aperçu sans modifier la réserve ni les cellules.
6. Indexer aussi les conduites déjà posées, avec une règle explicite pour les croisements, supports et raccords partagés.
7. Convertir les lignes et virages en tubes, coudes et joints R69, en respectant leurs vrais ports et leurs longueurs ; vérifier les contacts par géométrie.
8. Réserver l'espace d'une vanne complète avant confirmation, y compris son volant. Écarter les placements qui traversent pont, fenêtre, équipement ou autre conduite.
9. Valider la totalité du plan avant une seule transaction de pose ; un refus ne débite aucune matière et ne modifie pas la topologie.
10. Définir coût, masse et inertie des conduites. Les attribuer à la coque autoritaire ; les meshes de présentation ne portent pas ces données.
11. Charger le kit depuis les actifs canoniques seulement après revue et reconstruction portable ; instancier à l'échelle métrique, jamais à celle du studio d'auteur.
12. Synchroniser l'ouverture de la vanne et son rotor avec l'état logique sauvegardé ; aucune animation ne doit changer seule le débit.
13. Ajouter des collisions adaptées aux passages creux et à la navigation sur le pont. Vérifier que leurs enveloppes correspondent au modèle visible.
14. Étendre le codec de façon versionnée, avec limites de segments/octets et migration V3 sans inventer des tuyaux sur les anciens graphes.
15. Raccorder annulation, rétablissement, suppression d'équipement et scission. Conserver exactement la quantité courante ; couper ou réattribuer les conduites par IDs réels.
16. Vérifier par actions réelles Windows/Chrome/Edge : pose, refus, vanne, découpe, sauvegarde, reprise et passage du personnage. Ajouter un trajet chargé pour mesurer le coût des recherches et du rendu.

Ces étapes sont autorisées dans le périmètre demandé et restent à développer. Les pompes, transferts entre navires et causes jouables de dommages nécessitent ensuite leurs propres contrats et vérifications. Voir les planches et le périmètre [MONDE-OUVERT](MONDE-OUVERT.md).

La méthode de recherche s'appuie sur [Red Blob Games, implémentation d'A*](https://www.redblobgames.com/pathfinding/a-star/implementation.html). La grille3D, les contraintes de raccordement et les budgets ci-dessus sont des choix du projet, pas des performances empruntées à cette référence.
