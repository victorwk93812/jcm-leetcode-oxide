//! # 2. Add Two Numbers
//!
//! <https://leetcode.com/problems/add-two-numbers/>
//!
//! Difficulty:
//! Tags:
//!
//! (problem statement summary)

pub struct Solution;

// Only ListNode::new() function allowed in this problem
use crate::common::ListNode;


// Copy to submission from here

fn pop_digit(list: &mut Option<Box<ListNode>>) -> i32 {
    match list.take() {
        Some(node) => {
            *list = node.next;
            node.val
        },
        None => 0
    }
}

impl Solution {
    pub fn add_two_numbers(
        mut l1: Option<Box<ListNode>>,
        mut l2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut dummy = ListNode::new(0);
        let mut tail = &mut dummy;
        let mut carry: i32 = 0;
        while l1.is_some() || l2.is_some() || carry != 0 {
            carry += pop_digit(&mut l1) + pop_digit(&mut l2);
            tail = tail.next.insert(Box::new(ListNode::new(carry % 10)));
            carry /= 10;
        }
        dummy.next
    }
}

// Copy until here

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let l1 = ListNode::from_slice(&[2, 4, 3]);
        let l2 = ListNode::from_slice(&[5, 6, 4]);
        assert_eq!(
            Solution::add_two_numbers(l1, l2),
            ListNode::from_slice(&[7, 0, 8])
        );
    }
    #[test]
    fn example_2() {
        let l1 = ListNode::from_slice(&[0]);
        let l2 = ListNode::from_slice(&[0]);
        assert_eq!(
            Solution::add_two_numbers(l1, l2),
            ListNode::from_slice(&[0])
        );
    }
    #[test]
    fn example_3() {
        let l1 = ListNode::from_slice(&[9, 9, 9, 9, 9, 9, 9]);
        let l2 = ListNode::from_slice(&[9, 9, 9, 9]);
        assert_eq!(
            Solution::add_two_numbers(l1, l2),
            ListNode::from_slice(&[8, 9, 9, 9, 0, 0, 0, 1])
        );
    }
}
