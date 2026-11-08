# Tracing extension agent instructions

Before making changes, read and follow the complete
[shared Rust coding requirements](../az_foundation_suite_rlib/CODING.md).
Resolve this path relative to this file; report an unavailable shared guide
explicitly. Foundation owns the common guide. Keep only repository-specific
requirements and explicit overrides below.

## Repository layout

- Cargo workspace root: the repository root (overrides the shared `crates/`
  default). Member crates live under `crates/`.
- Keep Rust tests under each member crate's `src/tests/`.
