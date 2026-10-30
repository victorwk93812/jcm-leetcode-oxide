//! # 1. New Sum Revisited
//!
//! <https://leetcode.com/problems/new-sum-revisited/>
//!
//! Difficulty:
//! Tags:
//!
//! (problem statement summary)
pub struct Solution;

use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut seen: HashMap<i32, i32> = HashMap::with_capacity(nums.len());
        for (i, &num) in nums.iter().enumerate() {
            if let Some(&j) = seen.get(&(target - num)) {
                return vec![j, i as i32];
            } else {
                // as since enumerate returns usize
                seen.insert(num, i as i32);
            }
        }
        unreachable!("problem guarantees solution unity");
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
