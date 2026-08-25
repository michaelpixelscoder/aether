# Thoughts

This directory is the workspace for exploration, brainstorming, hypotheses, and possible solutions.

A thought does not need to be correct or complete. Its purpose is to make an idea explicit, examine it, connect supporting material, and decide whether it deserves further investigation.

## Structure

Give every thought its own directory:

```text
thoughts/
└── thought-name/
    ├── thought-name.thought.md
    └── supporting-files...
```

The directory can contain images, references, examples, prototypes, data, or scripts used during the exploration. Keep the main narrative and links to supporting material in `thought-name.explore.md`.

Copy `_template/` when starting a thought, then rename both the directory and main file. Use a short lowercase kebab-case name.

## What belongs here

- A problem that may be worth solving
- A possible service, product, or workflow
- A business or pricing hypothesis
- Several possible interpretations of an observation
- Research and experiments used to assess an idea
- Open questions that require interviews or evidence

Do not present assumptions as facts. If the work begins with a real event or observation, record that evidence in `stories/` and link to it.

## Lifecycle

Use the `status` field to show where an exploration stands:

- `seed`: captured but not yet explored
- `exploring`: actively being researched or developed
- `testing`: being validated with evidence or a real experiment
- `validated`: supported strongly enough to inform action
- `parked`: worth retaining but not active
- `rejected`: evidence does not support pursuing it

Changing direction is expected. Preserve rejected thoughts and document why they were rejected; that learning can prevent repeated work.
