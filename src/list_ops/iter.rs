use crate::{RustyList, RustyListNode, node_ref, rusty_container_of};
use core::marker::PhantomData;
use core::ptr::NonNull;

/// Iterator over the linked items of a `RustyList`, head to tail.
///
/// It borrows the list, so the chain cannot change underneath it.
pub struct Iter<'l, T, Tag> {
    current: Option<NonNull<RustyListNode<T, Tag>>>,
    offset: usize,
    _list: PhantomData<&'l T>,
}

impl<'l, T, Tag> Iterator for Iter<'l, T, Tag> {
    type Item = &'l T;

    fn next(&mut self) -> Option<&'l T> {
        let node = self.current?;
        // SAFETY: linked nodes lie inside items borrowed by the list, and the
        // list borrow held by this iterator keeps the chain unchanged for `'l`.
        unsafe {
            self.current = node_ref(node).next.get();
            Some(&*rusty_container_of(node, self.offset))
        }
    }
}

impl<T, Tag> RustyList<'_, T, Tag> {
    /// Iterates the linked items from head to tail.
    pub fn iter(&self) -> Iter<'_, T, Tag> {
        Iter {
            current: self.head,
            offset: self.offset,
            _list: PhantomData,
        }
    }
}

impl<'l, T, Tag> IntoIterator for &'l RustyList<'_, T, Tag> {
    type Item = &'l T;
    type IntoIter = Iter<'l, T, Tag>;

    fn into_iter(self) -> Iter<'l, T, Tag> {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::RustyList;
    use crate::fixtures::{TestItem, cmp, make_item};
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn iter_yields_head_to_tail() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let items = [make_item(3), make_item(1), make_item(2)];
        for item in &items {
            assert_eq!(list.insert(item), Ok(()));
        }
        let seen: Vec<i32> = list.iter().map(|item| item.value).collect();
        assert_eq!(seen, vec![1, 2, 3]);
    }

    #[test]
    fn iter_on_empty_list_yields_nothing() {
        let list = RustyList::<TestItem>::new();
        assert!(list.iter().next().is_none());
    }

    #[test]
    fn iter_reflects_removal() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let items = [make_item(1), make_item(2), make_item(3)];
        for item in &items {
            assert_eq!(list.insert(item), Ok(()));
        }
        assert_eq!(list.remove(&items[1]), Ok(()));
        let seen: Vec<i32> = list.iter().map(|item| item.value).collect();
        assert_eq!(seen, vec![1, 3]);
    }

    #[test]
    fn for_loop_over_list_reference() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(4);
        let b = make_item(5);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        let mut total = 0;
        for item in &list {
            total += item.value;
        }
        assert_eq!(total, 9);
    }
}
