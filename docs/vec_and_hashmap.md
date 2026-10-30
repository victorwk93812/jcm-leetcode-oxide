# `Vec<T>` and `HashMap<K, V>`: a working reference

These two collections carry most of the weight in everyday Rust and in nearly
every LeetCode solution. This page covers what each one *is* in memory (because
that explains every borrow-checker rule you will hit), the methods worth
knowing cold, and worked situations to study. Every code block on this page is
a doctest: `cargo test --doc` compiles and runs them.

---

## Part 1: `Vec<T>`

### 1.1 What a `Vec` is in memory

A `Vec<T>` is a 24-byte anchor on the stack tethered to a buffer on the heap:

```text
stack                          heap
┌──────────────┐               ┌────┬────┬────┬────┬────┬────┐
│ ptr  ────────┼──────────────▶│ 10 │ 20 │ 30 │ ?? │ ?? │ ?? │
│ cap  = 6     │               └────┴────┴────┴────┴────┴────┘
│ len  = 3     │                 ◀── len ──▶
└──────────────┘                 ◀────────── cap ──────────▶
```

Three facts follow from this picture and explain most of the API:

- `push` is amortized O(1). When `len == cap`, the buffer is reallocated at
  roughly double the size and the old bytes are moved. **Every pointer into the
  old buffer is now dangling.** That is why the borrow checker refuses to let
  you `push` while a `&v[i]` is alive: it is protecting you from a real
  use-after-free, not being pedantic.
- `insert(i, x)` and `remove(i)` are O(n): everything after `i` shifts.
  `swap_remove(i)` is O(1) because it moves the last element into the hole,
  at the cost of order.
- Passing `&v` to a function costs nothing: it copies 8 bytes (a pointer to the
  anchor), or 16 bytes if you pass `&v[..]` as a slice (`ptr`, `len`).

```rust
let mut v: Vec<i32> = Vec::with_capacity(4);
assert_eq!(v.len(), 0);
assert!(v.capacity() >= 4);

v.extend([1, 2, 3, 4]);
v.push(5); // len exceeded cap: buffer reallocated, possibly moved
assert!(v.capacity() >= 5);
assert_eq!(v, [1, 2, 3, 4, 5]);
```

The rule the compiler enforces, shown failing on purpose:

```rust,compile_fail
let mut v = vec![1, 2, 3];
let first = &v[0]; // shared borrow into the heap buffer
v.push(4);         // error[E0502]: cannot borrow `v` as mutable
println!("{first}");
```

Fix pattern: copy the value out (`let first = v[0];` since `i32: Copy`), or
finish using the borrow before mutating.

### 1.2 Method cheat sheet

| Group | Method | Cost | Notes |
|---|---|---|---|
| create | `Vec::new()`, `vec![]` | O(1) | no allocation until first push |
| | `Vec::with_capacity(n)` | O(1) | use when the final size is known |
| | `vec![x; n]` | O(n) | clones `x` n times, see 1.4 pitfall |
| | `iter.collect::<Vec<_>>()` | O(n) | the workhorse |
| read | `v[i]` | O(1) | panics out of range |
| | `v.get(i)` | O(1) | `Option<&T>`, no panic |
| | `v.first()`, `v.last()` | O(1) | `Option<&T>` |
| | `v.len()`, `v.is_empty()` | O(1) | |
| | `&v[a..b]` | O(1) | slice view, no copy |
| ends | `push(x)`, `pop()` | O(1)* | *amortized; this is your stack |
| middle | `insert(i, x)`, `remove(i)` | O(n) | shifts the tail |
| | `swap_remove(i)` | O(1) | order not preserved |
| | `swap(i, j)` | O(1) | |
| bulk | `extend(iter)` | O(k) | |
| | `truncate(n)`, `clear()` | O(n) | drops the tail |
| | `resize(n, x)` | O(n) | grow with `x` or shrink |
| | `retain(\|x\| ..)` | O(n) | in-place filter |
| | `dedup()` | O(n) | removes *consecutive* duplicates |
| | `drain(range)` | O(n) | moves a range out as an iterator |
| | `split_off(i)` | O(n) | splits into two Vecs |
| order | `sort()`, `sort_unstable()` | O(n log n) | needs `T: Ord`; unstable is faster |
| | `sort_by(\|a, b\| ..)`, `sort_by_key(\|x\| ..)` | O(n log n) | `f64`: use `a.total_cmp(b)` |
| | `reverse()`, `rotate_left(k)` | O(n) | |
| search | `binary_search(&x)` | O(log n) | sorted input; `Err(i)` is the insert point |
| | `contains(&x)` | O(n) | linear scan |
| | `iter().position(\|x\| ..)` | O(n) | |
| views | `windows(k)`, `chunks(k)` | O(1) | overlapping / non-overlapping slices |
| | `split_at_mut(i)` | O(1) | two disjoint `&mut` halves |
| | `iter()`, `iter_mut()`, `into_iter()` | O(1) | `&T`, `&mut T`, `T` |

