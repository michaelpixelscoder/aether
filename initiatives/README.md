# Initiatives

An initiative is a direction Move Media has chosen to pursue through active work an$d one or more iterations.

Examples include launching the first community website, preparing for September, or improving project tracking. An initiative may last a few days or continue for months. Unlike a thought, it represents a commitment to act. Unlike a story, it describes work in development rather than an event that has already happened.

Every initiative must begin with a precise problem and observable facts that will determine whether the work is complete. Activity alone is not success.

## Structure

Give every initiative its own directory:

```text
initiatives/
└── initiative-name/
    └── initiative-name.initiative.md
```

Copy `_template/` when starting an initiative, then rename both the directory and main file. Use a short lowercase kebab-case name.

This structure intentionally defines only the initiative specification. Where tasks, test scripts, release records, and other project material should live will be decided separately when their requirements are better understood.

## Required progression

An initiative should move through these stages:

1. **Define the problem.** State who experiences it, the current situation, and the evidence that justifies working on it.
2. **Define completion.** List measurable facts that would prove the intended outcome has been reached, plus how and when they will be measured.
3. **Set stop conditions.** Decide in advance when the initiative should be paused, discarded, or reconsidered—for example a missed deadline, exceeded budget, or lack of measurable progress.
4. **Explore solutions.** Compare plausible approaches, including a minimal option and the option of taking no action.
5. **Select a solution.** Record the decision, its rationale, constraints, and explicitly excluded scope.
6. **Plan implementation.** Break the chosen solution into ordered milestones and tasks with clear completion checks.
7. **Define tests.** Prefer deterministic instructions that an AI agent can run from the command line and that return an unambiguous result.
8. **Release and observe.** Deliver a usable increment, collect measurements and feedback, then decide what correction is required.
9. **Close or continue.** Compare results with the completion and stop conditions. Complete, iterate, pause, or discard the initiative based on evidence.

Do not select a solution before the problem and completion measures are clear enough to judge it.

## Status values

Use one of these values in the `status` field:

- `proposed`: problem and intended outcome are being defined
- `evaluating`: solutions, feasibility, and stop conditions are being assessed
- `planned`: a solution and implementation plan have been selected
- `active`: implementation or an observation cycle is underway
- `blocked`: progress currently depends on an unresolved condition
- `paused`: intentionally inactive but may resume
- `completed`: measurable completion criteria have been met
- `discarded`: a stop condition was met or evidence no longer supports continuing

## Tasks

The implementation plan may identify or link to multiple tasks, but this directory does not define where those tasks live. Each referenced task should make clear:

- The result it must produce
- Relevant dependencies or constraints
- Its current status
- How completion will be verified

Tasks explain delivery work; they do not replace the initiative's outcome measures.

## Release, observation, and correction cycles

Summarize each cycle in the initiative specification. A useful cycle contains:

- What was released and to whom
- What was expected
- How long the observation period lasted
- Quantitative measurements and qualitative feedback
- Unexpected effects or failures
- The correction or next decision

Preserve unsuccessful cycles. They are part of the evidence used to assess the initiative.

## Test instructions

Tests should be repeatable by someone without hidden context. When possible:

- Give commands that run from the repository root
- State prerequisites and required fixtures
- Avoid steps that depend only on visual judgment
- Define the expected exit code or output
- Separate automated checks from human acceptance checks
- Never place credentials or production secrets in instructions or scripts

If an important result cannot be tested automatically—such as customer adoption—define exactly where its data comes from, the measurement window, and the threshold that passes.

## Closing an initiative

An initiative is not completed merely because every task is checked off. Compare actual measurements with the original completion criteria.

When closing an initiative, record:

- The final status and date
- Results against each completion measure
- Resources used compared with constraints
- What was learned
- Follow-up thoughts, stories, or initiatives
- Why the initiative was completed or discarded
