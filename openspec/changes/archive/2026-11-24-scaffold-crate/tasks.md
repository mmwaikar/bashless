## 1. Crate setup

- [x] 1.1 Create `Cargo.toml` (package `bashless`, edition 2021, `license = "MIT OR Apache-2.0"`; dependencies: xshell 0.2.7, deno_task_shell =0.33.3, futures 0.3, tokio 1 with rt feature) plus a placeholder `src/lib.rs`, and verify `cargo build` succeeds.
- [x] 1.2 Create `README.md` stub (what/why, one usage example, status: scaffold) and verify the content matches this change's proposal.

## 2. Facade implementation

- [x] 2.1 Implement `Error` (ScriptParse / Io variants) and the `Result` alias with hand-rolled `Display`/`std::error::Error` impls in `src/lib.rs`, and verify `cargo build` passes and `grep -rn "unwrap()\|expect(" src/` finds nothing.
- [x] 2.2 Implement the `Bashless` facade (typed Shell + env overlay + mock registry + owned current-thread tokio runtime) with `new`, `set_var`, and the `shell()` accessor, and verify `cargo build` passes.
- [x] 2.3 Implement `eval` (parse via deno_task_shell::parser; execute with the mirrored env map and registry; parse failure → `Err`, exit code → `Ok`) and verify `cargo build` passes.
- [x] 2.4 Implement `eval_captured` (inside `block_on`: `ShellState::new`, `pipe()` pairs, drop the stdin writer, `pipe_to_string_handle` readers, `execute_with_pipes`, await handles → `(code, stdout, stderr)`) and verify `cargo build` passes.

## 3. Integration tests (PoC parity)

- [x] 3.1 Typed layer: `cmd!` echo with interpolation succeeds, and a typed command that exits non-zero yields `Err` — verify `cargo test` passes.
- [x] 3.2 Shared state: `set_var` then `eval("echo hi $WHO && echo subshell files: $(ls | wc -l)")` returns code 0 — verify `cargo test` passes.
- [x] 3.3 Pipeline: `eval("echo c b a | tr ' ' '\n' | sort")` returns code 0 — verify `cargo test` passes.
- [x] 3.4 Mock + capture: register a git double, `eval_captured("git status && git commit -m 'hi'")` returns code 0 and the captured stdout contains both recorded argument lists (the real git binary never spawns) — verify `cargo test` passes.

## 4. Wrap-up

- [x] 4.1 Clean-build check: `cargo build && cargo test` green from a cleared `target/`, no `unwrap`/`expect` in `src/`, and any new gotchas discovered recorded in the QnA log.