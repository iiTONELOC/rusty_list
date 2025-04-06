use crate::{RustyList,  rusty_container_of_mut};

impl<T> RustyList<T> {
    /// Removes and returns the first node in the list.
    ///
    /// # Safety
    /// - The returned pointer is to the outer `T`, not the node.
    /// - Caller must ensure the pointer is used safely.
    pub fn pop(&mut self) -> Option<*mut T> {
        unsafe { self.pop_raw() }
    }

    /// Unsafe internal function to remove the first node in the list.
    unsafe fn pop_raw(&mut self) -> Option<*mut T> {
        if self.len == 0 || self.head.is_none() {
            return None;
        }

        let node_ptr = self.head.unwrap().as_ptr();
        let node = unsafe{&mut *node_ptr};

        let next = node.next;

        self.head = next;

        if let Some(next_ptr) = next {
            unsafe{(*next_ptr.as_ptr()).prev = None};
        } else {
            // List is now empty
            self.tail = None;
        }

        node.next = None;
        node.prev = None;

        self.len -= 1;

       unsafe{ Some(rusty_container_of_mut(node_ptr, self.offset))}
    }
}


#[cfg(test)]
mod tests {
    use crate::{RustyList, RustyListNode, HasRustyNode, rusty_offset};

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct TestItem {
        pub value: i32,
        pub node: RustyListNode<TestItem>,
    }

    impl HasRustyNode for TestItem {
        fn rusty_offset() -> usize {
            rusty_offset(|x: &Self| &x.node)
        }
    }

    fn make_item(val: i32) -> TestItem {
        TestItem {
            value: val,
            node: RustyListNode::new(),
        }
    }

    #[test]
    fn test_pop_removes_head() {
        let mut list = RustyList::<TestItem>::new();
        let mut a = make_item(100);
        let mut b = make_item(200);

     
            list.push(&mut a);
            list.push(&mut b);
      

        assert_eq!(list.len, 2);

        let popped =list.pop() ;
        assert!(popped.is_some());
        assert_eq!(unsafe { (*popped.unwrap()).value }, 100);
        assert_eq!(list.len, 1);

        let popped2 = list.pop();
        assert!(popped2.is_some());
        assert_eq!(unsafe { (*popped2.unwrap()).value }, 200);
        assert_eq!(list.len, 0);

        assert!(list.head.is_none());
        assert!(list.tail.is_none());
    }
}
