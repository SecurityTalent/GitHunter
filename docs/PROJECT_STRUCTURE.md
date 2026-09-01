# Project Structure

GITHUNTER uses a small, layered Rust layout so each concern has a stable home.

```text
githunter/
|- Cargo.toml                  # package manifest and dependencies
|- README.md                   # setup and supported workflow
|- LICENSE                     # Apache-2.0 license
|- docs/
|  |- architecture.md          # architecture decisions
|  |- cli-design.md            # command hierarchy and output contracts
|  |- data-model.md            # persisted entities and relationships
|  |- project-plan.md          # product specification and priorities
|  |- project-track.md         # implementation history and next milestone
|  |- roadmap.md               # delivery phases
|  |- security.md              # local-first safety model
|  `- PROJECT_STRUCTURE.md     # this guide
|- src/
|  |- main.rs                  # executable entry point
|  |- cli/                     # argument parsing and command dispatch
|  |- application/             # use-case orchestration
|  |- domain/                  # validation, normalization, and tool models
|  |- database/                # SQLite schema and migrations
|  `- repository/              # .githunter discovery and persistence
`- tests/
   |- init.rs                  # repository initialization behaviour
   |- research_state.rs        # target-to-diff workflow
   |- scope_workflow.rs        # scope imports and precedence
   |- asset_ingestion.rs       # asset ingestion and provenance
   |- tools_workflow.rs        # tool configuration and execution
   `- recommend_and_completion.rs # advice and shell completions
```

Generated build files live in `target/` and are intentionally excluded from
version control. A user's local research data lives in `.githunter/`, also ignored.

## Naming conventions

- Directories and Rust modules use lowercase `snake_case`.
- Rust types use `PascalCase`; functions and variables use `snake_case`.
- Markdown design documents use lowercase kebab-case, except this navigational
  file which is uppercase for easy discovery in file explorers.
- Test files describe a user-visible workflow or capability.
