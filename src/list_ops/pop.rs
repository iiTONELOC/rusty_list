use crate::{RustyList, rusty_container_of};

impl<'a, T, Tag> RustyList<'a, T, Tag> {
    /// Unlinks the head item and returns it.
    #[must_use]
    pub fn pop(&mut self) -> Option<&'a T> {
        let head = self.head?;
        self.unlink(head);
        self.len -= 1;
        // SAFETY: the item is borrowed for `'a`, so a shared reference for `'a` is valid.
        Some(unsafe { &*rusty_container_of(head, self.offset) })
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::{TestItem, make_item};
    use crate::{RustyList, UNLINKED};

    #[test]
    fn pop_removes_head_in_order() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop().unwrap().value, 1);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop().unwrap().value, 2);
        assert!(list.is_empty());
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
    }

    #[test]
    fn popped_item_is_unlinked() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        assert_eq!(list.push(&a), Ok(()));
        let popped = list.pop().unwrap();
        assert!(core::ptr::eq(popped, &a));
        assert_eq!(a.node.owner.get(), UNLINKED);
        assert!(a.node.prev.get().is_none());
        assert!(a.node.next.get().is_none());
        assert_eq!(list.push(popped), Ok(()));
    }
}
