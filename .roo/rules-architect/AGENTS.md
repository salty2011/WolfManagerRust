# Project Architecture Rules (Non-Obvious Only)

- The repository does not yet contain a Rust workspace (no [Cargo.toml](Cargo.toml)). Architectural constraints are not codified in manifests; introduce them by committing workspace files and crate boundaries rather than documenting them ad hoc.
- Dev environment assumptions should be encoded in [".devcontainer/devcontainer.json"](.devcontainer/devcontainer.json:1); extend this file when adding toolchain or service dependencies so agents inherit the same environment.