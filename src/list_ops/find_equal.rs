use crate::{RustyList, RustyListNode, node_ref, rusty_container_of};
use core::cmp::Ordering;
use core::ptr::NonNull;

impl<'a, T, Tag> RustyList<'a, T, Tag> {
    /// The first linked item the comparator reports equal to `target`.
    /// `None` when nothing matches or the list has no comparator.
    #[must_use]
    pub fn find_equal(&self, target: &T) -> Option<&'a T> {
        let node = self.find_node(target)?;
        // SAFETY: the item is borrowed for `'a`, so a shared reference for `'a` is valid.
        Some(unsafe { &*rusty_container_of(node, self.offset) })
    }

    fn find_node(&self, target: &T) -> Option<NonNull<RustyListNode<T, Tag>>> {
        let order = self.order_function?;
        let mut current = self.head;
        while let Some(node) = current {
            // SAFETY: linked nodes lie inside items borrowed for `'a`.
            let item = unsafe { &*rusty_container_of(node, self.offset) };
            if order(item, target) == Ordering::Equal {
                return Some(node);
            }
            // SAFETY: `node` is linked, so it is live.
            current = unsafe { node_ref(node) }.next.get();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::RustyList;
    use crate::fixtures::{TestItem, cmp, make_item};

    #[test]
    fn find_returns_the_linked_item() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let a = make_item(1);
        let b = make_item(2);
        let c = make_item(3);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert_eq!(list.insert(&c), Ok(()));
        let found = list.find_equal(&make_item(2)).unwrap();
        assert!(core::ptr::eq(found, &b));
    }

    #[test]
    fn find_returns_none_for_missing_value() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert!(list.find_equal(&make_item(9)).is_none());
    }

    #[test]
    fn find_in_empty_list_returns_none() {
        let list = RustyList::<TestItem>::new_with_order(cmp);
        assert!(list.find_equal(&make_item(4)).is_none());
    }
}
