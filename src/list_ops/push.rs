use crate::{ListError, RustyList};

impl<'a, T, Tag> RustyList<'a, T, Tag> {
    /// Links `item` at the tail. The comparator is not used.
    ///
    /// `item` stays borrowed by the list for `'a`. A node that is already linked is refused.
    pub fn push(&mut self, item: &'a T) -> Result<(), ListError> {
        let node = self.claim(item)?;
        self.insert_node_at_tail(node);
        self.len += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::{TestItem, make_item, values};
    use crate::{ListError, RustyList};
    use std::vec;

    #[test]
    fn push_appends_to_tail() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        assert_eq!(list.len(), 2);
        assert_eq!(values(&list), vec![1, 2]);
    }

    #[test]
    fn push_twice_is_refused() {
        let mut list = RustyList::<TestItem>::new();
        let a = make_item(1);
        let b = make_item(2);
        assert_eq!(list.push(&a), Ok(()));
        assert_eq!(list.push(&b), Ok(()));
        assert_eq!(list.push(&a), Err(ListError::AlreadyLinked));
        assert_eq!(list.len(), 2);
        assert_eq!(values(&list), vec![1, 2]);
        assert!(b.node.next.get().is_none());
    }
}
