//! Smart pointers: Box, Rc, and Weak.
//!
//! A smart pointer is a struct that owns data and behaves like a
//! reference (via the Deref trait) while adding a capability:
//! * `Box<T>`  — heap allocation, single owner. Capability: indirection.
//! * `Rc<T>`   — reference counting, many owners, single thread.
//! * `Arc<T>`  — like Rc but atomic, for multiple threads (see the
//!   concurrency crate).
//! * `Weak<T>` — a non-owning Rc handle that breaks reference cycles.

use std::rc::{Rc, Weak};

/// Box's killer feature: recursive types. Without indirection, the
/// compiler cannot size this enum — a List containing a List
/// containing a List... has infinite size. Box makes the size one
/// pointer, breaking the recursion.
#[derive(Debug, PartialEq)]
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    pub fn from_slice(items: &[i32]) -> List {
        // Build back-to-front so each node wraps the rest.
        items
            .iter()
            .rev()
            .fold(List::Nil, |rest, &item| List::Cons(item, Box::new(rest)))
    }

    pub fn sum(&self) -> i32 {
        match self {
            List::Cons(value, rest) => value + rest.sum(),
            List::Nil => 0,
        }
    }
}

/// Rc: shared ownership. `clone` does NOT copy the data — it bumps a
/// counter and hands back another owner of the SAME allocation. The
/// data drops when the last owner does. Use it when ownership is
/// genuinely shared and no single owner makes sense (graphs, caches,
/// shared config).
pub fn shared_config_example() -> (usize, usize) {
    let config = Rc::new(String::from("max_retries=3"));
    let count_before = Rc::strong_count(&config);

    // Two "subsystems" each keep their own handle.
    let parser_handle = Rc::clone(&config); // idiomatic: Rc::clone, not config.clone()
    let logger_handle = Rc::clone(&config);

    let count_during = Rc::strong_count(&config);
    drop(parser_handle);
    drop(logger_handle);

    (count_before, count_during) // (1, 3)
}

/// Weak: breaking cycles. A tree where children point back at parents
/// would leak with Rc in both directions (the counts never hit zero).
/// The rule: ownership direction uses Rc, the back-edge uses Weak.
pub struct TreeNode {
    pub value: i32,
    // Weak does not keep the parent alive — and that's the point:
    // parents own children, not vice versa.
    pub parent: std::cell::RefCell<Weak<TreeNode>>,
    pub children: std::cell::RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    pub fn new(value: i32) -> Rc<TreeNode> {
        Rc::new(TreeNode {
            value,
            parent: std::cell::RefCell::new(Weak::new()),
            children: std::cell::RefCell::new(Vec::new()),
        })
    }

    pub fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
        // Downgrade: Rc -> Weak for the back-edge.
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    /// Upgrade: Weak -> Option<Rc>. None means the parent is gone —
    /// Weak forces you to handle that case, which is exactly the
    /// safety Rc cycles would have silently broken.
    pub fn parent_value(&self) -> Option<i32> {
        self.parent.borrow().upgrade().map(|p| p.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_enables_recursive_types() {
        let list = List::from_slice(&[1, 2, 3]);
        assert_eq!(list.sum(), 6);
        assert_eq!(List::from_slice(&[]), List::Nil);
    }

    #[test]
    fn rc_counts_owners_not_copies() {
        assert_eq!(shared_config_example(), (1, 3));
    }

    #[test]
    fn weak_back_edges_dont_own() {
        let root = TreeNode::new(1);
        let leaf = TreeNode::new(2);
        TreeNode::add_child(&root, Rc::clone(&leaf));

        assert_eq!(leaf.parent_value(), Some(1));

        // Drop the tree; only our direct handle on `leaf` remains.
        drop(root);
        // The Weak parent pointer now upgrades to None instead of
        // dangling — no leak, no use-after-free.
        assert_eq!(leaf.parent_value(), None);
    }
}

// Exercises
// ---------
// 1. Add a `len` method to List. Then try `impl Iterator` for a
//    borrowing ListIter — harder than it looks, instructive either way.
// 2. Build two TreeNodes that point at each other with Rc in BOTH
//    directions and explain in a comment why neither is ever freed.
// 3. When would you reach for Box<dyn Trait> vs Rc<dyn Trait>? Write
//    one example signature for each.
