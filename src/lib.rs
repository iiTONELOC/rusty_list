#![no_std]

#[cfg(test)]
extern crate std;

mod core_types;      // RustyListNode, RustyList, traits, offset helpers
mod list_ops;        // insert, remove, pop, push, etc.

#[allow(unused_imports)]
pub use core_types::*;
#[allow(unused_imports)]
pub use list_ops::{
    insert::*,
    remove::*,
    find_equal::*,
    pop::*,
    push::*,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct TestItem {
        value: i32,
        node: RustyListNode<TestItem>,
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
    fn test_insert_sorted_and_order_is_correct() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let mut items = [
            make_item(30),
            make_item(10),
            make_item(50),
            make_item(40),
            make_item(20),
        ];

        for item in &mut items {
            list.insert(item);
        }

        assert_eq!(list.len, 5);

        let mut current = list.head;
        let mut values = vec![];
        while let Some(node) = current {
            let item = unsafe { &*rusty_container_of(node.as_ptr(), list.offset) };
            values.push(item.value);
            current = unsafe { (*node.as_ptr()).next };
        }

        assert_eq!(values, vec![10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_find_equal_works() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let mut items = [
            make_item(10),
            make_item(20),
            make_item(30),
        ];

        for item in &mut items {
            list.insert(item);
        }

        let target = make_item(20);
        let result = list.find_equal(&target);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, 20);
    }

    #[test]
    fn test_remove_middle_node() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let mut items = [
            make_item(10),
            make_item(20),
            make_item(30),
        ];

        for item in &mut items {
            list.insert(item);
        }

        let target = make_item(20);
        let found_ptr = {
            let temp = list.find_equal(&target).unwrap();
            temp as *mut TestItem
        };

        list.remove(unsafe { &mut *found_ptr });

        let mut current = list.head;
        let mut values = vec![];
        while let Some(node) = current {
            let item = unsafe { &*rusty_container_of(node.as_ptr(), list.offset) };
            values.push(item.value);
            current = unsafe { (*node.as_ptr()).next };
        }

        assert_eq!(list.len, 2);
        assert_eq!(values, vec![10, 30]);
    }

    #[test]
    fn test_push_appends_to_tail() {
        let mut list = RustyList::<TestItem>::new();
        let mut a = make_item(1);
        let mut b = make_item(2);

        list.push(&mut a);
        list.push(&mut b);

        assert_eq!(list.len, 2);

        let head_val = unsafe {
            let node = list.head.unwrap().as_ptr();
            (*rusty_container_of(node, list.offset)).value
        };

        let tail_val = unsafe {
            let node = list.tail.unwrap().as_ptr();
            (*rusty_container_of(node, list.offset)).value
        };

        assert_eq!(head_val, 1);
        assert_eq!(tail_val, 2);
    }

    #[test]
    fn test_pop_removes_head_and_returns_correct_item() {
        let mut list = RustyList::<TestItem>::new();
        let mut items = [make_item(10), make_item(20)];

        list.push(&mut items[0]);
        list.push(&mut items[1]);

        let popped = list.pop();
        assert!(popped.is_some());
        assert_eq!(unsafe { (*popped.unwrap()).value }, 10);
        assert_eq!(list.len, 1);

        let popped2 = list.pop();
        assert!(popped2.is_some());
        assert_eq!(unsafe { (*popped2.unwrap()).value }, 20);
        assert_eq!(list.len, 0);

        assert!(list.head.is_none());
        assert!(list.tail.is_none());
    }
}
