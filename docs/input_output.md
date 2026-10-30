# Reading input and writing output

Coming from C++, the first surprise is that Rust has no `cin >> x`. Standard
input hands you bytes or lines, and turning text into a number is a separate,
explicit, fallible step. This page covers the three standard reading patterns,
fast output, a reusable token scanner, and the crates that wrap all of it.

LeetCode never touches standard input: the judge calls your `Solution` method
with parsed arguments. This page matters for Codeforces, AtCoder, Advent of
Code, and ordinary command-line programs.

Every code block is a doctest. Blocks that read the real standard input are
marked `no_run`: they are compiled and type-checked but not executed, so
`cargo test --doc` never waits on a terminal. Each of them is followed by an
executed version that reads from in-memory bytes instead.

---

## 1. C++ to Rust at a glance

| C++ | Rust |
|---|---|
| `#include <iostream>` | `use std::io;` |
| `std::cin >> x;` | read text, then `tok.parse::<T>()` |
| `std::getline(std::cin, s);` | `io::stdin().read_line(&mut s)` |
| `while (std::getline(std::cin, s))` | `for line in io::stdin().lock().lines()` |
| `std::cout << x << '\n';` | `println!("{x}")` or `writeln!(out, "{x}")` |
| `std::endl` (flushes) | `out.flush()` |
| `std::cerr << x;` | `eprintln!("{x}")` |
| `ios::sync_with_stdio(false); cin.tie(nullptr);` | lock once, wrap stdout in `BufWriter` |
| `if (!(cin >> x))` failure state | `parse()` returns `Result`; `read_line` returns `Ok(0)` at end of input |

## 2. Parsing is explicit and fallible

`str::parse` is generic over the output type and returns a `Result`. The type
comes from an annotation or a turbofish. It does not skip whitespace, which is
the usual cause of a mysterious parse failure after `read_line`.

```rust
assert_eq!("42".parse::<i32>(), Ok(42));
assert!("4x".parse::<i32>().is_err());
assert!("42\n".parse::<i32>().is_err()); // trailing newline is not skipped
assert_eq!("42\n".trim().parse::<i32>(), Ok(42));

let x: f64 = "2.5".parse().expect("a float"); // type from the annotation
assert_eq!(x, 2.5);
```

There is no hidden failure flag as on a C++ stream. You decide at each call
what a bad token means: `expect` in contest code, `?` in real programs.

## 3. Pattern one: a line at a time

The closest thing to interactive `getline`. `read_line` **appends** to the
string and **keeps the trailing newline**, so `trim` before parsing and
`clear` before reusing the buffer. It returns the number of bytes read, and
`Ok(0)` means end of input.

```rust,no_run
use std::io;

fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("stdin readable");
    let n: usize = line.trim().parse().expect("an integer");

    line.clear(); // read_line appends
    io::stdin().read_line(&mut line).expect("stdin readable");
    let nums: Vec<i64> = line
        .split_ascii_whitespace()
        .map(|tok| tok.parse().expect("an integer"))
        .collect();

    println!("{n} {nums:?}");
}
```

The same logic, executed against in-memory input. A `&[u8]` implements
`BufRead`, which is what makes input code testable (see section 7).

```rust
use std::io::BufRead;

let mut input: &[u8] = b"3\n10 20 30\n";
let mut line = String::new();

input.read_line(&mut line).expect("readable");
assert_eq!(line, "3\n"); // newline kept
let n: usize = line.trim().parse().expect("an integer");

line.clear();
input.read_line(&mut line).expect("readable");
let nums: Vec<i64> = line
    .split_ascii_whitespace()
    .map(|tok| tok.parse().expect("an integer"))
    .collect();
assert_eq!((n, nums), (3, vec![10, 20, 30]));

line.clear();
assert_eq!(input.read_line(&mut line).expect("readable"), 0); // end of input
```

## 4. Pattern two: read everything, then pull tokens

The usual choice for competitive programming and the one that behaves most
like `cin >>`: line boundaries stop mattering. One read, one allocation, then
an iterator of `&str` tokens borrowed from that buffer.

```rust,no_run
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("stdin readable");
    let mut tokens = input.split_ascii_whitespace();
    let mut next = || tokens.next().expect("more input");

    let n: usize = next().parse().expect("an integer");
    let nums: Vec<i64> = (0..n).map(|_| next().parse().expect("an integer")).collect();
    println!("{}", nums.iter().sum::<i64>());
}
```

```rust
let input = "3\n10 20\n30\n"; // values split across lines: irrelevant here
let mut tokens = input.split_ascii_whitespace();
let mut next = || tokens.next().expect("more input");

let n: usize = next().parse().expect("an integer");
let nums: Vec<i64> = (0..n).map(|_| next().parse().expect("an integer")).collect();
assert_eq!(nums, [10, 20, 30]);
```

This cannot be used for interactive problems, because it waits for end of
input before doing anything. Use pattern one or the scanner in section 6 there.

## 5. Pattern three: iterate over lines

For line-structured input of unknown length. `lines()` strips the newline and
yields `io::Result<String>`, one fresh `String` per line. `lock()` takes the
stdin mutex once instead of once per call.

```rust,no_run
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("valid UTF-8");
        println!("{}", line.len());
    }
}
```

```rust
use std::io::{BufRead, Cursor};

let input = Cursor::new("ab\ncde\n\nf");
let lengths: Vec<usize> = input
    .lines()
    .map(|line| line.expect("valid UTF-8").len())
    .collect();
assert_eq!(lengths, [2, 3, 0, 1]); // empty line kept, last line needs no newline
```

