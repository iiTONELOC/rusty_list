use crate::{RustyList, RustyListNode};

impl<T> RustyList<T> {
    /// Removes a node from the list.
    ///
    /// # Safety
    /// - `item` must be a valid, non-null pointer to a `T` that contains a `RustyListNode<T>`.
    /// - The `offset` field of the list must be correct.
    pub fn remove(&mut self, item: &mut T) {
        unsafe {
            self.remove_raw(item as *mut T);
        }
    }

    /// Unsafe internal function to remove a raw pointer from the list.
    unsafe fn remove_raw(&mut self, item: *mut T) {
        if item.is_null() || self.len == 0 {
            return;
        }

        // Get pointer to RustyListNode<T> inside item
        let node_ptr = unsafe{(item as *mut u8).add(self.offset)} as *mut RustyListNode<T>;
        let node =unsafe{ &mut *node_ptr};

        let node_next = node.next.map(|nn| nn.as_ptr());
        let node_prev = node.prev.map(|nn| nn.as_ptr());

        // If this is the head
        if self.head.map(|h| h.as_ptr()) == Some(node_ptr) {
            // set the head pointer to the next node
            self.head = node.next;
            // If there is a next node, set its prev pointer to None
            if let Some(new_head_ptr) = self.head {
                unsafe {(*new_head_ptr.as_ptr()).prev = None};
            }
        }

        // If this is the tail
        if self.tail.map(|t| t.as_ptr()) == Some(node_ptr) {
            // set the tail pointer to the prev node
            self.tail = node.prev;
            // If there is a prev node, set its next pointer to None
            if let Some(new_tail_ptr) = self.tail {
                unsafe {(*new_tail_ptr.as_ptr()).next = None};
            }
        }

        // Middle node re-linking
        // if the prev node exists, set its next pointer to the next node
        if let Some(prev_ptr) = node_prev {
            unsafe {(*prev_ptr).next = node.next};
        }

        // if the next node exists, set its prev pointer to the prev node
        if let Some(next_ptr) = node_next {
            unsafe{(*next_ptr).prev = node.prev};
        }

        // Clear the removed node's links
        node.prev = None;
        node.next = None;

        // Decrement list length
        self.len -= 1;

        // Final cleanup if list is empty
        if self.len == 0 {
            self.head = None;
            self.tail = None;
        }
    }
}



#[cfg(test)]
mod tests {
    use std::vec;
    use crate::{RustyList, RustyListNode, HasRustyNode, rusty_offset};


    #[repr(C)]
    #[derive(Debug)]
    struct TestItem {
        pub value: i32,
        pub node: RustyListNode<TestItem>,
    }

    impl HasRustyNode for TestItem {
        fn rusty_offset() -> usize {
            rusty_offset(|x: &Self| &x.node)
        }
    }

    fn cmp(a: *const TestItem, b: *const TestItem) -> i32 {
        unsafe { (*a).value.cmp(&(*b).value) as i32 }
    }

    fn make_item(val: i32) -> TestItem {
        TestItem {
            value: val,
            node: RustyListNode::new(),
        }
    }

    #[test]
    fn remove_only_node_resets_list() {
        let mut list = RustyList::<TestItem>::new();
        let mut item = make_item(1);

      
            list.insert(&mut item);
            assert_eq!(list.len, 1);
            list.remove(&mut item);
        

        assert_eq!(list.len, 0);
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
    }

    #[test]
    fn remove_head_preserves_tail() {
        let mut list = RustyList::<TestItem>::new();
        let mut a = make_item(1);
        let mut b = make_item(2);

  
            list.insert(&mut a);
            list.insert(&mut b); // b should go after a
            list.remove(&mut a);
       

        assert_eq!(list.len, 1);
        let head = unsafe { &*list.head.unwrap().as_ptr() };
        assert!(head.prev.is_none());
    }

    #[test]
    fn remove_tail_preserves_head() {
        let mut list = RustyList::<TestItem>::new();
        let mut a = make_item(1);
        let mut b = make_item(2);

        
            list.insert(&mut a);
            list.insert(&mut b);
            list.remove(&mut b);
      

        assert_eq!(list.len, 1);
        let tail = unsafe { &*list.tail.unwrap().as_ptr() };
        assert!(tail.next.is_none());
    }

    #[test]
    fn remove_middle_preserves_links() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let mut a = make_item(1);
        let mut b = make_item(2);
        let mut c = make_item(3);

    
            list.insert(&mut a);
            list.insert(&mut b);
            list.insert(&mut c);
            assert_eq!(list.len, 3);

            list.remove(&mut b);
        

        assert_eq!(list.len, 2);

        // walk head → tail
        let mut vals = vec![];
        let mut cursor = list.head;

        while let Some(ptr) = cursor {
            let item = unsafe { crate::rusty_container_of(ptr.as_ptr(), list.offset) };
            vals.push(unsafe { (*item).value });
            cursor = unsafe { (*ptr.as_ptr()).next };
        }

        assert_eq!(vals, vec![1, 3]);
    }
}

