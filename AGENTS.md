= Context
See `README.md` for the purpose of `shortcut-catapult`.

This is a simple tool and background service, designed to be quick and easy to use.

== Design

=== Architecture
==== Libraries

- `regex` for regular expression parsing
- `serde` for parsing the config file
- `tokio` for async IO
- `clap`  for parsing the command line
- `tracing` and `tracing-subscriber` for logging
- `eyre` and `color-eyre` for error handling
- `axum` for running the async HTTP server
- `serde_yaml` for reading the YAML configuration
- `url` for manipulating URLs
- `strsim` for fuzzy string matching
- `xdg` for locating the `$XDG_CONFIG_HOME` path
- `assert_cmd` for integration testing command line behavior (dev)
- `assert_fs` for creating temporary files in tests (dev)
- `predicates` for asserting process output (dev)

==== Program Structure
This is a command line tool (bin). It consists of the following high-level modules. These can of course contain sub-modules.

- `main` the program entrypoint. Sets up global libraries like `tracing-subscriber`
- `cli` defines the command line interface (using `clap` and its derive feature)
- `config` contains the config parsing logic
- `matching` defines the core logic of the tool (free of any IO/HTTP concerns)
- `matching::xxx` each matcher is a sub-module of `matching`
- `daemon` contains the http server (handles HTTP concerns, calls out to `matching`)

=== Priorities

==== Maintainability and correctness are top priority. 

Make illegal state unrepresentable. E.g., If two optional parameters/fields are only ever both absent or both present, encode them as an optional tuple `Option<(A, B)>` or 
optional struct. 

Prefer enums over booleans, unless it is 100% clear what the boolean means.
E.g., a `enabled: bool` is fine, but a `debug_mode: bool` should really be a `mode: Mode` with an appropriately defined `enum Mode`.

Prefer integration/acceptance-level tests over fine-grained unit tests.
The tests don't literally have to be integration tests (in the cargo sense), but they should test as much of the system as possible.
Test input/output behavior of the overall system. E.g., given config X and input URL Y, expect output URL Z.

Don't test functionality of third-party libraries. We assume that third-party libraries are fully tested.
E.g., when writing tests for the regex matcher, we don't need to test different regex patterns (we assume the regex crate is properly tested).
But we do need to verify that the pattern we specify in the config is the one that is being used.

Prefer readability over performance. It is fine to clone values. 

The tool must behave correctly under concurrent use (multiple concurrent requests).

Handle errors properly. If you use `unwrap` or `expect`, add a comment that explains why this is safe: `// UNWRAP: checked that the list is non-empty above`.
For module-private functions, you can also add requirements in the form of doc comments (e.g., "requires non-empty list").

==== Performance & Efficiency
We are targeting modern desktop systems. The tool should not waste resources while it's not being used (e.g., it shouldn't poll the config file in the background).

If easily doable, we should take advantage of Rusts ownership model to avoid unnecessary copies. That means: where it's trivially doable, use references and slices.

Follow the usual Rust best practices of letting the caller decide whether to clone a value. If you need an owned value in a function, ask the caller to provide one.

But once again: if it becomes tricky to satisfy Rust lifetime rules, just clone a value.

==== Security
We trust the user's configuration and the input. E.g., we don't defend against malicious regexes or excessively long requests.

You must not use `unsafe`. Undefined behavior is not acceptable. Dynamic memory leaks are unacceptable. (It is OK to statically allocate as part of program startup.)

== Contributing
=== Workflow

1. (for bugfixes only) write a test that reproduces the bug. The test should initially fail
2. implement the feature or fix the bug
3. (for features) write high-level/acceptance tests that assert the required functionality
4. `cargo check` until it compiles (warnings are OK at this stage)
6. `cargo test` until all tests pass
7. `cargo clippy` until there are no more warnings (ask before suppressing a warning)
8. `cargo fmt` before open/update the PR (rustfmt)

=== Code Style
1. We use rustfmt default settings
2. Inline comments must explain WHY something happens (why is it important, why some other approach doesn't work). A comment must not simply repeat WHAT the code is doing.
3. If units of measure are involved, add them as suffixes. (bad: `timeout: u32`, good: `timeout_ms: u32`, best: `timeout: Duration`). This is very important for parameters and fields.
4. Add documentation comments where types alone are not adequate. E.g., to describe units of measure
5. Test functions should use the following naming convention: `<scenario>_<expected outcome>`. For example, a test that tests that an empty input results in an error could be called `empty_error`.
6. Don't prefix test functions with `test`
