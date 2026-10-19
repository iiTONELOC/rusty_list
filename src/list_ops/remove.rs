use crate::{ListError, RustyList, RustyListNode, node_ref};
use core::ptr::NonNull;

impl<T, Tag> RustyList<'_, T, Tag> {
    /// Unlinks `item` from this list.
    ///
    /// Refused when `item` is not linked, or is linked into a different list.
    pub fn remove(&mut self, item: &T) -> Result<(), ListError> {
        let node = self.node_of(item);
        // SAFETY: `node` lies inside `item`, which is live for this call.
        if unsafe { node_ref(node) }.owner.get() != self.id {
            return Err(ListError::NotLinked);
        }
        self.unlink(node);
        self.len -= 1;
        Ok(())
    }

    pub(crate) fn unlink(&mut self, node: NonNull<RustyListNode<T, Tag>>) {
        // SAFETY: `node` and its neighbors lie inside items borrowed for `'a`.
        unsafe {
            let prev = node_ref(node).prev.get();
            let next = node_ref(node).next.get();
            match prev {
                Some(prev) => node_ref(prev).next.set(next),
                None => self.head = next,
            }
            match next {
                Some(next) => node_ref(next).prev.set(prev),
                None => self.tail = prev,
            }
            node_ref(node).clear_links();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::{TestItem, cmp, make_item, values};
    use crate::{ListError, RustyList, UNLINKED};
    use std::vec;

    #[test]
    fn remove_only_node_empties_list() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.len(), 1);
        assert_eq!(list.remove(&a), Ok(()));
        assert!(list.is_empty());
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
    }

    #[test]
    fn remove_head_preserves_tail() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert_eq!(list.remove(&a), Ok(()));
        assert_eq!(values(&list), vec![2]);
        assert_eq!(a.node.owner.get(), UNLINKED);
        assert!(b.node.prev.get().is_none());
        assert!(b.node.next.get().is_none());
    }

    #[test]
    fn remove_tail_preserves_head() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert_eq!(list.remove(&b), Ok(()));
        assert_eq!(values(&list), vec![1]);
        assert!(a.node.prev.get().is_none());
        assert!(a.node.next.get().is_none());
    }

    #[test]
    fn remove_middle_preserves_links() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let a = make_item(1);
        let b = make_item(2);
        let c = make_item(3);
        assert_eq!(list.insert(&a), Ok(()));
        assert_eq!(list.insert(&b), Ok(()));
        assert_eq!(list.insert(&c), Ok(()));
        assert_eq!(list.remove(&b), Ok(()));
        assert_eq!(list.len(), 2);
        assert_eq!(values(&list), vec![1, 3]);
        let before_c = c.node.prev.get().unwrap();
        assert!(core::ptr::eq(before_c.as_ptr(), &raw const a.node));
    }

    #[test]
    fn remove_non_member_returns_not_linked() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        let c = make_item(3);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        assert_eq!(list.remove(&c), Err(ListError::NotLinked));
        assert_eq!(list.len(), 2);
        assert_eq!(values(&list), vec![1, 2]);
    }

    #[test]
    fn remove_from_empty_list_returns_not_linked() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.remove(&a), Err(ListError::NotLinked));
        assert!(list.is_empty());
    }

    #[test]
    fn remove_without_order_works() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        assert_eq!(list.remove(&b), Ok(()));
        assert_eq!(values(&list), vec![1]);
    }
}
