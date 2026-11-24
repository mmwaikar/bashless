//! PoC parity: the four demo behaviors from the incubation workspace's
//! `2-infra/poc-shell/src/main.rs`, as integration tests.

use std::rc::Rc;

use bashless::{Bashless, ExecuteResult, LocalBoxFuture, ShellCommand, ShellCommandContext};
use xshell::cmd;

/// Zero-sized test double: records invoked args via its stdout line.
struct MockGit;

impl ShellCommand for MockGit {
    fn execute(&self, mut ctx: ShellCommandContext) -> LocalBoxFuture<'static, ExecuteResult> {
        Box::pin(async move {
            let _ = ctx
                .stdout
                .write_line(&format!("[mock git] args: {:?}", ctx.args));
            ExecuteResult::from_exit_code(0)
        })
    }
}

#[test]
fn typed_layer_runs_and_surfaces_non_zero_exit_as_error() {
    let bk = Bashless::new().unwrap();
    let layer = "typed";
    cmd!(bk.shell(), "echo hello from the {layer} layer")
        .run()
        .unwrap();

    // non-zero typed exit → Err (spec: typed command execution)
    #[cfg(unix)]
    assert!(cmd!(bk.shell(), "false").run().is_err());
    #[cfg(windows)]
    assert!(cmd!(bk.shell(), "cmd /C exit 1").run().is_err());
}

#[test]
fn set_var_reaches_the_script_layer() {
    let mut bk = Bashless::new().unwrap();
    bk.set_var("WHO", "bash-killer");
    let code = bk
        .eval("echo hi $WHO && echo subshell files: $(ls | wc -l)")
        .unwrap();
    assert_eq!(code, 0);
}

#[test]
fn pipelines_evaluate_cross_platform() {
    let bk = Bashless::new().unwrap();
    let code = bk.eval("echo c b a | tr ' ' '\n' | sort").unwrap();
    assert_eq!(code, 0);
}

#[test]
fn unparseable_scripts_are_errors_not_panics() {
    let bk = Bashless::new().unwrap();
    assert!(bk.eval("echo 'unterminated").is_err());
}

#[test]
fn mocked_git_intercepts_and_capture_returns_streams() {
    let mut bk = Bashless::new().unwrap();
    bk.custom_mut().insert("git".into(), Rc::new(MockGit));
    let (code, out, _err) = bk
        .eval_captured("git status && git commit -m 'hi'")
        .unwrap();
    assert_eq!(code, 0);
    assert!(
        out.contains(r#"[mock git] args: ["status"]"#),
        "missing status call, got: {out}"
    );
    assert!(
        out.contains(r#"[mock git] args: ["commit", "-m", "hi"]"#),
        "missing commit call, got: {out}"
    );
}