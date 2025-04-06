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

    /// Unsafe internal function to insert a raw pointer into the `RustyList`.
    unsafe fn insert_raw(&mut self, item: *mut T) {
        if item.is_null() {
            return;
        }

        // SAFETY: We are assuming that the item is valid and properly aligned.
        // We are also assuming that the offset is valid and that the item is a valid pointer to T.
        let node_ptr = unsafe { (item as *mut u8).add(self.offset) } as *mut RustyListNode<T>;

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
            let cmp_fn = self.order_function;

            // Insert at tail if no sort or item > tail
            if cmp_fn.is_none() || {
                let tail_node = self.tail.unwrap().as_ptr();
                let tail_item = unsafe { rusty_container_of(tail_node, self.offset) };
                cmp_fn.unwrap()(item, tail_item) > 0
            } {
                // Link new node to tail
                // get the tail node pointer
                let tail_ptr = self.tail.unwrap().as_ptr();
                // set the next pointer of the tail node to the new node
                unsafe { (*tail_ptr).next = Some(new_node) };
                // set the prev pointer of the new node to the tail node
                node.prev = Some(unsafe { NonNull::new_unchecked(tail_ptr) });
                // set the tail pointer of the list to the new node
                self.tail = Some(new_node);
                // set the next pointer of the new node to None
                node.next = None;
            }
            // Insert at head if item < head
            else if {
                let head_node = self.head.unwrap().as_ptr();
                let head_item = unsafe { rusty_container_of(head_node, self.offset) };
                cmp_fn.unwrap()(item, head_item) < 0
            } {
                // Link new node to head
                // get the head node pointer
                let head_ptr = self.head.unwrap().as_ptr();
                // set the prev pointer of the head node to the new node
                unsafe { (*head_ptr).prev = Some(new_node) };
                // set the next pointer of the new node to the head node
                node.next = Some(unsafe { NonNull::new_unchecked(head_ptr) });
                // set the head pointer of the list to the new node
                self.head = Some(new_node);
                // set the prev pointer of the new node to None
                node.prev = None;
            }
            // Insert in the middle
            else {
                // hold the current node pointer
                let mut current = self.head.unwrap().as_ptr();

                // traverse the list to find the right position
                unsafe {
                    while let Some(next_ptr) = (*current).next {
                        // get the item pointer of the current node
                        let current_item = rusty_container_of(current, self.offset);

                        // if the item is less than the current item, break the loop
                        if cmp_fn.unwrap()(item, current_item) < 0 {
                            break;
                        }
                        // move to the next node
                        current = next_ptr.as_ptr();
                    }
                }

                // get the previous node
                let prev_ptr = unsafe { (*current).prev.unwrap().as_ptr() };

                // set the next pointer of the new node to the current node
                node.next = Some(unsafe { NonNull::new_unchecked(current) });
                // set the prev pointer of the new node to the previous node
                node.prev = Some(unsafe { NonNull::new_unchecked(prev_ptr) });
                // set the next pointer of the previous node to the new node
                unsafe { (*prev_ptr).next = Some(new_node) };
                // set the prev pointer of the current node to the new node
                unsafe { (*current).prev = Some(new_node) };
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
