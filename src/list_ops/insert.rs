use crate::{RustyList, RustyListNode, rusty_container_of};
use core::ptr::NonNull;

impl<T> RustyList<T> {
    /// Inserts a new node into the `RustyList` at the appropriate position based on the
    /// sorting order defined by the `order_function`. If no sorting function is provided,
    /// the node is appended to the end of the list.
    ///
    /// # Safety
    /// This function is marked as `unsafe` because it operates on raw pointers and assumes
    /// that the provided `item` pointer is valid and properly aligned. The caller must ensure
    /// that the `item` pointer is not null and points to a valid memory location.
    ///
    /// # Parameters
    /// - `item`: A raw pointer to the item to be inserted into the list. The item must be
    ///   properly aligned and valid for the lifetime of the list.
    ///
    /// # Behavior
    /// - If the list is empty, the new node becomes both the head and the tail of the list.
    /// - If the list is not empty:
    ///   - The node is appended to the tail if no sorting function is provided or if the
    ///     item is greater than the current tail.
    ///   - The node is prepended to the head if the item is less than the current head.
    ///   - Otherwise, the node is inserted in the middle of the list at the appropriate
    ///     position based on the sorting function.
    ///
    /// # Notes
    /// - The `order_function` is expected to be a comparison function that takes two raw
    ///   pointers and returns an `i32` indicating the comparison result:
    ///   - A positive value indicates the first item is greater than the second.
    ///   - A negative value indicates the first item is less than the second.
    ///   - Zero indicates the items are equal.
    /// - The function uses the `rusty_container_of` utility to retrieve the container
    ///   structure from the raw node pointer.
    ///
    /// # Panics
    /// This function does not panic, but improper usage of raw pointers or invalid memory
    /// can lead to undefined behavior.
    ///
    ///
    pub fn insert(&mut self, item: &mut T) {
        unsafe {
            self.insert_raw(item as *mut T);
        }
    }

    unsafe fn insert_node_at_head(&mut self, node: *mut RustyListNode<T>) {
        let new_node = unsafe { NonNull::new_unchecked(node) };
        if self.len == 0 {
            self.head = Some(new_node);
            self.tail = Some(new_node);
            unsafe {
                (*node).prev = None;
                (*node).next = None
            };
        } else {
            // set the next pointer of the new node to the current head
            unsafe { (*node).next = self.head };
            // set the prev pointer of the current head to the new node
            unsafe { (*self.head.unwrap().as_ptr()).prev = Some(new_node) };
            // set the head pointer of the list to the new node
            self.head = Some(new_node);
            // set the prev pointer of the new node to None
            unsafe { (*node).prev = None };
        }
    }

    unsafe fn _insert_node_at_tail(&mut self, node: *mut RustyListNode<T>) {
        let new_node = unsafe { NonNull::new_unchecked(node) };

     

        // set the next pointer of the current tail node to the new node
        unsafe { (*self.tail.unwrap().as_ptr()).next = Some(new_node) };
        // set the prev pointer of the new node to the current tail
        unsafe { (*node).prev = Some(self.tail.unwrap()) };
        // set the tail of the list to the new node
        self.tail = Some(new_node);
        // set the next pointer of the new node to None
        unsafe { (*node).next = None };

        // if the list has only one node, set the heads next pointer to the new node
        if self.len == 1 {
            unsafe { (*self.head.unwrap().as_ptr()).next = Some(new_node) };
        }
    }

