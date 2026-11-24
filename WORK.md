# this_file: WORK.md

# Work progress

- PyPI publishing uses the `pypi` environment and OIDC through `pypi-upload.yml`. The repository-side workflow and local handoff were checked without publishing. The PyPI project owner must confirm the trusted publisher registration.
- `./build.sh docs` rebuilt the fly-january 0.2.1 site in strict mode with zero warnings.
- The workspace gate with Cargo's `--no-fail-fast` passed 559 Rust tests and 5 doctests. Two CLI fixtures failed because the sandbox denied access to `/dev/stdout`.
- A fresh Python wheel passed 38 tests. The published WebAssembly bundle passed six smoke cases and four assertions. The C API smoke passed. The C++ example failed to compile against the current C header.
- The scheduled fuzz workflow is disabled for inactivity. The active backlog and its next checks are in `PLAN.md` and `TODO.md`.
