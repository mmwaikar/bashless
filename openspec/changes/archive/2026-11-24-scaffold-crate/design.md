## Context

See proposal.md — Why. The repo has no code; the architecture was proven in a PoC facade (struct `Bk`, incubation workspace `2-infra/poc-shell`) whose behavior this design ports. Constraints that shape the approach, all PoC-verified:

- deno_task_shell is async and spawns pipe/substitution tasks via tokio; its stream-handle helper panics outside a tokio context, so pipe setup must run inside `block_on`.
- The engine's types are `!Send` (`Rc<dyn ShellCommand>`, `LocalBoxFuture`) — single-threaded execution only.
- Scripts get no env/cwd back-propagation (`EnvChange` is consumed internally); xshell's env map is private, so overrides set on the facade must be mirrored into the plain env map passed to the engine.

## Goals / Non-Goals

**Goals:**
- Library-first crate whose public surface lets the PoC's checks port 1:1 into integration tests.
- Error type at every trust boundary; non-zero script exit codes as values, not errors.
- Zero dependencies beyond the four PoC-validated crates.

**Non-Goals:**
- Runner binary, mock-assertion ergonomics API, CI, crates.io metadata (separate later changes).
- Any script-engine behavior change or version-pin change.

## Decisions

1. **Facade struct `Bashless`** (renamed from the PoC's `Bk`) holds: `xshell::Shell`, `HashMap<OsString, OsString>` env overlay, `HashMap<String, Rc<dyn ShellCommand>>` mock registry, and an owned `tokio::runtime::Runtime` (current-thread, `enable_all`). Alternative considered: building the runtime per `eval` call — rejected for construction cost; the PoC proved one owned runtime works.
2. **Error type hand-rolled**: `pub enum Error { ScriptParse(String), Io(std::io::Error) }` + `pub type Result<T>` with manual `Display`/`std::error::Error` impls. Alternative: `thiserror` — rejected to keep the dependency set at exactly the four PoC crates.
3. **Exit-code mapping**: `eval` returns `Ok(exit_code)`; parse failure returns `Err`. Non-zero script exits are NOT errors — capture-style callers (`eval_captured` → `(code, stdout, stderr)`) must be able to assert on the code. Typed-layer calls keep xshell's own non-zero-equals-error behavior.
4. **Typed-layer access**: public `shell(&self) -> &xshell::Shell` accessor so tests and users can run `cmd!` against the facade's shared shell state; the PoC reached the field directly only because it lived in one file.
5. **`eval_captured` internals** (lifted from the PoC): inside `rt.block_on`, build `ShellState::new`, create `pipe()` pairs, drop the stdin writer (immediate EOF), attach `pipe_to_string_handle` readers, `execute_with_pipes(...).await`, then await both reader handles into `(code, stdout, stderr)`.
6. **File layout**: single `src/lib.rs` for the scaffold — a module split is premature until the mock-ergonomics change; tests live in a single `tests/facade.rs`.

## Risks / Trade-offs

- The facade is `!Send` (it owns `Rc`s) — it cannot be shared across threads. Accepted: matches the engine's design and the CLI/test use case.
- `eval` returning `Ok(non-zero)` differs from the typed layer's error-on-non-zero; the split (scripts = exit-code domain, typed = error domain) must be documented in the API docs to avoid surprise.
- xshell 0.2.7 and deno_task_shell 0.33.3 are both pre-2.0; the engine pin is exact (`=0.33.3`) so upstream churn surfaces as a deliberate change, not a surprise build break.

## Open Questions

None — assumptions (edition 2021, facade name, license field now / LICENSE files at publish) are recorded in the proposal and Decisions above.