### 1.3 The iteration trio

The three loops below are `v.iter()`, `v.iter_mut()`, and `v.into_iter()`.
Which one you want is the ownership question: look, edit, or take.

```rust
let v = vec![String::from("a"), String::from("b")];

for s in &v {
    let _: &String = s; // borrow: v is untouched afterwards
}

let mut v = v;
for s in &mut v {
    s.push('!'); // exclusive borrow: edit in place
}
assert_eq!(v, ["a!", "b!"]);

for s in v {
    let _: String = s; // move: v is gone after this loop
}
```

For `Copy` element types, `iter().copied()` gives you values instead of
references, which keeps the rest of the chain free of `&`.

### 1.4 Idioms and pitfalls

**Accept slices, return Vecs.** A function that only reads should take `&[T]`,
not `&Vec<T>`. Arrays, Vecs, and other slices all coerce to it.

```rust
fn total(xs: &[i32]) -> i32 {
    xs.iter().sum()
}
assert_eq!(total(&[1, 2, 3]), 6);
assert_eq!(total(&vec![4, 5]), 9);
```

**2D grids.** `vec![vec![0; cols]; rows]` is correct because `Vec::clone`
deep-copies the buffer. `vec![shared_handle; n]` is the trap: you get `n`
handles to one allocation.

```rust
use std::cell::RefCell;
use std::rc::Rc;

let mut grid = vec![vec![0; 3]; 2]; // two independent rows
grid[0][0] = 9;
assert_eq!(grid[1][0], 0);

let shared = vec![Rc::new(RefCell::new(0)); 3]; // three handles, ONE cell
*shared[0].borrow_mut() += 1;
assert_eq!(*shared[2].borrow(), 1); // surprise, if you expected 0
```

**Two mutable elements at once.** `&mut v[i]` and `&mut v[j]` cannot coexist
because the compiler cannot prove `i != j`. Use `swap`, or split the slice.

```rust
let mut v = vec![1, 2, 3, 4];
let (left, right) = v.split_at_mut(2);
left[0] += right[1]; // v[0] += v[3]
assert_eq!(v, [5, 2, 3, 4]);
v.swap(0, 3);
assert_eq!(v, [4, 2, 3, 5]);
```

**Sorting variants.**

```rust
let mut v = vec![3, 1, 2];
v.sort_unstable(); // fastest default for plain numbers
assert_eq!(v, [1, 2, 3]);
v.sort_unstable_by(|a, b| b.cmp(a)); // descending
assert_eq!(v, [3, 2, 1]);

let mut words = vec!["bb", "a", "ccc", "dd"];
words.sort_by_key(|w| w.len()); // stable: "bb" stays before "dd"
assert_eq!(words, ["a", "bb", "dd", "ccc"]);

let mut fs = vec![2.5_f64, -1.0, 0.0];
fs.sort_by(|a, b| a.total_cmp(b)); // f64 is not Ord because of NaN
assert_eq!(fs, [-1.0, 0.0, 2.5]);

let sorted = [1, 3, 5, 7];
assert_eq!(sorted.binary_search(&5), Ok(2));
assert_eq!(sorted.binary_search(&4), Err(2)); // where 4 would be inserted
```

