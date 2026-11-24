## Why

The repository has no code yet — the two-layer architecture (xshell typed layer + deno_task_shell script engine behind one facade) is proven only in a throwaway PoC in the incubation workspace. Everything that follows — the mock/assert differentiator API, CI with a Windows leg, the runner crate — needs a real library crate with a proper error type and tests as its foundation, so the scaffold is the foundation change.

## What Changes

- New library crate `bashless` at the repo root (edition 2021, `license = "MIT OR Apache-2.0"`).
- Dependencies pinned to the exact PoC-validated set: xshell 0.2.7, deno_task_shell =0.33.3, futures 0.3, tokio 1 (rt).
- `src/lib.rs`: facade struct `Bashless` (renamed from the PoC's `Bk`) bundling the typed shell, the env-overlay mirror, the mock-command registry, and a tokio current-thread runtime; methods `new`, `set_var`, `eval`, `eval_captured` with PoC-verified behavior.
- Crate error type at every trust boundary (script parse, spawn failure): `Result` returns, no `unwrap`/`expect` in library code. Non-zero script exit codes remain values (`Ok(code)`), not errors — capture-style callers need the code to assert on.
- Integration tests in `tests/` porting the PoC's four checks: typed `cmd!` echo, shared-state eval (`$WHO`, `$(ls | wc -l)`), pipeline eval (`echo c b a | tr | sort`), and mocked `git` with captured output.
- README stub: what/why, one example, status = scaffold.

Layers touched: **typed** and **mock-capture** (facade construction only — no script-engine behavior or runner changes; deno_task_shell stays pinned =0.33.3).

## Non-goals

- Full-bash compatibility (brush's niche) — permanent non-goal.
- Interactive-shell features — permanent non-goal.
- Runner binary — deferred until the library API settles.
- Env/cwd back-propagation from scripts — PoC-verified engine behavior; explicit `set_var` is the supported path.
- Publishing to crates.io — after CI and LICENSE files exist.

## Capabilities

### New Capabilities
- `library-facade`: the public facade — construction and shared state (typed shell, env overlay, mock registry, tokio runtime), typed command execution, script evaluation over the bash-subset engine, output capture, and command mocking that resolves before PATH.

### Modified Capabilities

(none — this is the first capability; no specs exist under `openspec/specs/` yet)

## Impact

- New files only: `Cargo.toml`, `src/lib.rs`, `tests/`, `README.md` — nothing existing breaks.
- The public API surface is born here: `Bashless`, crate `Error` + `Result` alias (hand-rolled `std::error::Error` impl — no additional dependencies), mock registration via the `ShellCommand` trait re-exported from deno_task_shell.
- Constraints carried into implementation (PoC-verified): `!Send` stack (`Rc`, `LocalBoxFuture`) — single-threaded by design; `pipe()` handles must be created inside `block_on`; env overrides mirrored in a HashMap because xshell's env map is private.
- Assumptions recorded: edition 2021 (PoC parity); facade named `Bashless`; license field set now, LICENSE files added at publish time.
