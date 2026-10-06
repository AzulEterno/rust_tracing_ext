# Agent Operating Rules

## Practical Panic Policy

- Prevent avoidable first-party panics from invalid public input, malformed data,
  poisoned shared state, arithmetic, and broken ownership or lifecycle transitions.
  Validate at the owning boundary before mutation or external effects; prefer
  checked operations and existing typed errors. Preserve accounting, rollback,
  ordering, and cleanup when propagating failures.
- In non-test source, do not use unexplained `unwrap()`, `expect()`, panic macros,
  unchecked indexing, or helpers with known panic preconditions. Retained
  exceptions must document the enforced invariant or deliberate panic reason at
  the call site, and caller-reachable panic conditions in public API documentation.
  An `expect()` message alone is not justification. Review panic during unwinding
  separately, especially in `Drop`; do not mask broken state with empty defaults
  or broad `catch_unwind` wrappers.
- Ordinary collection/string allocation, cloning, and serialization follow the
  process allocation contract. Do not add storage-only public APIs/errors,
  synthetic reservation helpers, or pervasive `try_reserve` plumbing solely to
  recover individual allocations while adjacent operations still allocate.
  This policy does not promise recovery from allocator exhaustion or arbitrary
  dependency internals, callbacks, and destructors.
- Check concrete count/length arithmetic, destination layout/addressability,
  bounded-container limits, and known dependency preconditions caused by our
  inputs. Preserve explicit resource admission before consuming ownership,
  invoking effectful callbacks, or publishing committed state. Capacity overflow
  is distinct from allocator exhaustion; ordinary allocation is not a reason to
  remove a real admission or ownership contract.
- Change public APIs only for a concrete recoverable failure that callers need to
  handle. Update in-repository callers directly; avoid compatibility wrappers,
  speculative frameworks, and exhaustive allocation/callee ledgers as closure
  requirements. A lexical match is a review lead, not proof of a bug.
- Tests may use panic operations for assertions and fixture setup. Verify real
  failure boundaries, unchanged state and valid retry where relevant; distinguish
  passing tests from ignored, compile-only, or unavailable runtime gates.

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
