//! LeetCode's singly-linked list, plus iteration support.
//!
//! `ListNode` is a *node*, not a cursor, so it does not implement `Iterator`
//! itself. Iteration state lives in two small cursor types: [`Iter`]
//! (borrowing, yields `&i32`) and [`IntoIter`] (owning, yields `i32`).
//!
//! LeetCode hands lists around as `Option<Box<ListNode>>`. `Option` is a
//! foreign type and `Iterator`/`IntoIterator`/`FromIterator` are foreign
//! traits, so the orphan rule forbids implementing them on that type here.
//! The ergonomic entry points therefore live on the [`ListExt`] extension
//! trait (local trait, foreign type: allowed).

/// Singly-linked list node, exactly as LeetCode defines it.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }

    /// Build a list from a slice: `[1, 2, 3]` -> `1 -> 2 -> 3`.
    /// Returns `None` for an empty slice. See also the [`list!`](crate::list) macro.
    pub fn from_slice(vals: &[i32]) -> Option<Box<ListNode>> {
        vals.iter()
            .rev()
            .fold(None, |next, &val| Some(Box::new(ListNode { val, next })))
    }

    /// Borrowing iterator over this node and everything after it.
    pub fn iter(&self) -> Iter<'_> {
        Iter { cur: Some(self) }
    }
}

/// Borrowing cursor over a list. Yields `&i32` like `slice::iter`; use
/// `.copied()` when you want values.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    cur: Option<&'a ListNode>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a i32;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.cur?;
        self.cur = node.next.as_deref();
        Some(&node.val)
    }
}

impl std::iter::FusedIterator for Iter<'_> {}

/// Owning cursor over a list. Consumes the list node by node, freeing each
/// box as it is passed.
#[derive(Debug)]
pub struct IntoIter {
    cur: Option<Box<ListNode>>,
}

impl Iterator for IntoIter {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        let node = self.cur.take()?;
        self.cur = node.next;
        Some(node.val)
    }
}

impl std::iter::FusedIterator for IntoIter {}

impl<'a> IntoIterator for &'a ListNode {
    type Item = &'a i32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

// `Box` is `#[fundamental]`, so `Box<ListNode>` counts as a local type and
// this impl is allowed even though `Box` is foreign. It also makes
// `head.into_iter().flatten()` work on an `Option<Box<ListNode>>`.
impl IntoIterator for Box<ListNode> {
    type Item = i32;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter { cur: Some(self) }
    }
}

/// Iteration over a list head, i.e. an `Option<Box<ListNode>>`.
///
/// `Option` already has inherent `iter` / `into_iter` methods that would
/// shadow anything with those names, so these follow the
/// `HashMap::values` / `into_values` naming instead.
pub trait ListExt {
    /// Borrowing iterator over the values, front to back.
    fn values(&self) -> Iter<'_>;

    /// Owning iterator over the values, front to back. Consumes the list.
    fn into_values(self) -> IntoIter;

    /// Convenience for assertions: `self.values().copied().collect()`.
    fn to_vec(&self) -> Vec<i32> {
        self.values().copied().collect()
    }
}

impl ListExt for Option<Box<ListNode>> {
    fn values(&self) -> Iter<'_> {
        Iter {
            cur: self.as_deref(),
        }
    }

    fn into_values(self) -> IntoIter {
        IntoIter { cur: self }
    }
}

/// Shorthand for list literals: `list![1, 2, 3]`.
#[macro_export]
macro_rules! list {
    ($($v:expr),* $(,)?) => {
        $crate::common::ListNode::from_slice(&[$($v),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let list = list![1, 2, 3];
        assert_eq!(list.to_vec(), [1, 2, 3]);
    }

    #[test]
    fn empty() {
        assert_eq!(list![], None);
        assert_eq!(None.to_vec(), Vec::<i32>::new());
        assert_eq!(None.into_values().count(), 0);
    }

    #[test]
    fn borrowing_iteration_does_not_consume() {
        let list = list![4, 5, 6];
        let sum: i32 = list.values().sum();
        let max = list.values().copied().max();
        assert_eq!((sum, max), (15, Some(6)));
        assert_eq!(list.to_vec(), [4, 5, 6]); // still intact
    }

    #[test]
    fn owning_iteration_adapts() {
        let list = list![1, 2, 3, 4];
        let evens: Vec<i32> = list.into_values().filter(|n| n % 2 == 0).collect();
        assert_eq!(evens, [2, 4]);
    }

    #[test]
    fn into_iterator_impls() {
        let head = list![7, 8];
        let node: &ListNode = head.as_deref().expect("non-empty");
        assert_eq!(node.into_iter().copied().collect::<Vec<_>>(), [7, 8]);
        for (i, v) in node.iter().enumerate() {
            assert_eq!(*v, 7 + i as i32);
        }
        // Option<Box<ListNode>> -> Box<ListNode> -> i32, via flatten.
        assert_eq!(head.into_iter().flatten().collect::<Vec<_>>(), [7, 8]);
    }

    #[test]
    fn zip_two_lists() {
        let a = list![1, 2, 3];
        let b = list![10, 20, 30];
        let pairwise: Vec<i32> = a.values().zip(b.values()).map(|(x, y)| x + y).collect();
        assert_eq!(pairwise, [11, 22, 33]);
    }
}
