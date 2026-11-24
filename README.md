# bashless

Shell scripting without writing bash — a dax-style Rust library with mockable, testable commands.

Two layers behind one facade:

- **typed layer** (`xshell`): `cmd!` builds argv at compile time — interpolation is
  injection-proof, non-zero exits are errors.
- **script layer** (`deno_task_shell`): evaluate bash-subset strings — pipelines,
  `&&`/`||`/`;`, globs, `$()` — cross-platform, no `/bin/sh` involved. Control flow
  belongs in Rust, not in the script.
- **mock layer**: register test doubles for command names; they resolve before PATH,
  so tests never run the real `git`.

## Status

Scaffold — implemented per [`openspec/changes/scaffold-crate`](openspec/changes/scaffold-crate).
Next up: mock/assert ergonomics, CI (with a Windows leg), a thin runner crate.

## Example

```rust
use std::rc::Rc;
use bashless::{Bashless, ExecuteResult, ShellCommand, ShellCommandContext};

let mut bk = Bashless::new()?;

// script layer — the dax `$`-string equivalent
bk.set_var("WHO", "world");
assert_eq!(bk.eval("echo hi $WHO")?, 0);

// test layer — mock git, capture output, assert on what ran
// bk.custom_mut().insert("git".into(), Rc::new(MockGit));
// let (code, stdout, stderr) = bk.eval_captured("git status && git commit -m 'hi'")?;
```

Scripts deliberately have no `if`/`for`/`while` — control flow lives in the Rust
caller, which is the whole point of the two-layer design.
