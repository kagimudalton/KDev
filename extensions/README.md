# KDev Extensions

Extensions are isolated feature packages that can add language tooling, themes, commands, previews, and integrations without changing the core workspace format.

Each extension should declare:

- `id`
- `name`
- `version`
- `engines.kdev`
- contributed commands
- supported languages/file types
- permissions, when required

Extensions must not silently execute external commands or access projects outside their granted scope. Security-sensitive integrations should require explicit user approval.