**In-place bulk edits and slice views.**

```rust
let mut v = vec![1, 2, 2, 3, 3, 3, 4];
v.retain(|&x| x != 4); // in-place filter
v.dedup();             // consecutive duplicates only; sort first for a full dedup
assert_eq!(v, [1, 2, 3]);

let tail: Vec<i32> = v.drain(1..).collect(); // move a range out
assert_eq!(v, [1]);
assert_eq!(tail, [2, 3]);

let w = [1, 2, 3, 4, 5];
let pair_sums: Vec<i32> = w.windows(2).map(|p| p[0] + p[1]).collect();
assert_eq!(pair_sums, [3, 5, 7, 9]);

let rows: Vec<&[i32]> = w.chunks(2).collect();
assert_eq!(rows.len(), 3);
assert_eq!(rows[2], [5]);
```

**Strings on LeetCode.** For ASCII input, `s.as_bytes()` (a `&[u8]`, free)
or `s.bytes()` is faster and simpler than `s.chars().collect::<Vec<char>>()`.
Reach for `Vec<char>` only when the input is genuinely Unicode.

**Queues.** `v.remove(0)` is O(n). For BFS use `std::collections::VecDeque`
(`push_back` / `pop_front`, both O(1)).

### 1.5 Situations to study

**Situation A: a stack (Valid Parentheses, LC 20).** `push` and `pop` are the
whole data structure. `pop` returning `Option` folds the empty-stack check into
the comparison.

```rust
fn is_valid(s: &str) -> bool {
    let mut stack: Vec<u8> = Vec::with_capacity(s.len());
    for b in s.bytes() {
        let open = match b {
            b')' => b'(',
            b']' => b'[',
            b'}' => b'{',
            _ => {
                stack.push(b);
                continue;
            }
        };
        if stack.pop() != Some(open) {
            return false;
        }
    }
    stack.is_empty()
}
assert!(is_valid("()[]{}"));
assert!(!is_valid("(]"));
assert!(!is_valid("(("));
assert!(!is_valid("))"));
```

**Situation B: a monotonic stack of indices (Daily Temperatures, LC 739).**
Store indices, not values, when you need to write an answer back at a
position. `while let Some(&j) = stack.last()` copies the index out, so the
borrow of `stack` ends before `pop`.

```rust
fn daily_temperatures(temps: &[i32]) -> Vec<i32> {
    let mut answer = vec![0; temps.len()];
    let mut stack: Vec<usize> = Vec::new(); // indices awaiting a warmer day
    for (i, &t) in temps.iter().enumerate() {
        while let Some(&j) = stack.last() {
            if temps[j] >= t {
                break;
            }
            stack.pop();
            answer[j] = (i - j) as i32;
        }
        stack.push(i);
    }
    answer
}
assert_eq!(
    daily_temperatures(&[73, 74, 75, 71, 69, 72, 76, 73]),
    [1, 1, 4, 2, 1, 1, 0, 0]
);
```

**Situation C: sort then sweep (Merge Intervals, LC 56).** Sorting turns a
quadratic overlap check into a single pass. `last_mut()` lets you extend the
interval already in the output without re-pushing.

