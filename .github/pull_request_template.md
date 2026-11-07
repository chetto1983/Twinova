<!-- Keep PRs focused: one concern per PR. Template from chetto1983/ModbusTool. -->

## What & why

<!-- What does this change do, and why? Link the issue or the spec in docs/superpowers/specs/. -->

Closes #

## Type

- [ ] feat — new capability
- [ ] fix — bug fix
- [ ] refactor / perf
- [ ] docs
- [ ] test
- [ ] chore / ci / build

## Checklist

- [ ] `npm run check && npm run lint && npm run format:check && npm run coverage` pass (in `app/`)
- [ ] `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` pass (in `app/src-tauri/`)
- [ ] Coverage ≥ 85% (Rust and frontend), no test skipped to hide a failure
- [ ] No file exceeds 600 LOC (`bash scripts/check-file-size.sh`); touched files refactored on touch
- [ ] Engine or scene changes checked end to end on the real app (`npm run e2e`) — or N/A
- [ ] Spec / `CHANGELOG.md` updated if behaviour or interfaces changed, per `CLAUDE.md`; dated verification file written
- [ ] UI changes use the Aura tokens only (no hard-coded colours)

## Notes for reviewers

<!-- Anything non-obvious: trade-offs, follow-ups, areas wanting extra scrutiny. -->
