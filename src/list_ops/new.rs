use crate::{
    HasRustyNode, ListError, NEXT_LIST_ID, OrderFunction, RustyList, RustyListNode, UNLINKED,
    node_ref,
};
use core::cell::Cell;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;

const LIST_ID_EXHAUSTED: &str = "list id space exhausted";

fn next_list_id() -> usize {
    let mut current = NEXT_LIST_ID.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1).expect(LIST_ID_EXHAUSTED);
        match NEXT_LIST_ID.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return current,
            Err(seen) => current = seen,
        }
    }
}

impl<T, Tag> RustyListNode<T, Tag> {
    /// A node with no links and `dynamic` false.
    pub const fn new() -> Self {
        Self {
            dynamic: false,
            owner: Cell::new(UNLINKED),
            prev: Cell::new(None),
            next: Cell::new(None),
            _marker: PhantomData,
        }
    }

    /// Sets `dynamic` and returns the node.
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = dynamic;
        self
    }

    pub(crate) fn clear_links(&self) {
        self.owner.set(UNLINKED);
        self.prev.set(None);
        self.next.set(None);
    }
}

impl<T, Tag> Default for RustyListNode<T, Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HasRustyNode<Tag>, Tag> RustyList<'_, T, Tag> {
    /// An empty list whose `insert` appends.
    pub fn new() -> Self {
        Self {
            id: next_list_id(),
            len: 0,
            dynamic: false,
            head: None,
            tail: None,
            offset: T::rusty_offset(),
            order_function: None,
            _items: PhantomData,
        }
    }

    /// An empty list whose `insert` keeps it sorted by `order`.
    pub fn new_with_order(order: OrderFunction<T>) -> Self {
        Self {
            order_function: Some(order),
            ..Self::new()
        }
    }

    /// Sets `dynamic` and returns the list.
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = dynamic;
        self
    }
}

impl<T: HasRustyNode<Tag>, Tag> Default for RustyList<'_, T, Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Tag> RustyList<'_, T, Tag> {
    /// Number of linked items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when no item is linked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn node_of(&self, item: &T) -> NonNull<RustyListNode<T, Tag>> {
        // SAFETY: `HasRustyNode<Tag>` places this list's node `offset` bytes into every `T`.
        unsafe {
            NonNull::from(item)
                .cast::<u8>()
                .add(self.offset)
                .cast::<RustyListNode<T, Tag>>()
        }
    }

    pub(crate) fn claim(&self, item: &T) -> Result<NonNull<RustyListNode<T, Tag>>, ListError> {
        let node = self.node_of(item);
        // SAFETY: `node` lies inside `item`, which is live for this call.
        let fields = unsafe { node_ref(node) };
        if fields.owner.replace(self.id) != UNLINKED {
            return Err(ListError::AlreadyLinked);
        }
        fields.prev.set(None);
        fields.next.set(None);
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::{TestItem, cmp, make_item};
    use crate::{HasRustyNode, RustyList, RustyListNode, UNLINKED};
    use core::cmp::Ordering;
    use core::ptr::NonNull;

    #[test]
    fn new_initializes_fields() {
        let list = RustyList::<TestItem>::new();
        assert_ne!(list.id, UNLINKED);
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert!(!list.dynamic);
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
        assert!(list.order_function.is_none());
        assert_eq!(list.offset, <TestItem as HasRustyNode>::rusty_offset());
    }

    #[test]
    fn new_with_order_stores_comparator() {
        let list = RustyList::<TestItem>::new_with_order(cmp);
        let a = make_item(1);
        let b = make_item(2);
        let order = list.order_function.unwrap();
        assert_eq!(order(&a, &b), Ordering::Less);
    }

    #[test]
    fn with_dynamic_sets_the_marker_on_list_and_node() {
        let list = RustyList::<TestItem>::new().with_dynamic(true);
        assert!(list.dynamic);
        assert!(!list.with_dynamic(false).dynamic);
        let node = RustyListNode::<u32>::new().with_dynamic(true);
        assert!(node.dynamic);
        assert!(!node.with_dynamic(false).dynamic);
    }

    #[test]
    fn each_list_gets_its_own_id() {
        let first = RustyList::<TestItem>::new();
        let second = RustyList::<TestItem>::new();
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn node_new_has_no_links() {
        let node = RustyListNode::<u32>::new();
        assert!(!node.dynamic);
        assert_eq!(node.owner.get(), UNLINKED);
        assert!(node.prev.get().is_none());
        assert!(node.next.get().is_none());
    }

    #[test]
    fn clear_links_drops_both_links() {
        let node = RustyListNode::<u32>::new();
        node.owner.set(7);
        node.prev.set(Some(NonNull::dangling()));
        node.next.set(Some(NonNull::dangling()));
        node.clear_links();
        assert_eq!(node.owner.get(), UNLINKED);
        assert!(node.prev.get().is_none());
        assert!(node.next.get().is_none());
    }
}
