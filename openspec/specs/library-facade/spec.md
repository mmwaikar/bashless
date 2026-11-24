# library-facade Specification

## Purpose
Defines the public facade through which scripts and typed commands are executed, tested, and captured — the single entry point that makes shell automation mockable and testable in Rust.

## Requirements

### Requirement: Facade initialization
The library MUST provide a single constructor that initializes the shared shell state — the process environment plus explicit overrides, an empty command registry, and the current directory — and MUST report initialization failures as errors rather than panics.

#### Scenario: Construction succeeds
- **WHEN** a caller constructs the facade in a normal environment
- **THEN** construction succeeds and subsequent operations use the inherited process environment

#### Scenario: Initialization failure is an error
- **WHEN** construction cannot complete (e.g. its execution runtime cannot be built)
- **THEN** the caller receives an error value, and no panic escapes the library

### Requirement: Shared variable state
Variables set through the facade MUST be visible to both typed commands and evaluated scripts.

#### Scenario: Variable reaches the script layer
- **WHEN** a caller sets a variable and evaluates a script that references it
- **THEN** the script resolves the variable to the set value

### Requirement: Script evaluation
The facade MUST evaluate script strings supporting pipelines, `&&`/`||`/`;` sequencing, command substitution, and globbing, without spawning a platform shell; it MUST return the script's exit code, and MUST report unparseable scripts as errors rather than panics.

#### Scenario: Pipeline evaluates cross-platform
- **WHEN** a caller evaluates a script containing a pipeline
- **THEN** evaluation returns exit code 0 and runs without invoking `/bin/sh`

#### Scenario: Command substitution inside scripts
- **WHEN** a caller evaluates a script containing `$(...)` substitution
- **THEN** the substituted command's output is used within the outer script

#### Scenario: Unparseable script is an error
- **WHEN** a caller evaluates a script that fails to parse
- **THEN** the caller receives an error, and no command runs

### Requirement: Output capture
Evaluating in capture mode MUST return the exit code together with stdout and stderr as strings, and MUST NOT write the script's output to the terminal.

#### Scenario: Captured streams are returned
- **WHEN** a caller captures a script that writes to both stdout and stderr
- **THEN** both streams are returned as separate strings and nothing is written to the terminal

### Requirement: Command mocking
A registered test double for a command name MUST take precedence over PATH lookup for that command during script evaluation; commands without a registered double MUST continue to resolve via PATH.

#### Scenario: Registered double intercepts the command
- **WHEN** a script calls a command name that has a registered double
- **THEN** the double executes with the script's arguments and the real binary never runs

#### Scenario: Unregistered commands still resolve
- **WHEN** a script calls a command name with no registered double
- **THEN** the command resolves via PATH as usual

### Requirement: Typed command execution
The facade MUST expose the typed layer so callers can construct commands with compile-time-checked, injection-safe argument interpolation, and a typed command's non-zero exit MUST be reported as an error.

#### Scenario: Interpolation is injection-safe
- **WHEN** a typed command is built with an interpolated argument containing spaces or quotes
- **THEN** the argument is passed to the spawned process as a single, unmodified argument

#### Scenario: Non-zero typed exit is an error
- **WHEN** a typed command exits non-zero
- **THEN** the caller receives an error rather than a success value
