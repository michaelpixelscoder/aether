# Textures originales du reboot

Méthode : outil ImageGen intégré, sans CLI ni API externe. Les originaux restent dans le répertoire de génération de Codex. Copies utilisées : `assets/textures/cedar.png`, `cedar-normal.png`, `indigo-canvas.png`, `sky-panorama-v2.png`. Aucun redimensionnement ni retouche d'image par script. La carte normale est une interprétation générée de l'albedo, pas un scan ni une mesure physique.

## Cèdre — génération

Single square seamless tileable PBR diffuse base color texture of dark warm weathered cedar ship planking for a high quality voxel fantasy airship. Orthographic flat surface fills entire image. Four horizontal rows of long timber planks, narrow dark recessed seams (1 percent plank height), staggered end joints, subtle warm chocolate brown grain and small restrained knots, slightly worn edges, realistic timber craftsmanship. Average color dark warm brown sRGB #60391f. Even neutral illumination, NO baked shadows, no gloss highlights. No metal, no nails, no brass, no ornament, no borders, no frames, no objects, no text, no watermarks. Four horizontal plank courses across the square. Restrained clean high fidelity material, not noisy/grungy, not painterly. Identical seamless tiling along all four edges. 1024x1024.

## Normale — édition du cèdre ci-dessus

Convert this exact 4-course cedar planking diffuse texture into a precise tangent-space OpenGL normal map for a game renderer. Preserve same square resolution, exact positions of the FOUR horizontal rows of planks, staggered end seams and all grain. Flat surface RGB (128,128,255), X red, +Y green. Very restrained shallow fine grain normal detail, small bevel only along seams, uniform blue neutral plank interiors. Absolutely no brown color or original lighting, no labels. Output only the seamless blue purple tangent-space normal map, no diffuse preview, no split view.

## Toile — génération

Create a single seamless tileable PBR base color texture for a premium stylized realistic voxel airship game: old indigo navy blue woven sailcloth. Square 1024 x 1024 image, orthographic perfectly flat surface, fills the entire image edge to edge. Extremely subtle visible fine linen weave and a little natural irregular fiber detail; deep rich midnight navy dyed textile, uniform average color approximately sRGB #142a50, no highlight or shadow gradient. Diffuse albedo only under neutral even illumination. No seams, no stitches, no folds, no border, no objects, no emblem, no text, no watermark. Fine authentic textile with restrained clean detail, not grungy/noisy. All edges tile seamlessly. This will be mapped repeatedly onto a modeled sail with separate geometric gold compass embroidery.

## Premier ciel, conservé dans les essais

Panorama original de ciel bleu, nuages blancs et éclairage chaud, sans îles ni vaisseaux peints. La sphère de ciel utilise un mélange étroit aux longitudes extrêmes pour atténuer le raccord du panorama ; les îles restent des objets du jeu. Le prompt détaillé de cette première génération n'a pas été conservé dans le fichier auteur ; cette description ne le remplace pas comme trace exacte.

## Ciel du socle 0.1 — génération, panorama 1774×887 obtenu

Create a production game sky texture, a full 360-degree equirectangular panorama with exact 2:1 width-to-height composition, highest available resolution. Aether Isles: open luminous blue heavens above a sea of clouds, warm late-afternoon sunlight from the right, cool blue violet cloud shadows, convincing finely detailed cumulus clouds. The horizon stays clear and open around the full panorama. Upper half predominantly saturated azure blue with a few thin high clouds. Small-to-medium bright cumulus banks cluster toward the LOWER quarter; bottom contains a continuous distant soft cloud sea, not a single huge foreground cloud. Avoid giant clouds dominating the mid or upper hemisphere. Clean physically plausible in-engine skybox texture, clear edges and delicate shading, not painterly and not blurry. Uniform exposure, rich color, no pure-white clipping. Left and right edges must join naturally and match. Full image is only sky and clouds: no islands, rocks, ships, trees, birds, planets, text, symbols, water, ground or UI. Wide 2:1 panorama format.