```rust
fn merge(mut intervals: Vec<[i32; 2]>) -> Vec<[i32; 2]> {
    intervals.sort_unstable_by_key(|iv| iv[0]);
    let mut out: Vec<[i32; 2]> = Vec::with_capacity(intervals.len());
    for iv in intervals {
        if let Some(last) = out.last_mut()
            && iv[0] <= last[1]
        {
            last[1] = last[1].max(iv[1]);
        } else {
            out.push(iv);
        }
    }
    out
}
assert_eq!(
    merge(vec![[1, 3], [2, 6], [8, 10], [15, 18]]),
    [[1, 6], [8, 10], [15, 18]]
);
assert_eq!(merge(vec![[1, 4], [4, 5]]), [[1, 5]]);
```

**Situation D: prefix sums (Range Sum Query, LC 303).** Precompute once in
O(n), answer every range query in O(1). Indexing `p[i]` while assigning
`p[i + 1]` is fine: an index expression is a momentary borrow, not a held one.

```rust
fn prefix_sums(nums: &[i32]) -> Vec<i32> {
    let mut p = vec![0; nums.len() + 1];
    for (i, &n) in nums.iter().enumerate() {
        p[i + 1] = p[i] + n;
    }
    p
}
let p = prefix_sums(&[-2, 0, 3, -5, 2, -1]);
let range_sum = |l: usize, r: usize| p[r + 1] - p[l]; // inclusive [l, r]
assert_eq!(range_sum(0, 2), 1);
assert_eq!(range_sum(2, 5), -1);
assert_eq!(range_sum(0, 5), -3);
```

**Situation E: a 2D DP table (Minimum Path Sum, LC 64).** Index loops are the
right tool here: the recurrence reads neighbours by position. The
`(Option, Option)` match keeps the edge cases explicit instead of seeding the
table with sentinels.

```rust
fn min_path_sum(grid: &[Vec<i32>]) -> i32 {
    let (rows, cols) = (grid.len(), grid[0].len());
    let mut dp = vec![vec![0; cols]; rows];
    for r in 0..rows {
        for c in 0..cols {
            let up = if r > 0 { Some(dp[r - 1][c]) } else { None };
            let left = if c > 0 { Some(dp[r][c - 1]) } else { None };
            let best = match (up, left) {
                (None, None) => 0,
                (Some(u), None) => u,
                (None, Some(l)) => l,
                (Some(u), Some(l)) => u.min(l),
            };
            dp[r][c] = best + grid[r][c];
        }
    }
    dp[rows - 1][cols - 1]
}
assert_eq!(min_path_sum(&[vec![1, 3, 1], vec![1, 5, 1], vec![4, 2, 1]]), 7);
assert_eq!(min_path_sum(&[vec![1, 2, 3], vec![4, 5, 6]]), 12);
```

**Situation F: two pointers in place (Remove Duplicates from Sorted Array,
LC 26).** A read index and a write index over the same buffer. The library
already has this as `dedup`, shown afterwards for comparison.

```rust
fn remove_duplicates(nums: &mut [i32]) -> usize {
    let mut write = 0;
    for read in 0..nums.len() {
        if read == 0 || nums[read] != nums[write - 1] {
            nums[write] = nums[read];
            write += 1;
        }
    }
    write
}
let mut nums = vec![0, 0, 1, 1, 1, 2, 2, 3, 3, 4];
let k = remove_duplicates(&mut nums);
assert_eq!(k, 5);
assert_eq!(&nums[..k], [0, 1, 2, 3, 4]);

let mut same = vec![0, 0, 1, 1, 1, 2, 2, 3, 3, 4];
same.dedup();
assert_eq!(same, [0, 1, 2, 3, 4]);
```

---

## Part 2: `HashMap<K, V>`

### 2.1 What a `HashMap` is in memory

`std::collections::HashMap` is hashbrown's SwissTable: one heap allocation
holding a run of 1-byte *control bytes* followed by the slots that store
`(K, V)` pairs inline.

```text
stack                    heap (one allocation)
┌──────────────┐         ┌─ control bytes ──┐┌──── slots: (K, V) inline ────┐
│ table ptr ───┼────────▶│ h7│ e │ h7│ h7│ e ││ (k,v)│  ·  │ (k,v)│ (k,v)│  ·  │
│ len          │         └──────────────────┘└──────────────────────────────┘
│ hasher state │           h7 = top 7 bits of hash, e = empty
└──────────────┘
```

