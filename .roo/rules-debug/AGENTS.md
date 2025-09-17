# Project Debug Rules (Non-Obvious Only)

- The only automated setup occurs during Dev Container initialization (global npm install of @anthropic-ai/claude-code) [".devcontainer/devcontainer.json"](.devcontainer/devcontainer.json:34). If the CLI is missing, rebuild/reopen the container so the postCreate step re-runs.
- No additional logging, debug flags, or hidden output channels are defined in the repository at this time.