    /// Unsafe internal function to insert a raw pointer into the `RustyList`.
    unsafe fn insert_raw(&mut self, item: *mut T) {
        if item.is_null() {
            return;
        }

        // SAFETY: We are assuming that the item is valid and properly aligned.
        // We are also assuming that the offset is valid and that the item is a valid pointer to T.
        let node_ptr = unsafe { (item as *mut u8).add(self.offset) } as *mut RustyListNode<T>;
        let item_container = unsafe { rusty_container_of(node_ptr, self.offset) };

        let node = unsafe { &mut *node_ptr };
        node.prev = None;
        node.next = None;

        let new_node = unsafe { NonNull::new_unchecked(node_ptr) };

        if self.len == 0 {
            // List is empty
            self.head = Some(new_node);
            self.tail = Some(new_node);

            // set the next and prev pointers to None
            node.prev = None;
            node.next = None;
        } else {
            // list is not empty find the correct position to insert the new node
            let cmp_fn = self.order_function;
            let mut current = self.head.unwrap().as_ptr();

            // if there is no order function OR the new node should be inserted at the tail
            if cmp_fn.is_none() || {
                let tail_node = self.tail.unwrap().as_ptr();
                let tail_item = unsafe { rusty_container_of(tail_node, self.offset) };
                cmp_fn.unwrap()(item_container, tail_item) > 0
            } {
                // Insert at tail
                unsafe { self._insert_node_at_tail(node_ptr) };
            }
            // if we have an order function and the new node should be inserted at the head
            else if {
                let head_node = self.head.unwrap().as_ptr();
                let head_item = unsafe { rusty_container_of(head_node, self.offset) };
                cmp_fn.unwrap()(item_container, head_item) < 0
            } {
                // Insert at head
                unsafe { self.insert_node_at_head(node_ptr) };
            }
            // other wise we are inserting in the middle of the list
            else {
                while !current.is_null() {
                    // look for a position to insert the new node
                    let current_item = unsafe { rusty_container_of(current, self.offset) };

                    // if the new item is less than the current item, break the loop
                    if cmp_fn.unwrap()(item_container, current_item) < 0 {
                        break;
                    }
                    // move to the next node
                    current = unsafe {
                        match (*current).next {
                            Some(next_node) => next_node.as_ptr(),
                            None => core::ptr::null_mut(),
                        }
                    };
                }

                // check to see if current is null, this should not happen but just in case
                if current.is_null() {
                    // Insert at tail
                    unsafe { self._insert_node_at_tail(node_ptr) };
                } else {
                    // Insert in the middle
                    let prev_ptr = unsafe { (*current).prev.unwrap().as_ptr() };
                    // set the pointer of the new node to the current node
                    unsafe { (*node_ptr).next = Some(NonNull::new_unchecked(current)) };
                    // set the prev pointer of the new node to the previous node
                    unsafe { (*node_ptr).prev = Some(NonNull::new_unchecked(prev_ptr)) };
                    // set the next pointer of the previous node to the new node
                    unsafe { (*prev_ptr).next = Some(new_node) };
                    // set the prev pointer of the current node to the new node
                    unsafe { (*current).prev = Some(new_node) };
                }
            }
        }
        self.len += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HasRustyNode, RustyList, RustyListNode, rusty_offset};
    use core::marker::PhantomData;
    use std::vec;

    #[repr(C)]
    #[derive(Debug)]
    struct TestItem {
        pub value: i32,
        pub node: RustyListNode<TestItem>,
    }

    /// Implement the HasRustyNode trait for TestItem
    /// This allows the RustyList to know how to find the RustyListNode inside TestItem.
    impl HasRustyNode for TestItem {
        fn rusty_offset() -> usize {
            rusty_offset(|x: &TestItem| &x.node)
        }
    }

    /// Comparison function for sorting TestItem nodes in the list.
    /// This function is used to determine the order of items in the list.
    fn cmp(a: *const TestItem, b: *const TestItem) -> i32 {
        unsafe {
            let a = &*a;
            let b = &*b;
            a.value.cmp(&b.value) as i32
        }
    }

    #[test]
    fn insert_sorted_into_list() {
        let mut list = RustyList::<TestItem> {
            len: 0,
            dynamic: false,
            head: None,
            tail: None,
            offset: TestItem::rusty_offset(),
            order_function: Some(cmp),
        };

        let mut one = TestItem {
            value: 1,
            node: RustyListNode {
                dynamic: false,
                _marker: PhantomData,
                prev: None,
                next: None,
            },
        };

        let mut three = TestItem {
            value: 3,
            node: RustyListNode {
                dynamic: false,
                _marker: PhantomData,
                prev: None,
                next: None,
            },
        };

        let mut two = TestItem {
            value: 2,
            node: RustyListNode {
                dynamic: false,
                _marker: PhantomData,
                prev: None,
                next: None,
            },
        };

        list.insert(&mut three);
        list.insert(&mut one);
        list.insert(&mut two);

        assert_eq!(list.len, 3);

        // Walk and verify order is 1 → 2 → 3
        let mut cursor = list.head;
        let mut values = std::vec::Vec::new();

        while let Some(ptr) = cursor {
            let item = unsafe { rusty_container_of(ptr.as_ptr(), list.offset) };
            values.push(unsafe { (*item).value });
            cursor = unsafe { (*ptr.as_ptr()).next };
        }

        assert_eq!(values, vec![1, 2, 3]);
    }
}
