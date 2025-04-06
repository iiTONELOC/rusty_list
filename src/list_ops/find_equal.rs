use crate::RustyList;
use crate::HasRustyNode;
use crate::rusty_container_of;

impl<T: HasRustyNode> RustyList<T> {
    /// Safe version of `find_equal`, compares items using the order function.
    pub fn find_equal(&self, target: &T) -> Option<&mut T> {
        let raw_ptr = target as *const T;
        unsafe { self.find_equal_raw(raw_ptr).map(|p| &mut *p) }
    }

    /// Internal unsafe implementation of find_equal
    unsafe fn find_equal_raw(&self, target: *const T) -> Option<*mut T> {
        if target.is_null() || self.len == 0 || self.order_function.is_none() {
            return None;
        }

        let mut current = self.head.map(|nn| nn.as_ptr());

        while let Some(node_ptr) = current {
            let current_item = unsafe{rusty_container_of(node_ptr, self.offset)};
            let cmp = self.order_function.unwrap()(current_item, target);

            if cmp == 0 {
                return Some(current_item as *mut T);
            }

            current = unsafe{(*node_ptr).next.map(|nn| nn.as_ptr())};
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustyListNode, rusty_offset};

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

    fn cmp(a: *const TestItem, b: *const TestItem) -> i32 {
        unsafe {
            let a = &*a;
            let b = &*b;
            a.value.cmp(&b.value) as i32
        }
    }

    fn make_item(val: i32) -> TestItem {
        TestItem {
            value: val,
            node: RustyListNode::new(),
        }
    }

    #[test]
    fn find_existing_node_by_value() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);

        let mut a = make_item(1);
        let mut b = make_item(2);
        let mut c = make_item(3);

        list.insert(&mut a);
        list.insert(&mut b);
        list.insert(&mut c);

        let target = make_item(2);
        let found = list.find_equal(&target);
        assert!(found.is_some());
        assert_eq!(found.unwrap().value, 2);
    }

    #[test]
    fn find_returns_none_for_missing_value() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);

        let mut a = make_item(10);
        let mut b = make_item(20);

        list.insert(&mut a);
        list.insert(&mut b);

        let target = make_item(99);
        let result = list.find_equal(&target);
        assert!(result.is_none());
    }

    #[test]
    fn find_in_empty_list() {
        let list = RustyList::<TestItem>::new_with_order(cmp);
        let target = make_item(42);
        let result = list.find_equal(&target);
        assert!(result.is_none());
    }
}
