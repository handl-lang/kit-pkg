# kit-pkg

The package kit — `.handl.toml` project discovery and dependency wiring for the prelude `Kit` (`handl-kit-pkg`).

## Layout

```text
crates/kit-pkg/    the handl-kit-pkg crate
```

## Development

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
```

`cargo xtask` wraps the repo's `xtask/` crate; `workflows/gate.yaml` is the gate entry. Sibling deps are git
dependencies (`branch = "main"`); the workspace-root `[patch]` tables resolve
them from sibling checkouts for local development.

## License

MIT.
