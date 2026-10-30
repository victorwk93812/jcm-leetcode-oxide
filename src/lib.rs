//! LeetCode solutions, one module per problem.
//!
//! Layout:
//! - `src/problems/pNNNN_slug.rs` — one file per problem, containing a
//!   `Solution` struct with the LeetCode-shaped method and its own tests.
//! - `src/problems/mod.rs` — registry of problem modules (kept sorted).
//! - `src/common/` — shared LeetCode data structures (`ListNode`, `TreeNode`)
//!   and helpers to build them in tests.
//! - `docs/` — study notes; their code blocks run as doctests via `src/study.rs`.
//!
//! Add a problem with `scripts/new.sh <number> <slug>`.

pub mod common;
pub mod problems;
pub mod study;