Lookup: hash the key, jump to a group of 16 control bytes, compare all 16
against the key's 7-bit tag in one SIMD instruction, then check `Eq` only on
the (usually zero or one) candidates. Facts that follow:

- **Keys and values live inside the table.** Inserting moves them in. When the
  table grows past its load factor (7/8) it reallocates and every pair moves,
  so a `&V` from `get` blocks mutation exactly like a `&T` from a `Vec`.
- **Keys need `Hash + Eq`.** Integers, `bool`, `char`, `String`, `&str`,
  tuples and arrays of hashable things all qualify. `f64` does not (NaN breaks
  `Eq`); quantize to an integer or use `ordered_float` if you must.
- **The default hasher is SipHash-1-3 with a random seed.** It resists
  hash-flooding attacks and is slower than it needs to be for small integer
  keys. Consequence one: **iteration order is arbitrary and changes between
  runs.** Never assert on it. Consequence two: when a hot loop is dominated by
  hashing, `rustc-hash` (FxHashMap) or `ahash` are drop-in replacements.
- Average O(1) for `get` / `insert` / `remove`. `Vec` still wins on constants
  when keys are small dense integers (see 2.5).

### 2.2 Method cheat sheet

| Group | Method | Notes |
|---|---|---|
| create | `HashMap::new()`, `with_capacity(n)` | |
| | `HashMap::from([(k, v), ..])` | literal maps |
| | `iter_of_pairs.collect::<HashMap<_, _>>()` | from `(K, V)` items |
| read | `m.get(&k)` | `Option<&V>` |
| | `m.get_mut(&k)` | `Option<&mut V>` |
| | `m[&k]` | `&V`, **panics** if absent |
| | `m.contains_key(&k)` | |
| | `m.len()`, `m.is_empty()` | |
| write | `m.insert(k, v)` | returns `Option<V>`: the replaced value |
| | `m.remove(&k)` | returns `Option<V>` |
| | `m.entry(k)` | one hash, read-or-write; see 2.3 |
| | `m.retain(\|k, v\| ..)` | in-place filter |
| | `m.extend(iter_of_pairs)` | |
| iterate | `keys()`, `values()`, `values_mut()`, `iter()`, `iter_mut()` | borrowing |
| | `into_keys()`, `into_values()`, `into_iter()` | consuming |
| set | `HashSet::insert(x)` | `bool`: was it new? |
| | `HashSet::contains(&x)`, `remove(&x)` | |
| | `a.intersection(&b)`, `union`, `difference` | lazy iterators |

`insert` and `remove` handing back the old value is often exactly the signal
you need:

```rust
use std::collections::HashMap;

let mut m = HashMap::new();
assert_eq!(m.insert("k", 1), None);    // fresh key
assert_eq!(m.insert("k", 2), Some(1)); // replaced: old value handed back
assert_eq!(m.remove("k"), Some(2));
assert_eq!(m.remove("k"), None);
assert!(m.is_empty());
```

Indexing panics on a missing key. Use `get` unless absence is a bug.

```rust,should_panic
use std::collections::HashMap;

let m: HashMap<&str, i32> = HashMap::new();
let _ = m["missing"]; // panics; m.get("missing") returns None instead
```

### 2.3 The entry API

This is the barrier most newcomers hit. The naive shape hashes the key twice
and reads awkwardly. `entry` hashes once and returns a handle that is either
`Occupied` or `Vacant`; the combinators on it cover the common cases.

