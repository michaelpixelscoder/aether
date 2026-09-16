# Voxel material authoring

Each material owns its generation history instead of treating exported images as
anonymous binaries:

- `texture_prompt.md`: canonical root-level generation brief, consumed when
  technical maps are generated.
- `tech_maps.json`: optional workflow paths, namespace, validator, and runtime
  aliases for `scripts/generate_tech_maps <asset-directory>`.
- `source/`: immutable candidates, preparation code, workflow, and inputs.
- `source/generation.json`: generator, revisions, model/resources, parameters,
  map conventions, license notes, and source hashes.
- `source/workflow.api.json`: executable ComfyUI API graph.
- `source/prepare_inputs.py`: deterministic masks and geometry prior shared by
  all derived maps.
- `textures/`: reviewed runtime maps only.
- `previews/`: tiled and channel contact sheets for human review.
- `validation/`: machine-readable measurements and acceptance report.

Borders, seams, fasteners, wear, and material IDs belong in a shared geometry
prior. Normal, height, AO, roughness, and metallic maps are derived from that
prior so details cannot drift between channels.
