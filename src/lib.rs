//! bashless — shell scripting without writing bash.
//!
//! A dax-style Rust library: typed, injection-proof command building ([`xshell`])
//! plus bash-subset script evaluation ([`deno_task_shell`]) behind one facade,
//! with dax-style command mocking for tests.
//!
//! Architecture (PoC-proven; see `openspec/changes/scaffold-crate/design.md`):
//! - typed layer: `cmd!` — compile-time argv, non-zero exit = error
//! - script layer: pipelines, `&&`/`||`, globs, `$()`; a bash subset by design —
//!   control flow belongs in Rust
//! - mock layer: registered commands resolve BEFORE PATH — test doubles for free
//!
//! Traps encoded here (do not rediscover): everything runs on a tokio
//! current-thread runtime owned by the facade; the stack is `!Send`
//! (single-threaded by design); scripts get no env/cwd back-propagation —
//! use [`Bashless::set_var`].

use std::{collections::HashMap, ffi::OsString, rc::Rc};

use xshell::Shell;

pub use deno_task_shell::{ExecuteResult, ShellCommand, ShellCommandContext};
pub use futures::future::LocalBoxFuture;

/// Errors at the facade's trust boundaries: script parsing and I/O.
#[derive(Debug)]
pub enum Error {
    /// A script string failed to parse.
    ScriptParse(String),
    /// An I/O failure (facade construction, pipe handling, ...).
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ScriptParse(msg) => write!(f, "script parse error: {msg}"),
            Error::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ScriptParse(_) => None,
            Error::Io(err) => Some(err),
        }
    }
}

/// Crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// The facade: two layers, one logical shell state.
///
/// `!Send` by design (the mock registry holds `Rc`s) — use it from one thread.
pub struct Bashless {
    sh: Shell,
    // xshell's env map is private, so overrides set via set_var are mirrored
    // here for the script layer; the PoC proved this is the working shape.
    env_overlay: HashMap<OsString, OsString>,
    // dax-style mock hooks: command name → test double; resolved before PATH.
    custom: HashMap<String, Rc<dyn ShellCommand>>,
    // deno_task_shell is async and spawns its pipe tasks via tokio.
    rt: tokio::runtime::Runtime,
}

impl Bashless {
    /// Builds the runtime, the typed shell, an empty override map and an
    /// empty mock registry.
    pub fn new() -> Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(Error::Io)?;
        // xshell::Error impls std::error::Error — box it as the Io source.
        let sh = Shell::new()
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(Self {
            sh,
            env_overlay: HashMap::new(),
            custom: HashMap::new(),
            rt,
        })
    }

    /// Sets a variable visible to BOTH layers (xshell keeps its env private,
    /// so the script layer sees the mirrored override).
    pub fn set_var(&mut self, key: &str, val: &str) {
        self.sh.set_var(key, val);
        self.env_overlay.insert(key.into(), val.into());
    }

    /// The typed layer: build `cmd!` commands against this shell.
    /// Non-zero exits are errors here — unlike [`Bashless::eval`].
    pub fn shell(&self) -> &Shell {
        &self.sh
    }

    /// Mutable access to the mock registry:
    /// `bk.custom_mut().insert("git".into(), Rc::new(MyGit))` — registered
    /// names resolve before PATH during script evaluation.
    pub fn custom_mut(&mut self) -> &mut HashMap<String, Rc<dyn ShellCommand>> {
        &mut self.custom
    }

    fn env(&self) -> HashMap<OsString, OsString> {
        let mut env: HashMap<_, _> = std::env::vars_os().collect();
        env.extend(self.env_overlay.clone());
        env
    }

    /// Evaluates a bash-subset script (pipelines, `&&`/`||`/`;`, globs, `$()`)
    /// and returns its exit code. Parse failures are errors; non-zero exits
    /// are values — capture-style callers assert on them.
    pub fn eval(&self, script: &str) -> Result<i32> {
        let list = deno_task_shell::parser::parse(script)
            .map_err(|e| Error::ScriptParse(e.to_string()))?;
        Ok(self.rt.block_on(deno_task_shell::execute(
            list,
            self.env(),
            self.sh.current_dir(),
            self.custom.clone(),
            Default::default(),
        )))
    }

    /// Like [`Bashless::eval`], but captures stdout/stderr as strings instead
    /// of inheriting the terminal. Returns `(exit_code, stdout, stderr)`.
    pub fn eval_captured(&self, script: &str) -> Result<(i32, String, String)> {
        let list = deno_task_shell::parser::parse(script)
            .map_err(|e| Error::ScriptParse(e.to_string()))?;
        let env = self.env();
        let cwd = self.sh.current_dir();
        let custom = self.custom.clone();
        // Pipe setup must run inside the runtime — the stream-handle helper
        // spawns tokio tasks and panics outside one.
        self.rt.block_on(async move {
            let state =
                deno_task_shell::ShellState::new(env, cwd, custom, Default::default());
            let (in_r, in_w) = deno_task_shell::pipe();
            drop(in_w); // stdin = immediate EOF
            let (out_r, out_w) = deno_task_shell::pipe();
            let (err_r, err_w) = deno_task_shell::pipe();
            let out_h = out_r.pipe_to_string_handle();
            let err_h = err_r.pipe_to_string_handle();
            let code =
                deno_task_shell::execute_with_pipes(list, state, in_r, out_w, err_w).await;
            let out = out_h
                .await
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let err = err_h
                .await
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            Ok((code, out, err))
        })
    }
}