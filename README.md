# leetcode

Rust solutions to LeetCode problems, one module per problem.

## Layout

```
src/
  lib.rs                 crate root
  common/                LeetCode's ListNode / TreeNode + test helpers, `tree!` macro
  problems/
    mod.rs               registry: one `pub mod` per problem, sorted by number
    p0001_two_sum.rs     one file per problem: doc header, `Solution`, tests
templates/problem.rs     scaffold used by scripts/new.sh
scripts/new.sh           create + register a new problem
```

Each problem file is self-contained: a `pub struct Solution;` with the
LeetCode-shaped method, so the body can be pasted straight into the judge,
and a `#[cfg(test)]` module with the examples plus any edge cases.

## Usage

```sh
# scaffold problem 206
scripts/new.sh 206 reverse-linked-list

# run one problem's tests
cargo test p0206

# run everything
cargo test

# lint + format
cargo clippy --all-targets
cargo fmt
```

Linked-list and tree problems can import the shared types:

```rust
use crate::common::{ListExt, ListNode, TreeNode};
use crate::{list, tree};

let head = list![1, 2, 3];
let sum: i32 = head.values().sum();             // borrowing iterator, yields &i32
let doubled: Vec<i32> = head.into_values().map(|v| v * 2).collect(); // owning
assert_eq!(list![4, 5].to_vec(), [4, 5]);        // shorthand for assertions

let root = tree![3, 9, 20, None, None, 15, 7];
```

## Study notes

`docs/` holds longer-form notes. Their code blocks are compiled and run as
doctests through `src/study.rs`, so they cannot go stale:

```sh
cargo test --doc
```

- `docs/vec_and_hashmap.md` — `Vec` and `HashMap` memory model, method cheat
  sheets, idioms, and worked LeetCode situations.
- `docs/input_output.md` — reading stdin and writing stdout coming from C++:
  the three reading patterns, a token scanner, fast output, and helper crates.
