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
    /// Returns `None` for an empty slice.
    pub fn from_slice(vals: &[i32]) -> Option<Box<ListNode>> {
        vals.iter()
            .rev()
            .fold(None, |next, &val| Some(Box::new(ListNode { val, next })))
    }

    /// Collect a list back into a `Vec` for assertions.
    pub fn to_vec(mut head: Option<Box<ListNode>>) -> Vec<i32> {
        let mut out = Vec::new();
        while let Some(node) = head {
            out.push(node.val);
            head = node.next;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let list = ListNode::from_slice(&[1, 2, 3]);
        assert_eq!(ListNode::to_vec(list), vec![1, 2, 3]);
    }

    #[test]
    fn empty() {
        assert_eq!(ListNode::from_slice(&[]), None);
        assert_eq!(ListNode::to_vec(None), Vec::<i32>::new());
    }
}