Original : `exec-f58763c4-ca9f-43ce-94af-06bff03740a8.png`, créé avec l'outil intégré. Copie inchangée dans `assets/textures/sky-panorama-v2.png`. Le runtime ajuste seulement les UV, l'exposition et le raccord de longitude dans son shader.

## Pierre taillée r11 — génération intégrée

Use case: stylized-concept. Asset type: seamless tileable base-color texture for pale carved limestone architecture in a detailed voxel fantasy game. Generate a square 1024x1024 flat orthographic material swatch filling the whole canvas: softly warm ivory limestone, large quiet stone surfaces with subtle broad cloudy beige mineral variations and restrained shallow worn chips. This is a neutral albedo texture, evenly illuminated with no directional lighting, no cast shadows, no perspective. Very low contrast, light warm grey ivory. No brick mortar lines, no cracks spanning the tile, no black pores, no speckled grain, no noisy microtexture, no text, no objects, no border. The texture will repeat across carved columns, cornices and dressed blocks, and must remain visually calm at a distance. Preserve some gentle natural stone variation rather than an absolutely uniform fill.

Original : `exec-f4efd8dc-45b3-40c9-b80a-9dd3b93105b5.png`, créé avec ImageGen intégré. Copie inchangée dans `assets/textures/dressed-limestone-r11.png`. L'image finale fournie par l'outil est 1254 × 1254; aucun redimensionnement ni retouche par script. Facteurs linéaires du générateur : roche 0,32 × palette, strates 0,23 ×, maçonnerie 0,40 ×, corniche 0,65 ×.

## Calcaire fracturé du monde — génération

Production game texture, a single square seamlessly tileable albedo texture of aged grey-blue limestone cliff stone for the monumental floating islands of Aether Isles. Flat orthographic material scan, no perspective, no scene, no objects. Natural fractured horizontal mineral strata with small angular fissures, weathered rough granular surfaces, subtle warm ivory mineral edges and a few desaturated moss stains in the cracks. Broad quiet areas alternating with fine cracks. Physically plausible high quality rock surface, crisp texture detail but no noisy grain filter, no baked directional light, no shadows, no highlights, no ambient occlusion, no vignette, no text, no border. Neutral mid-grey overall, low to medium contrast, avoid black crevices or white clipping. This will repeat across huge sculpted voxel cliffs with real geometry and PBR lighting. Square 2048x2048, seamless on all four borders.

Original ImageGen `exec-e0aa3b27-2dc9-429e-9e7f-d582fbebfab0.png`, dans le dossier de génération `01a0f81f-371e-7440-ba13-362c122d50e5`. Copie inchangée `assets/textures/limestone-world.png`. Empreinte SHA256 : `a8d1095cb49df8271937f7f87e55b32d05980f696c8804df5fbe63c5d1a70510`.

## Panorama du monde — génération ImageGen de travail

Create a production game environment texture for Aether Isles: an extremely high quality cinematic 360-degree equirectangular panoramic skybox, 2:1 aspect ratio, ideally 4096x2048. SKY AND CLOUDS ONLY. Entire frame is sky, no islands, no buildings, no mountains, no water, no text. Rich deep cobalt blue open sky above an immense sea of luminous voluminous white cumulonimbus clouds. Large detailed dramatic cumulus towers at middle latitudes, sculptural cloud banks extending up both sides; sun breaking through upper left with warm pale gold silver linings, blue shadow interiors. Maintain large blue opening in central upper area. Bottom third entirely fills with fluffy cloud sea seen from above. Crisp fine cloud detail, photorealistic AAA sky texture, realistic atmosphere and exposure, rich color and contrast, absolutely no flat washed-out grey-blue haze. Horizon near 60% frame height. Seamlessly continuous left/right edges. Zenith uncluttered blue. No baked lens flare or bloom filter. This texture will be shown behind modeled 3D voxel sky islands, so do not paint any solid objects or silhouettes.

Original ImageGen `exec-40134893-9478-44a5-9411-8e69ca0b0140.png`, même dossier. Résultat réellement obtenu : 1774×887, conservé sans agrandissement dans `assets/textures/sky-world.png`. Le prompt demandant une plus grande taille ne constitue pas une preuve de cette résolution.