```rust
use std::collections::HashMap;

let words = ["a", "b", "a", "c", "a"];

// Idiomatic: one lookup.
let mut counts: HashMap<&str, usize> = HashMap::new();
for w in words {
    *counts.entry(w).or_insert(0) += 1; // or `.or_default()`
}
assert_eq!(counts["a"], 3);

// Naive: two lookups on the miss path, and easy to get wrong.
let mut naive: HashMap<&str, usize> = HashMap::new();
for w in words {
    if let Some(c) = naive.get_mut(w) {
        *c += 1;
    } else {
        naive.insert(w, 1);
    }
}
assert_eq!(naive, counts);

// The combinators.
let mut m: HashMap<&str, Vec<i32>> = HashMap::new();
m.entry("k").or_insert_with(Vec::new).push(1); // lazy default
m.entry("k").or_default().push(2);             // V: Default shortcut
m.entry("k")
    .and_modify(|v| v.push(3))                 // present: mutate
    .or_insert_with(|| vec![0]);               // absent: insert this
assert_eq!(m["k"], [1, 2, 3]);
assert_eq!(m.entry("fresh").or_insert_with(|| vec![0]), &[0]);
```

`or_insert(expr)` evaluates `expr` even when the key exists. Use
`or_insert_with` when the default is expensive to build.

### 2.4 Key ownership and borrowed lookups

A `HashMap<String, V>` can be queried with a `&str`: `get` accepts any `&Q`
such that `K: Borrow<Q>`, and `String: Borrow<str>`. No allocation for the
lookup.

```rust
use std::collections::HashMap;

let mut m: HashMap<String, i32> = HashMap::new();
m.insert("apple".to_string(), 1);
assert_eq!(m.get("apple"), Some(&1)); // &str lookup against String keys
assert!(m.contains_key("apple"));
```

A `HashMap<&str, V>` stores *tethers* into whatever the strings live in. The
map cannot outlive that source:

```rust,compile_fail
use std::collections::HashMap;

let mut m: HashMap<&str, i32> = HashMap::new();
{
    let line = String::from("apple");
    m.insert(&line, 1); // stores a borrow of `line` in `m`
}                        // error[E0597]: `line` dropped while still borrowed
assert_eq!(m["apple"], 1);
```

Rule of thumb: borrow keys (`&str`) when the source outlives the map, such as
a function parameter you only read from. Own keys (`String`) when the map is
returned or stored.

### 2.5 When not to use a `HashMap`

- **Keys are small dense integers** (lowercase letters, digits, ASCII, node
  ids below n): index a fixed array or a `Vec`. It is often 10x faster and
  there is nothing to hash.
- **You need order, ranges, or min/max**: `BTreeMap` / `BTreeSet`
  (`range(a..b)`, `first_key_value`, `last_key_value`).
- **You only need membership**: `HashSet<T>`.
- **You need a priority queue**: `BinaryHeap` (max-heap; wrap in `Reverse` for
  min-heap).

The array-as-map pattern (Valid Anagram, LC 242):

```rust
fn is_anagram(s: &str, t: &str) -> bool {
    if s.len() != t.len() {
        return false;
    }
    let mut counts = [0i32; 26];
    for (a, b) in s.bytes().zip(t.bytes()) {
        counts[(a - b'a') as usize] += 1;
        counts[(b - b'a') as usize] -= 1;
    }
    counts.iter().all(|&c| c == 0)
}
assert!(is_anagram("anagram", "nagaram"));
assert!(!is_anagram("rat", "car"));
```

### 2.6 Situations to study

**Situation A: frequency count then rank (Top K Frequent Elements, LC 347).**
Count with `entry`, then move the pairs into a `Vec` to sort. The tie-break on
the key makes the output deterministic even though the map's order is not.

```rust
use std::collections::HashMap;

fn top_k_frequent(nums: &[i32], k: usize) -> Vec<i32> {
    let mut freq: HashMap<i32, usize> = HashMap::new();
    for &n in nums {
        *freq.entry(n).or_default() += 1;
    }
    let mut pairs: Vec<(i32, usize)> = freq.into_iter().collect();
    pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs.into_iter().take(k).map(|(n, _)| n).collect()
}
assert_eq!(top_k_frequent(&[1, 1, 1, 2, 2, 3], 2), [1, 2]);
assert_eq!(top_k_frequent(&[4, 4, 7, 7, 9], 2), [4, 7]);
```

