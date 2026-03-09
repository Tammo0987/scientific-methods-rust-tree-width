//! Small test suite covering Rust control-flow patterns that should produce
//! CFGs with known or predictable treewidth.
//!
//! Expected treewidth intuitions:
//!   - `trivial`          tw=0 (single block, no edges)
//!   - `linear_sequence`  tw=1 (path graph)
//!   - `if_else`          tw=1 (diamond: two branches merging)
//!   - `nested_if`        tw=1 (nested diamonds still a tree-like structure)
//!   - `simple_loop`      tw=1 (loop back-edge)
//!   - `match_three`      tw=1 (three-way branch)
//!   - `loop_with_break`  tw=1-2 (loop + early exit)
//!   - `nested_loops`     tw=2  (nested loops raise treewidth)
//!   - `unsafe_raw`       tw=0-1 (unsafe but structurally simple)

// --- Trivial: single basic block, no control flow ---

pub fn trivial(x: i32) -> i32 {
    x + 1
}

// --- Linear: straight-line sequence of operations ---

pub fn linear_sequence(x: i32) -> i32 {
    let a = x * 2;
    let b = a + 3;
    let c = b - 1;
    c
}

// --- if/else: simple diamond ---

pub fn if_else(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        -x
    }
}

// --- Nested if/else ---

pub fn nested_if(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            x + y
        } else {
            x - y
        }
    } else {
        if y > 0 {
            y - x
        } else {
            -(x + y)
        }
    }
}

// --- Simple loop ---

pub fn simple_loop(n: u32) -> u32 {
    let mut sum = 0u32;
    let mut i = 0u32;
    while i < n {
        sum += i;
        i += 1;
    }
    sum
}

// --- match with three arms ---

pub fn match_three(x: u8) -> &'static str {
    match x {
        0 => "zero",
        1 => "one",
        _ => "many",
    }
}

// --- loop with early break ---

pub fn loop_with_break(data: &[i32]) -> Option<i32> {
    let mut i = 0;
    loop {
        if i >= data.len() {
            break None;
        }
        if data[i] < 0 {
            break Some(data[i]);
        }
        i += 1;
    }
}

// --- Nested loops (should raise treewidth) ---

pub fn nested_loops(n: usize, m: usize) -> usize {
    let mut count = 0;
    for _i in 0..n {
        for _j in 0..m {
            count += 1;
        }
    }
    count
}

// --- Unsafe function ---

pub unsafe fn unsafe_raw(ptr: *const i32) -> i32 {
    *ptr
}

// --- Multiple returns ---

pub fn multiple_returns(x: i32) -> i32 {
    if x < 0 {
        return -1;
    }
    if x == 0 {
        return 0;
    }
    1
}

// --- Short-circuit evaluation ---

pub fn short_circuit(a: bool, b: bool, c: bool) -> bool {
    a && b || c
}