## 6. A reusable scanner

When you want `cin >>` ergonomics without reading all input up front, the
common wrapper is a small struct that refills a token buffer one line at a
time. It is generic over `BufRead`, so the same type serves stdin in `main`
and a byte slice in tests.

```rust
use std::io::BufRead;
use std::str::FromStr;

struct Scanner<R> {
    reader: R,
    tokens: Vec<String>, // tokens of the current line, reversed so `pop` is in order
}

impl<R: BufRead> Scanner<R> {
    fn new(reader: R) -> Self {
        Self { reader, tokens: Vec::new() }
    }

    /// Next whitespace-separated token parsed as `T`.
    /// `None` at end of input or when the token does not parse.
    fn next<T: FromStr>(&mut self) -> Option<T> {
        loop {
            if let Some(tok) = self.tokens.pop() {
                return tok.parse().ok();
            }
            let mut line = String::new();
            if self.reader.read_line(&mut line).ok()? == 0 {
                return None;
            }
            self.tokens = line.split_ascii_whitespace().rev().map(String::from).collect();
        }
    }
}

let mut sc = Scanner::new(&b"2\n3 4\n\n5\n"[..]);
let n: usize = sc.next().expect("n");
let v: Vec<i32> = (0..n).map(|_| sc.next().expect("a value")).collect();
assert_eq!(v, [3, 4]);
assert_eq!(sc.next::<i32>(), Some(5)); // blank line skipped
assert_eq!(sc.next::<i32>(), None);    // end of input
// In a real program: Scanner::new(std::io::stdin().lock())
```

Folding "end of input" and "bad token" into one `None` is a contest-code
shortcut. A program with real users should return a `Result` with an error
type that tells the two apart.

## 7. Make it testable: take `impl BufRead`, not stdin

The idiomatic shape for any program that reads input is a function over
abstract reader and writer, with `main` as thin glue. The logic never names
`stdin`, so tests feed it bytes and inspect the bytes it wrote.

```rust
use std::io::{self, BufRead, Write};

fn solve(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let mut text = String::new();
    input.read_to_string(&mut text)?;
    let mut tokens = text.split_ascii_whitespace();
    let n: usize = tokens.next().and_then(|t| t.parse().ok()).unwrap_or(0);
    let sum: i64 = tokens.take(n).filter_map(|t| t.parse::<i64>().ok()).sum();
    writeln!(output, "{sum}")
}

let mut out = Vec::new();
solve(&b"3\n10 20 30\n"[..], &mut out).expect("in-memory io does not fail");
assert_eq!(out, b"60\n");

// The glue a binary would have:
//
// fn main() -> io::Result<()> {
//     solve(io::stdin().lock(), io::BufWriter::new(io::stdout().lock()))
// }
```

## 8. Output: this is the slow side

`println!` takes the stdout lock on every call, and stdout is line-buffered,
so every newline is a `write` system call. A hundred thousand `println!` calls
are a hundred thousand system calls. Lock once, buffer, and use `writeln!`.

```rust,no_run
use std::io::{self, BufWriter, Write};

fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for i in 0..100_000 {
        writeln!(out, "{i}").expect("stdout writable");
    }
    out.flush().expect("stdout writable");
}
```

`BufWriter` flushes when dropped, but a flush error at drop time is silently
discarded. Call `flush()` yourself at the end when the output matters, and
after each query in interactive problems.

```rust
use std::io::{BufWriter, Write};

let mut sink: Vec<u8> = Vec::new();
{
    let mut out = BufWriter::new(&mut sink);
    for i in 0..3 {
        writeln!(out, "{i}").expect("writable");
    }
    out.flush().expect("writable");
}
assert_eq!(sink, b"0\n1\n2\n");
```

Formatting quick reference for the macros:

```rust
let (name, pi, n) = ("x", 3.14159_f64, 42);
assert_eq!(format!("{name}={n}"), "x=42");      // inline captures
assert_eq!(format!("{pi:.2}"), "3.14");          // precision
assert_eq!(format!("{n:>5}|{n:<5}|"), "   42|42   |"); // width and alignment
assert_eq!(format!("{n:05}"), "00042");          // zero padding
assert_eq!(format!("{:?}", vec![1, 2]), "[1, 2]"); // Debug

let joined = [1, 2, 3].map(|v| v.to_string()).join(" ");
assert_eq!(joined, "1 2 3"); // space-separated output line
```

## 9. Crates that wrap all this

None of these are dependencies of this repository. They are listed so you
recognise them.

- **`proconio`** is the competitive-programming standard. Its `input!` macro
  reads typed values declaratively: `input! { n: usize, a: [i64; n] }`.
  AtCoder provides it. Codeforces allows no external crates, so paste pattern
  two or the scanner there.
- **`text_io`** offers `read!()` and `scan!()`, closer in spirit to `scanf`.
- **`clap`** is the answer when the input is really command-line arguments
  rather than a stream.

## 10. Which pattern, when

| Situation | Use |
|---|---|
| Batch judge, all input available up front | read everything, pull tokens (section 4) |
| Interactive judge, or prompts to a user | `read_line` or the scanner, and `flush` after each write |
| Line-oriented text of unknown length | `lines()` (section 5) |
| Anything you intend to test | `fn solve(impl BufRead, impl Write)` (section 7) |
| Large output | `BufWriter` + `writeln!` (section 8) |
