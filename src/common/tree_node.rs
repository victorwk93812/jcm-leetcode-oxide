use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Binary tree node, exactly as LeetCode defines it.
#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }

    /// Build a tree from LeetCode's level-order encoding, where `None` marks
    /// a missing child: `[Some(1), None, Some(2)]` is `1` with right child `2`.
    pub fn from_level_order(vals: &[Option<i32>]) -> Option<Rc<RefCell<TreeNode>>> {
        let mut iter = vals.iter().copied();
        let root = Rc::new(RefCell::new(TreeNode::new(iter.next()??)));
        let mut queue = VecDeque::from([Rc::clone(&root)]);

        while let Some(node) = queue.pop_front() {
            if let Some(Some(v)) = iter.next() {
                let child = Rc::new(RefCell::new(TreeNode::new(v)));
                node.borrow_mut().left = Some(Rc::clone(&child));
                queue.push_back(child);
            }
            if let Some(Some(v)) = iter.next() {
                let child = Rc::new(RefCell::new(TreeNode::new(v)));
                node.borrow_mut().right = Some(Rc::clone(&child));
                queue.push_back(child);
            }
        }
        Some(root)
    }

    /// Serialize back to level order (trailing `None`s trimmed) for assertions.
    pub fn to_level_order(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<Option<i32>> {
        let mut out = Vec::new();
        let mut queue = VecDeque::from([root]);
        while let Some(slot) = queue.pop_front() {
            match slot {
                Some(node) => {
                    let node = node.borrow();
                    out.push(Some(node.val));
                    queue.push_back(node.left.clone());
                    queue.push_back(node.right.clone());
                }
                None => out.push(None),
            }
        }
        while out.last() == Some(&None) {
            out.pop();
        }
        out
    }
}

/// Shorthand for level-order literals: `tree![1, None, 2]`.
#[macro_export]
macro_rules! tree {
    ($($t:tt)*) => {
        $crate::__tree_impl!([] $($t)*)
    };
}

/// Token muncher behind [`tree!`]: wraps each value in `Some`, passes `None`
/// through, and accumulates into a slice literal.
#[doc(hidden)]
#[macro_export]
macro_rules! __tree_impl {
    ([$($acc:expr),*]) => {
        $crate::common::TreeNode::from_level_order(&[$($acc),*])
    };
    ([$($acc:expr),*] None $(, $($rest:tt)*)?) => {
        $crate::__tree_impl!([$($acc,)* None] $($($rest)*)?)
    };
    ([$($acc:expr),*] $v:expr $(, $($rest:tt)*)?) => {
        $crate::__tree_impl!([$($acc,)* Some($v)] $($($rest)*)?)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let t = tree![3, 9, 20, None, None, 15, 7];
        assert_eq!(
            TreeNode::to_level_order(t),
            vec![Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]
        );
    }

    #[test]
    fn empty() {
        assert_eq!(tree![], None);
        assert_eq!(TreeNode::to_level_order(None), Vec::new());
    }
}
