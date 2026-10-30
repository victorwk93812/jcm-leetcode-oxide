//! # 3. Longest Substring Without Repeating Characters
//!
//! <https://leetcode.com/problems/longest-substring-without-repeating-characters/>
//!
//! Difficulty:
//! Tags:
//!
//! (problem statement summary)

pub struct Solution;

use std::collections::HashMap;
use std::cmp;

impl Solution {
    pub fn length_of_longest_substring(s: String) -> i32 {
        let mut seen: HashMap<u8, usize> = HashMap::new();
        let mut start: usize = 0;
        let mut len: usize = 0;
        for (i, &byte) in s.as_bytes().iter().enumerate() {
            if let Some(prev) = seen.insert(byte, i) {
                start = cmp::max(start, prev + 1);
            }
            len = cmp::max(len, i + 1 - start);
        }
        i32::try_from(len).expect("length should not overflow i32")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        assert_eq!(Solution::length_of_longest_substring(String::from("abcabcbb")), 3);
    }

    #[test]
    fn example_2() {
        assert_eq!(Solution::length_of_longest_substring(String::from("bbbbb")), 1);
    }

    #[test]
    fn example_3() {
        assert_eq!(Solution::length_of_longest_substring(String::from("pwwkew")), 3);
    }
}
