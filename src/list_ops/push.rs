use core::ptr::NonNull;
use crate::{RustyList, RustyListNode};

impl<T> RustyList<T> {
    /// Adds a node to the end (tail) of the list.
    ///
    /// This does not use the `order_function`, it always appends.
    ///
    /// # Safety
    /// - `item` must be a valid pointer to a `T` with an embedded `RustyListNode<T>`.
    pub fn push(&mut self, item: &mut T) {
        unsafe {
            self.push_raw(item as *mut T);
        }
    }

    /// Unsafe internal function to add a raw pointer to the end (tail) of the list.
    unsafe fn push_raw(&mut self, item: *mut T) {
        if item.is_null() {
            return;
        }

        let node_ptr = unsafe{(item as *mut u8).add(self.offset)} as *mut RustyListNode<T>;
        let node = unsafe{&mut *node_ptr};

        node.prev = None;
        node.next = None;

        let new_node =unsafe{ NonNull::new_unchecked(node_ptr)};

        if self.len == 0 {
            self.head = Some(new_node);
            self.tail = Some(new_node);
        } else {
            let tail_ptr = self.tail.unwrap().as_ptr();
            unsafe{(*tail_ptr).next = Some(new_node)};
           unsafe{ node.prev = Some(NonNull::new_unchecked(tail_ptr))};
            self.tail = Some(new_node);
        }

        self.len += 1;
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
    fn test_push_appends_to_tail() {
        let mut list = RustyList::<TestItem>::new();
        let mut a = make_item(10);
        let mut b = make_item(20);

  
            list.push(&mut a);
            list.push(&mut b);
       

        assert_eq!(list.len, 2);

        let head = list.head.unwrap().as_ptr();
        let tail = list.tail.unwrap().as_ptr();

        let head_val = unsafe { (*crate::rusty_container_of(head, list.offset)).value };
        let tail_val = unsafe { (*crate::rusty_container_of(tail, list.offset)).value };

        assert_eq!(head_val, 10);
        assert_eq!(tail_val, 20);
    }
}
