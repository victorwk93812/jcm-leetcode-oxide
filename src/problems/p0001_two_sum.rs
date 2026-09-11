//! # 1. Two Sum
//!
//! <https://leetcode.com/problems/two-sum/>
//!
//! Difficulty: Easy
//! Tags: array, hash-table
//!
//! Given an array of integers `nums` and an integer `target`, return indices
//! of the two numbers such that they add up to `target`. Exactly one solution
//! exists and an element may not be used twice.

use std::collections::HashMap;

pub struct Solution;

impl Solution {
    /// O(n) time, O(n) space: single pass with a value -> index map.
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut seen: HashMap<i32, i32> = HashMap::with_capacity(nums.len());
        for (i, &n) in nums.iter().enumerate() {
            if let Some(&j) = seen.get(&(target - n)) {
                return vec![j, i as i32];
            }
            seen.insert(n, i as i32);
        }
        unreachable!("problem guarantees exactly one solution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        assert_eq!(Solution::two_sum(vec![2, 7, 11, 15], 9), vec![0, 1]);
    }

    #[test]
    fn example_2() {
        assert_eq!(Solution::two_sum(vec![3, 2, 4], 6), vec![1, 2]);
    }

    #[test]
    fn duplicates() {
        assert_eq!(Solution::two_sum(vec![3, 3], 6), vec![0, 1]);
    }

    #[test]
    fn negative_numbers() {
        assert_eq!(Solution::two_sum(vec![-1, -2, -3, -4, -5], -8), vec![2, 4]);
    }
}