(An O(n) variant buckets by count into a `Vec<Vec<i32>>` indexed by
frequency. Worth writing yourself once.)

**Situation B: grouping by a derived key (Group Anagrams, LC 49).** The key is
a canonical form of the value: sorted bytes. `or_default().push(s)` moves each
string into its group without cloning. Output order is arbitrary, so the test
sorts before comparing.

```rust
use std::collections::HashMap;

fn group_anagrams(strs: Vec<String>) -> Vec<Vec<String>> {
    let mut groups: HashMap<Vec<u8>, Vec<String>> = HashMap::new();
    for s in strs {
        let mut key = s.clone().into_bytes();
        key.sort_unstable();
        groups.entry(key).or_default().push(s);
    }
    groups.into_values().collect()
}

let input = ["eat", "tea", "tan", "ate", "nat", "bat"]
    .into_iter()
    .map(String::from)
    .collect();
let mut got = group_anagrams(input);
for g in &mut got {
    g.sort();
}
got.sort();
assert_eq!(
    got,
    [vec!["ate", "eat", "tea"], vec!["bat"], vec!["nat", "tan"]]
);
```

**Situation C: prefix sums plus a count map (Subarray Sum Equals K, LC 560).**
This is Part 1's prefix sum idea with a `HashMap` replacing the array, because
the keys are sums, not indices. Seeding `{0: 1}` handles subarrays that start
at index 0.

```rust
use std::collections::HashMap;

fn subarray_sum(nums: &[i32], k: i32) -> i32 {
    let mut seen: HashMap<i32, i32> = HashMap::from([(0, 1)]);
    let (mut prefix, mut count) = (0, 0);
    for &n in nums {
        prefix += n;
        count += seen.get(&(prefix - k)).copied().unwrap_or(0);
        *seen.entry(prefix).or_insert(0) += 1;
    }
    count
}
assert_eq!(subarray_sum(&[1, 1, 1], 2), 2);
assert_eq!(subarray_sum(&[1, 2, 3], 3), 2);
assert_eq!(subarray_sum(&[1, -1, 0], 0), 3);
```

**Situation D: memoised recursion (Climbing Stairs, LC 70).** The map is
threaded through as `&mut`. Note the shape: `get`, recurse, `insert`. You
cannot use `entry(...).or_insert_with(|| recurse(memo))` here because the
closure would need `memo` while `entry` already holds it exclusively.

```rust
use std::collections::HashMap;

fn climb(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n <= 2 {
        return n;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = climb(n - 1, memo) + climb(n - 2, memo);
    memo.insert(n, v);
    v
}
assert_eq!(climb(5, &mut HashMap::new()), 8);
assert_eq!(climb(50, &mut HashMap::new()), 20_365_011_074);
```

The version that does not compile, so you recognise the error when you meet it:

```rust,compile_fail
use std::collections::HashMap;

fn climb(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n <= 2 {
        return n;
    }
    *memo
        .entry(n) // first &mut borrow of memo, held by the Entry
        .or_insert_with(|| climb(n - 1, memo) + climb(n - 2, memo)) // second &mut: E0499
}
```

When the keys are `0..=n`, a `Vec<u64>` memo (or bottom-up iteration) beats
the map. Reach for `HashMap` memoisation when the state is sparse or
composite, such as `(i, j, remaining)` tuples.

**Situation E: a seen-set (Longest Consecutive Sequence, LC 128).** Build a
`HashSet` once, then only start counting from numbers that begin a run. Each
element is visited O(1) times amortised, giving O(n) overall.

