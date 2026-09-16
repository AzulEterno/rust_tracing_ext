# Agent Operating Rules

## Progressive Source Documentation

- During every code task, refill missing Rust source documentation in the files
  materially touched by the change. Keep this progressive: do not turn a scoped
  implementation into an unrelated repository-wide documentation rewrite.
- Add or update `//!` module documentation and `///` documentation for public
  items and for non-obvious internal types, fields, variants, and functions.
- Document behavior, invariants, units, defaults, side effects, error and panic
  conditions, and security constraints where relevant. Do not merely restate an
  item's Rust name or signature.
- Add a small, compilable example for a reusable public API when an example
  materially clarifies correct use. Prefer ordinary doctests; use `no_run` when
  execution requires external resources, and use `ignore` only when compilation
  cannot be made portable.
- Keep examples synchronized with API changes and run the affected crate's
  documentation tests when executable examples are added or changed.
