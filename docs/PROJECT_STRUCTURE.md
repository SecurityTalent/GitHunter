# Project Structure

GITHUNTER uses a small, layered Rust layout so each concern has a stable home.

```text
githunter/
├── Cargo.toml                 # package manifest and dependencies
├── README.md                  # setup and supported workflow
├── install.sh                 # one-line installer script
├── LICENSE                    # Apache-2.0 license
├── docs/
│   ├── architecture.md         # architecture decisions
│   ├── cli-design.md           # command hierarchy and output contracts
│   ├── data-model.md           # persisted entities and relationships
│   ├── project-plan.md         # product specification and priorities
│   ├── project-track.md        # implementation history and next milestone
│   ├── roadmap.md              # delivery phases
│   ├── security.md             # local-first safety model
│   └── PROJECT_STRUCTURE.md    # this guide
├── src/
│   ├── main.rs                 # executable entry point
│   ├── cli/                    # argument parsing and command dispatch
│   │   ├── mod.rs
│   │   └── init.rs
│   ├── application/            # use-case orchestration
│   │   ├── mod.rs
│   │   └── research_state.rs
│   ├── domain/                 # pure validation, normalization, and tool models
│   │   ├── mod.rs
│   │   ├── asset.rs
│   │   └── tool.rs
│   ├── database/               # SQLite schema, migrations (WAL mode)
│   │   └── mod.rs
│   └── repository/             # .githunter creation, discovery, and file persistence
│       └── mod.rs
└── tests/
    ├── init.rs                 # repository initialization behaviour
    ├── research_state.rs       # target → scope → asset → snapshot → diff flow
    ├── scope_workflow.rs       # bulk scope files, comments, deduplication, precedence
    ├── asset_ingestion.rs      # mixed asset types, single add, stdin, multi-source provenance
    ├── tools_workflow.rs       # tool config, validation, execution, workflow automation
    └── recommend_and_completion.rs # advisory recommendations and shell completion
```

Generated build files live in `target/` and are intentionally excluded from
version control. A user's local research data lives in `.githunter/`, also ignored.

## Naming conventions

- Directories and Rust modules use lowercase `snake_case`.
- Rust types use `PascalCase`; functions and variables use `snake_case`.
- Markdown design documents use lowercase kebab-case, except this navigational
  file which is uppercase for easy discovery in file explorers.
- Test files describe a user-visible workflow or capability.
