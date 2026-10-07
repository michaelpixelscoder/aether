# R66 — conversion radiance uniforme des crêtes natives

Un seul prototype isolé décidé par le root. Il remplace exactement `let optical_profile=.12+1.88*middle;` par `let optical_profile=1.0;` dans le shader R64 figé. Aucun gain compensatoire, nouvelle texture ou auxiliaire. L'image native conserve seule sa hiérarchie entre les trois réseaux ; l'ancienne enveloppe transversale n'impose plus un rapport artificiel entre leurs émissions.

SHA shader candidat : **b76e67027f1b34cf6f9d56192bcdf315ad43810ff32c4e64e075297ab9a16c31**.

Préimage R64 : **0861bdc4197dab68fb10826a6165fa6ef5e325fa52ad6d42331a6b65e30c5cf5**.

NativePNG : **eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3**.

CurrentKTX : **68755a59545f61bc047975e7af6c301bd98164468a6f293a1336e9bdbb5b1f05**.

La couverture, le body associé, l'alpha, les UV, les gradients, le seuil radiance, l'horloge, les phases/seams, les expressions `middle`, `thermal` et `core`, les cascades, les pools et la haze sont byte exact. Le gain14 et le rolloff6 restent exacts. Comme le gain du cœur comporte aussi `optical_profile`, sa luminosité peut changer ; l'expression du masque et son terme cyan associé ne changent pas. Cinq fetches et les ressources GPU existantes sont conservés. Aucun Rust, géométrie ou fichier canonique n'est touché.

## Mesure CPU

`probe.py` importe les fonctions mathématiques de R65 authentifié, sans exécuter son ancien bloc principal. Il compare le même champ natif sur une grille1024×512, crop.42+.29, aux neuf mips0..8. Les domaines de bandes sont séparés aux vallées natives identifiées dans R65, pas par de nouveaux masques du shader. La masse RGB associée de crête avant conversion radiance est **exactement inchangée** partout. La couverture est **exactement inchangée**. Le JSON fournit leurs moyennes RGB/par bande, les radiances HDR avant/après et les mesures du cœur.

Les rapports ci-dessous concernent la luminance de radiance associée **après** gain/rolloff. Ils ne sont pas une prévision de contraste après exposition ou tonemapping.

| Mip | Total | Bande haute | Bande centrale | Bande basse |
|---|---:|---:|---:|---:|
|0|.9023|3.0629|.7166|1.3246|
|1|.9025|3.0956|.7155|1.3258|
|2|.9030|3.1603|.7134|1.3270|
|3|.9044|3.2360|.7100|1.3277|
|4|.9036|3.3327|.7043|1.3244|
|5|.8921|3.2789|.6943|1.3082|
|6|.8409|2.4518|.6681|1.2507|
|7|.7862|2.1559|.6458|1.0081|
|8|.8418|2.2186|.6104|1.2085|

Le total diminue, le réseau central diminue, les deux bandes externes remontent. Aux mips2–3 estimés des grands rubans proches, le réseau central émet environ .71×, la bande haute3.16–3.24× et la basse1.33×. Le shader garde sa borne individuelle6 par canal. Le rapport de radiance par fragment est borné .5..8.334× par les valeurs de l'ancien profil et la monotonie du rolloff. La masse native demeure conservée ; la radiance finale n'est pas conservée, puisque la conversion change explicitement.

Un champ RGB exactement nul a toujours une masse de crête nulle ; le body profond existant reste présent. Les champs natifs très faibles conservent leur pied .006 et leur classification : changer le profil peut modifier leur très faible émission aux banques. Ce n'est pas une exemption du delta radiance. La mesure d'un champ linéaire RGB≤.025 est incluse dans le JSON.

Limites : sampling bilinéaire à mip fixe, sans vraie empreinte anisotrope GPU ni mélange des phases ; pondération UV uniforme plutôt qu'aire écran ; body mesuré à grazing.5 ; le diagnostic du cœur représente son masque complètement engagé U≥18.25, avec terme cyan additif inchangé exclu du rapport. Le résultat natif peut perdre de la lecture centrale. Aucun gain de contraste final ou de performance n'est promis.

## Gate et reconstruction

`verify.mjs` reconstruit indépendamment la chaîne attendue depuis la préimage SHA R64, puis exige l'égalité binaire intégrale. Il n'importe pas le générateur. Vingt changements réservés sont rejetés (gain, couverture, body alpha, horloge, core/middle, crop/offset, seuil, binding, scroll/drift, rolloff, edge/near alpha, cascade, pool, haze, compensation) ; R64 non modifié est aussi rejeté comme candidat R66. Les PNG/KTX courant et waterfall historiques doivent être byte exact. Neuf mips/trois bandes finies et leur delta radiance seul sont vérifiés.

Avec Python+NumPy : exécuter `portable/tools/art/flow-r66-source/build_shader.py`, puis `probe.py`. Avec Node : exécuter `verify.mjs`. Le paquet est autonome sur D ; la reconstruction est testée dans une autre copie D. Les PNG/KTX sont copiés depuis R64, sans nouveau calcul de pixels ou mips. Le PNG1254² et ses11mips conservent le contrat R64. La galerie doit utiliser le diagnostic `--current-texture textures/world-flow-current-r64.ktx2`, avec séparation current-only déjà implémentée par le root ; l'ancienne texture `world-flow.ktx2` reste réservée aux cascades/pools.

Aucun rendu GPU ou validation de compilation native dans cette étude CPU. Le root capturera le seul candidat après ses travaux navigateur. R64/R65 historiques sont conservés. Aucun Git.
