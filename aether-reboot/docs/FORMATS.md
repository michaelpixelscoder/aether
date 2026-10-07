# Formats de données

Les fichiers sont UTF-8 JSON. La version courante est 1. Tout fichier de plus de 16 Mio, ID invalide ou répété, valeur non finie, corps hors limites ou référence absente est refusé avant remplacement de la partie.

Les IDs persistants sont des entiers positifs, au plus `u64::MAX − 1024`, uniques dans leur portée : flotte pour les corps, construction pour les composants. Ils restent distincts des entités Bevy. Créer un objet réserve un entier libre sans renuméroter ceux qui existent ; un fichier importé portant un très grand ID reste donc éditable. Réutiliser un ID libéré n'introduit pas deux objets simultanés de même identité, et les historiques conservent leurs snapshots complets.

## Blueprint

L'enveloppe contient `version`, `cell_size` égal à 0,5 et `blueprint`. Celui-ci contient un nom de 1 à 64 caractères, `cells` comme couples adresse/matériau, et `parts`. Les matériaux sont `Wood`, `Metal`, `Glass`. L'air est implicite, jamais une cellule exportée. L'adresse `[x,y,z]` désigne le centre local en unités de cellule. Y est vertical ; l'avant de la coque est −Z.

Un composant contient `id`, `kind`, `cell` de support et `quarter_turn` entre 0 et 3. Les types sont `Helm`, `Sail`, `Tank`, `Lift`, `Harpoon`. La rotation est autour de +Y par multiples de π/2. Les empreintes et propriétés sont issues du catalogue de cette version. Le blueprint n'inclut ni carburant, ni pose, ni identifiant de renderer.

Les cellules s'exportent dans un ordre stable. Les composants sont triés par ID. Les dimensions maximales sont 128 par axe, et le nombre de cellules est compris entre 1 et 10 000. Les coordonnées individuelles sont bornées à ±16 384 avant allocation. Les imports ne recentrent pas la construction.

## Session

L'enveloppe contient `version`, `seed`, `tick`, `vessels`, `active`, `checkpoint`, `progress`, `tether` et `walker`. Chaque vaisseau contient son ID, son blueprint, une position en mètres, un quaternion normalisé, ses vitesses linéaire et angulaire, `fuel`, `trim`, `docked` et `target_altitude`. Les limites de pose sont ±10 000 m, 150 m/s et 20 rad/s ; ce sont des limites de validation, pas des vitesses recommandées de jeu.

`target_altitude` est optionnel pour compatibilité avec les premières v1 : son absence reprend l'altitude de la pose, bornée à 3–65 m. Une nouvelle écriture le renseigne. L'absence historique de `walker` signifie pilotage. Les autres champs requis manquants et les versions inconnues échouent explicitement. Les champs inconnus ne sont pas silencieusement jetés.

Le câble contient l'ID du corps, l'index d'ancre 0–2, le point local d'un harpon existant et une longueur de 0,5–80 m. Il n'est pas accepté sur un navire amarré. Le personnage en marche conserve position et vitesse. Les caches de contacts, handles et intentions de la dernière image ne sont jamais sérialisés.

La session est écrite sans indentation pour tenir la flotte complète sous le plafond. Les préférences sont un document séparé avec valeurs bornées ; une préférence absente reprend sa valeur par défaut.

## Compatibilité et reprise

Les exemples dans `examples-data` sont de vrais exports des codecs. Le test `saves_preserve_altitude_setpoint_and_accept_early_v1` protège la reprise de la première forme v1. Les fixtures de round-trip couvrent pose tournée et origine asymétrique. Pour une future v2, ajouter une migration explicite et son test ; ne jamais charger par défaut une version inconnue en supprimant les champs qui gênent.

Le backup contient la version valide précédente. L'import d'un blueprint remplace la construction au quai en conservant au plus la réserve déjà présente et la capacité restante. Une nouvelle partie est la seule création normale d'une réserve pleine ; le secours est une règle de récupération volontaire.
