use crate::{ListError, RustyList, RustyListNode, node_ref, rusty_container_of};
use core::cmp::Ordering;
use core::ptr::NonNull;

impl<'a, T, Tag> RustyList<'a, T, Tag> {
    /// Links `item` in sorted position when the list has a comparator, else at the tail.
    ///
    /// `item` stays borrowed by the list for `'a`. A node that is already linked is refused.
    pub fn insert(&mut self, item: &'a T) -> Result<(), ListError> {
        let node = self.claim(item)?;
        match self.insert_position(item) {
            None => self.insert_node_at_tail(node),
            Some(current) => self.insert_node_before(node, current),
        }
        self.len += 1;
        Ok(())
    }

    /// The first linked node that `item` sorts before, or `None` to append.
    fn insert_position(&self, item: &T) -> Option<NonNull<RustyListNode<T, Tag>>> {
        let order = self.order_function?;
        let tail = self.tail?;
        // SAFETY: linked nodes lie inside items borrowed for `'a`.
        let last = unsafe { &*rusty_container_of(tail, self.offset) };
        if order(item, last) != Ordering::Less {
            return None;
        }
        let mut current = self.head;
        while let Some(node) = current {
            // SAFETY: linked nodes lie inside items borrowed for `'a`.
            let existing = unsafe { &*rusty_container_of(node, self.offset) };
            if order(item, existing) == Ordering::Less {
                return Some(node);
            }
            // SAFETY: `node` is linked, so it is live.
            current = unsafe { node_ref(node) }.next.get();
        }
        None
    }

    pub(crate) fn insert_node_at_tail(&mut self, node: NonNull<RustyListNode<T, Tag>>) {
        // SAFETY: `node` and the tail lie inside items borrowed for `'a`.
        unsafe {
            node_ref(node).prev.set(self.tail);
            match self.tail {
                Some(tail) => node_ref(tail).next.set(Some(node)),
                None => self.head = Some(node),
            }
        }
        self.tail = Some(node);
    }

    fn insert_node_before(
        &mut self,
        node: NonNull<RustyListNode<T, Tag>>,
        current: NonNull<RustyListNode<T, Tag>>,
    ) {
        // SAFETY: `node`, `current`, and its predecessor lie inside items borrowed for `'a`.
        unsafe {
            let prev = node_ref(current).prev.get();
            node_ref(node).prev.set(prev);
            node_ref(node).next.set(Some(current));
            node_ref(current).prev.set(Some(node));
            match prev {
                Some(prev) => node_ref(prev).next.set(Some(node)),
                None => self.head = Some(node),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RustyList;
    use crate::fixtures::{TestItem, cmp, make_item, values};
    use std::vec;

    #[test]
    fn insert_sorted_into_list() {
        let mut list = RustyList::<TestItem>::new_with_order(cmp);
        let one = make_item(1);
        let three = make_item(3);
        let two = make_item(2);
        assert_eq!(list.insert(&three), Ok(()));
        assert_eq!(list.insert(&one), Ok(()));
        assert_eq!(list.insert(&two), Ok(()));
        assert_eq!(list.len(), 3);
        assert_eq!(values(&list), vec![1, 2, 3]);
    }
}
