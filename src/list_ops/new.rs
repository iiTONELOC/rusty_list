use core::marker::PhantomData;
use crate::{RustyList, HasRustyNode, RustyListNode};

impl<T> RustyListNode<T> {
    /// Creates a new, non-dynamic list node with null prev/next (const version).
    pub const fn new_const() -> Self {
        Self {
            dynamic: false,
            _marker: PhantomData,
            prev: None,
            next: None,
        }
    }

    /// Creates a new, non-dynamic list node with null prev/next.
    pub fn new() -> Self {
        Self::new_const() // You can even make the original call the const one
    }

    /// Sets the `dynamic` property of the node and returns the modified instance.
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = dynamic;
        self
    }

    /// reset the node to initial state (not dynamic)
    pub fn clear_links(&mut self) {
        self.prev = None;
        self.next = None;
    }
}

/// Implementation of the `RustyList` struct for types that implement the `HasRustyNode` trait.
impl<T: HasRustyNode> RustyList<T> {
    
    /// Creates a new, empty `RustyList` instance.
    ///
    /// # Returns
    /// A new `RustyList` with default values:
    /// - `len` is set to 0.
    /// - `dynamic` is set to `false`.
    /// - `head` and `tail` are set to `None`.
    /// - `offset` is initialized using the `rusty_offset` method of the `HasRustyNode` trait.
    /// - `order_function` is set to `None`.
    pub fn new() -> Self {
        Self {
            len: 0,
            dynamic: false,
            head: None,
            tail: None,
            offset: T::rusty_offset(),
            order_function: None,
        }
    }

    /// Creates a new `RustyList` instance with a custom ordering function.
    ///
    /// # Parameters
    /// - `order`: A function pointer that takes two raw pointers to `T` and returns an `i32`.
    ///   This function defines the ordering of elements in the list.
    ///
    /// # Returns
    /// A new `RustyList` with the specified ordering function:
    /// - `len` is set to 0.
    /// - `dynamic` is set to `false`.
    /// - `head` and `tail` are set to `None`.
    /// - `offset` is initialized using the `rusty_offset` method of the `HasRustyNode` trait.
    /// - `order_function` is set to the provided `order` function.
    pub fn new_with_order(order: fn(*const T, *const T) -> i32) -> Self {
        Self {
            len: 0,
            dynamic: false,
            head: None,
            tail: None,
            offset: T::rusty_offset(),
            order_function: Some(order),
        }
    }

    /// Sets the `dynamic` property of the `RustyList` and returns the modified instance.
    ///
    /// # Parameters
    /// - `dynamic`: A boolean value indicating whether the list is dynamic.
    ///
    /// # Returns
    /// The modified `RustyList` instance with the `dynamic` property updated.
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = dynamic;
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::NonNull;
    use core::marker::PhantomData;
    use crate::{RustyListNode, rusty_offset};

    #[repr(C)]
    struct Dummy {
        id: u32,
        node: RustyListNode<Dummy>,
    }

    /// Implement the HasRustyNode trait for Dummy
    /// This allows the RustyList to know how to find the RustyListNode inside Dummy.
    impl HasRustyNode for Dummy {
        fn rusty_offset() -> usize {
            rusty_offset(|x: &Self| &x.node)
        }
    }

    /// Comparison function for sorting Dummy nodes in the list.
    fn dummy_cmp(a: *const Dummy, b: *const Dummy) -> i32 {
        unsafe {
            let a = &*a;
            let b = &*b;
            a.id.cmp(&b.id) as i32
        }
    }

    #[test]
    fn test_new_initializes_fields_correctly() {
        let list = RustyList::<Dummy>::new();

        assert_eq!(list.len, 0);
        assert!(!list.dynamic);
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
        assert!(list.order_function.is_none());

        // Sanity check on offset
        let expected_offset = Dummy::rusty_offset();
        assert_eq!(list.offset, expected_offset);
    }

    #[test]
    fn test_new_with_order_function() {
        let list = RustyList::<Dummy>::new_with_order(dummy_cmp);

        assert_eq!(list.len, 0);
        assert!(list.order_function.is_some());

        let a = Dummy {
            id: 1,
            node: RustyListNode {
                dynamic: false,
                _marker: PhantomData,
                prev: None,
                next: None,
            },
        };

        let b = Dummy {
            id: 2,
            node: RustyListNode {
                dynamic: false,
                _marker: PhantomData,
                prev: None,
                next: None,
            },
        };

        let cmp_fn = list.order_function.unwrap();
        let result = cmp_fn(&a as *const _, &b as *const _);
        assert!(result < 0);
    }

    #[test]
    fn test_with_dynamic_flag() {
        let list = RustyList::<Dummy>::new().with_dynamic(true);
        assert!(list.dynamic);

        let list = list.with_dynamic(false);
        assert!(!list.dynamic);
    }

    // ListNode tests
    #[test]
    fn test_node_new_defaults() {
        let node = RustyListNode::<u32>::new();
        assert!(!node.dynamic, "default node should not be dynamic");
        assert!(node.prev.is_none());
        assert!(node.next.is_none());
    }

    #[test]
    fn test_node_with_dynamic_true() {
        let node = RustyListNode::<u32>::new().with_dynamic(true);
        assert!(node.dynamic, "node should be marked dynamic");
    }

    #[test]
    fn test_node_with_dynamic_false() {
        let node = RustyListNode::<u32>::new().with_dynamic(false);
        assert!(!node.dynamic, "node should be non-dynamic");
    }

    #[test]
    fn test_node_clear_links() {
        // Pretend pointers (not dereferenced, just testing state change)
        let dummy_prev = 0x1 as *mut RustyListNode<u32>;
        let dummy_next = 0x2 as *mut RustyListNode<u32>;

        let mut node = RustyListNode {
            dynamic: false,
            _marker: PhantomData,
            prev: Some(unsafe { NonNull::new_unchecked(dummy_prev) }),
            next: Some(unsafe { NonNull::new_unchecked(dummy_next) }),
        };

        node.clear_links();
        assert!(node.prev.is_none());
        assert!(node.next.is_none());
    }
}
