#![no_std]

#[cfg(test)]
extern crate std;

mod core_types;
mod list_ops;

pub use core_types::*;
pub use list_ops::iter::Iter;

#[cfg(test)]
pub(crate) mod fixtures {
    use crate::{HasRustyNode, RustyList, RustyListNode};
    use core::cmp::Ordering;
    use std::vec::Vec;

    #[derive(Debug, PartialEq)]
    pub(crate) struct Alt;

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct TestItem {
        pub(crate) value: i32,
        pub(crate) node: RustyListNode<TestItem>,
        pub(crate) alt: RustyListNode<TestItem, Alt>,
    }

    // SAFETY: `node` is the embedded `RustyListNode<Self>` and `offset_of!` reports its offset.
    unsafe impl HasRustyNode for TestItem {
        fn rusty_offset() -> usize {
            core::mem::offset_of!(Self, node)
        }
    }

    // SAFETY: `alt` is the embedded `RustyListNode<Self, Alt>` and `offset_of!` reports its offset.
    unsafe impl HasRustyNode<Alt> for TestItem {
        fn rusty_offset() -> usize {
            core::mem::offset_of!(Self, alt)
        }
    }

    pub(crate) fn cmp(a: &TestItem, b: &TestItem) -> Ordering {
        a.value.cmp(&b.value)
    }

    pub(crate) const fn make_item(value: i32) -> TestItem {
        TestItem {
            value,
            node: RustyListNode::new(),
            alt: RustyListNode::new(),
        }
    }

    pub(crate) fn values<Tag>(list: &RustyList<'_, TestItem, Tag>) -> Vec<i32> {
        list.iter().map(|item| item.value).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{Alt, TestItem, cmp, make_item, values};
    use super::{ListError, RustyList};
    use core::cell::UnsafeCell;
    use std::vec;

    #[test]
    fn insert_sorted_keeps_order() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let items = [
            make_item(3),
            make_item(1),
            make_item(5),
            make_item(4),
            make_item(2),
        ];
        for item in &items {
            assert_eq!(list.insert(item), Ok(()));
        }
        assert_eq!(list.len(), 5);
        assert_eq!(values(&list), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn find_then_remove_unlinks_that_item() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let items = [make_item(1), make_item(2), make_item(3)];
        for item in &items {
            assert_eq!(list.insert(item), Ok(()));
        }
        let found = list.find_equal(&make_item(2)).unwrap();
        assert!(core::ptr::eq(found, &items[1]));
        assert_eq!(list.remove(found), Ok(()));
        assert_eq!(list.len(), 2);
        assert_eq!(values(&list), vec![1, 3]);
    }

    #[test]
    fn remove_then_reinsert_succeeds() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.remove(&a), Ok(()));
        assert!(list.is_empty());
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.len(), 1);
        assert_eq!(values(&list), vec![1]);
    }

    #[test]
    fn pop_then_push_succeeds() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.push(&a), Ok(()));
        let popped = list.pop().unwrap();
        assert!(list.is_empty());
        assert_eq!(list.push(popped), Ok(()));
        assert_eq!(values(&list), vec![1]);
    }

    #[test]
    fn insert_without_order_appends() {
        let mut list = RustyList::<TestItem>::new();
        let items = [make_item(3), make_item(1), make_item(2)];
        for item in &items {
            assert_eq!(list.insert(item), Ok(()));
        }
        assert_eq!(values(&list), vec![3, 1, 2]);
    }

    #[test]
    fn equal_keys_insert_after_existing() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let a = make_item(1);
        let b = make_item(2);
        let c = make_item(3);
        let d = make_item(2);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert_eq!(list.insert(&c), Ok(()));
        assert_eq!(list.insert(&d), Ok(()));
        assert_eq!(values(&list), vec![1, 2, 2, 3]);
        let after_b = b.node.next.get().unwrap();
        assert!(core::ptr::eq(after_b.as_ptr(), &raw const d.node));
    }

    #[test]
    fn pop_on_empty_returns_none() {
        let mut list = RustyList::<TestItem>::new();
        assert!(list.pop().is_none());
    }

    #[test]
    fn find_equal_without_order_returns_none() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.push(&a), Ok(()));
        assert!(list.find_equal(&make_item(1)).is_none());
    }

    #[test]
    fn item_in_two_lists_at_once() {
        let mut by_value = RustyList::<TestItem>::new_with_order(cmp);
        let mut by_arrival = RustyList::<TestItem, Alt>::new();
        let items = [make_item(3), make_item(1), make_item(2)];
        for item in &items {
            assert_eq!(by_value.insert(item), Ok(()));
            assert_eq!(by_arrival.push(item), Ok(()));
        }
        assert_eq!(values(&by_value), vec![1, 2, 3]);
        assert_eq!(values(&by_arrival), vec![3, 1, 2]);
        assert_eq!(by_value.remove(&items[0]), Ok(()));
        assert_eq!(values(&by_value), vec![1, 2]);
        assert_eq!(values(&by_arrival), vec![3, 1, 2]);
    }

    #[test]
    fn insert_linked_item_is_refused() {
        let mut first = RustyList::<TestItem>::new();
        let mut second = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(first.push(&a), Ok(()));
        assert_eq!(first.insert(&a), Err(ListError::AlreadyLinked));
        assert_eq!(second.push(&a), Err(ListError::AlreadyLinked));
        assert_eq!(first.len(), 1);
        assert!(second.is_empty());
    }

    #[test]
    fn remove_unlinked_item_is_refused() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.remove(&b), Err(ListError::NotLinked));
        assert_eq!(values(&list), vec![1]);
    }

    #[test]
    fn remove_from_other_list_is_refused() {
        let mut first = RustyList::<TestItem>::new();
        let mut second = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        let c = make_item(3);
        assert_eq!(first.push(&a), Ok(()));
        assert_eq!(first.push(&b), Ok(()));
        assert_eq!(second.remove(&b), Err(ListError::NotLinked));
        assert_eq!(second.push(&c), Ok(()));
        assert_eq!(values(&first), vec![1, 2]);
        assert_eq!(values(&second), vec![3]);
        assert!(b.node.next.get().is_none());
    }

    struct SyncCell<T>(UnsafeCell<T>);

    // SAFETY: each cell is used by one test, and that test runs on one thread.
    unsafe impl<T> Sync for SyncCell<T> {}

    static ITEMS: SyncCell<[TestItem; 3]> =
        SyncCell(UnsafeCell::new([make_item(2), make_item(1), make_item(3)]));

    static LIST: SyncCell<RustyList<'static, TestItem>> = SyncCell(UnsafeCell::new(RustyList {
        id: usize::MAX,
        len: 0,
        dynamic: false,
        head: None,
        tail: None,
        offset: core::mem::offset_of!(TestItem, node),
        order_function: Some(cmp),
        _items: core::marker::PhantomData,
    }));

    #[test]
    fn static_storage_end_to_end() {
        // SAFETY: the statics live for the whole program and only this test touches them.
        let (items, list) = unsafe { (&*ITEMS.0.get(), &mut *LIST.0.get()) };
        for item in items {
            assert_eq!(list.insert(item), Ok(()));
        }
        assert_eq!(values(list), vec![1, 2, 3]);
        for item in items {
            assert_eq!(list.remove(item), Ok(()));
        }
        assert!(list.is_empty());
    }
}