```rust
use std::collections::HashSet;

fn longest_consecutive(nums: &[i32]) -> usize {
    let set: HashSet<i32> = nums.iter().copied().collect();
    let mut best = 0;
    for &n in &set {
        if set.contains(&(n - 1)) {
            continue; // not the head of a run
        }
        let mut len = 1;
        while set.contains(&(n + len as i32)) {
            len += 1;
        }
        best = best.max(len);
    }
    best
}
assert_eq!(longest_consecutive(&[100, 4, 200, 1, 3, 2]), 4);
assert_eq!(longest_consecutive(&[0, 3, 7, 2, 5, 8, 4, 6, 0, 1]), 9);
assert_eq!(longest_consecutive(&[]), 0);
```

**Situation F: last-seen positions in a sliding window (Longest Substring
Without Repeating Characters, LC 3).** The map remembers where each byte was
last seen so the window start can jump instead of shrinking one step at a
time. For ASCII input a `[usize; 128]` array does the same job faster.

```rust
use std::collections::HashMap;

fn length_of_longest_substring(s: &str) -> usize {
    let mut last_seen: HashMap<u8, usize> = HashMap::new();
    let (mut start, mut best) = (0, 0);
    for (i, b) in s.bytes().enumerate() {
        if let Some(&prev) = last_seen.get(&b)
            && prev >= start
        {
            start = prev + 1;
        }
        last_seen.insert(b, i);
        best = best.max(i + 1 - start);
    }
    best
}
assert_eq!(length_of_longest_substring("abcabcbb"), 3);
assert_eq!(length_of_longest_substring("bbbbb"), 1);
assert_eq!(length_of_longest_substring("pwwkew"), 3);
assert_eq!(length_of_longest_substring(""), 0);
```

**Situation G: an index map built with `collect` (used in Construct Binary
Tree from Preorder and Inorder, LC 105).** Turning "find the position of `v`"
from O(n) into O(1) is the single most common `HashMap` speed-up.

```rust
use std::collections::HashMap;

let inorder = [9, 3, 15, 20, 7];
let index: HashMap<i32, usize> = inorder
    .iter()
    .enumerate()
    .map(|(i, &v)| (v, i))
    .collect();
assert_eq!(index[&20], 3);
assert_eq!(index.get(&42), None);
```

**Situation H: the iteration-order pitfall.** A test that passes on your
machine and fails in CI usually looks like this. Sort before asserting, or
collect into a `BTreeMap` when order is part of the contract.

```rust
use std::collections::{BTreeMap, HashMap};

let m: HashMap<&str, i32> = HashMap::from([("b", 2), ("a", 1), ("c", 3)]);

let mut keys: Vec<&str> = m.keys().copied().collect();
keys.sort_unstable(); // `keys()` order is arbitrary
assert_eq!(keys, ["a", "b", "c"]);

let ordered: BTreeMap<&str, i32> = m.into_iter().collect();
assert_eq!(ordered.keys().copied().collect::<Vec<_>>(), ["a", "b", "c"]);
assert_eq!(ordered.first_key_value(), Some((&"a", &1)));
```

---

## Summary table: which one, when

| Need | Reach for |
|---|---|
| Ordered sequence, index access, stack | `Vec<T>` |
| Queue / deque | `VecDeque<T>` |
| Key to value, keys are arbitrary hashable data | `HashMap<K, V>` |
| Key to value, keys are small dense integers | `Vec<V>` or `[V; N]` |
| Membership only | `HashSet<T>` |
| Sorted keys, range queries, min/max | `BTreeMap<K, V>` / `BTreeSet<T>` |
| Repeatedly extract the max/min | `BinaryHeap<T>` |

Both `Vec` and `HashMap` follow the same ownership law: a reference obtained
from `get` / indexing / `iter` is a tether into the heap allocation, and any
operation that might reallocate (`push`, `insert`, `entry`) is refused while
that tether is alive. When you hit that error, copy the value out, restructure
into two phases, or use `entry` / `split_at_mut` to express the disjointness
the compiler could not see.
