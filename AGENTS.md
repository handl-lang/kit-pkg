# kit-pkg

The package kit — `.handl.toml` project discovery and dependency wiring
for the prelude `Kit` (`handl-kit-pkg`).

## Layout

| path | role |
| --- | --- |
| `crates/kit-pkg/` | the `handl-kit-pkg` crate |
| `xtask/` | vendored gate crate (`fmt`, `clippy`, `test`, `doc`, `gate`) |
| `workflows/gate.yaml` | per-push gate — `handl-gates` rust campaign via bupha |

## Contract

- Single-member Cargo workspace; sibling deps are git dependencies
  (`branch = "main"`) resolved from sibling checkouts via the
  workspace-root `[patch]` tables — see the `Cargo.toml` comment.

## Verification

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
```

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
