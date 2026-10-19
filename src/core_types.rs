use core::cell::Cell;
use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

/// Comparator used by sorted `insert` and by `find_equal`.
pub type OrderFunction<T> = fn(&T, &T) -> Ordering;

pub(crate) const UNLINKED: usize = 0;
pub(crate) static NEXT_LIST_ID: AtomicUsize = AtomicUsize::new(UNLINKED + 1);

/// A node embedded in a `T` so the item can be linked into a `RustyList<T, Tag>`.
///
/// One node per list the item can be in. `Tag` tells each list which node is its own.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct RustyListNode<T, Tag = ()> {
    /// Marker the caller owns and the list never reads.
    pub dynamic: bool,
    pub(crate) owner: Cell<usize>,
    pub(crate) prev: Cell<Option<NonNull<RustyListNode<T, Tag>>>>,
    pub(crate) next: Cell<Option<NonNull<RustyListNode<T, Tag>>>>,
    pub(crate) _marker: PhantomData<(T, Tag)>,
}

/// An intrusive doubly linked list over items that embed a `RustyListNode<T, Tag>`.
///
/// Every linked item is borrowed for `'a`, so it stays live and in place while
/// the list is in use. Items are shared, so one item can sit in several lists.
#[derive(Debug)]
#[repr(C)]
pub struct RustyList<'a, T, Tag = ()> {
    pub(crate) id: usize,
    pub(crate) len: usize,
    /// Marker the caller owns and the list never reads.
    pub dynamic: bool,
    pub(crate) head: Option<NonNull<RustyListNode<T, Tag>>>,
    pub(crate) tail: Option<NonNull<RustyListNode<T, Tag>>>,
    pub(crate) offset: usize,
    /// Comparator for sorted `insert` and `find_equal`.
    pub order_function: Option<OrderFunction<T>>,
    pub(crate) _items: PhantomData<&'a T>,
}

/// Implemented by any `T` that embeds a `RustyListNode<T, Tag>`.
///
/// # Safety
/// `rusty_offset` must return the byte offset of the embedded `RustyListNode<T, Tag>`
/// inside `T`, as `core::mem::offset_of!` reports it.
pub unsafe trait HasRustyNode<Tag = ()> {
    /// Byte offset of the embedded node, from `core::mem::offset_of!`.
    fn rusty_offset() -> usize;
}

/// Why a link or unlink was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListError {
    /// The item's node for this list is already linked, into this list or another.
    AlreadyLinked,
    /// The item is not linked into this list.
    NotLinked,
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AlreadyLinked => "item is already linked",
            Self::NotLinked => "item is not linked into this list",
        })
    }
}

impl core::error::Error for ListError {}

/// Pointer to the `T` that holds `node`.
///
/// # Safety
/// `node` must point to the `RustyListNode<T, Tag>` embedded in a `T` at byte `offset`.
#[inline(always)]
pub(crate) unsafe fn rusty_container_of<T, Tag>(
    node: NonNull<RustyListNode<T, Tag>>,
    offset: usize,
) -> *const T {
    // SAFETY: the caller guarantees `node` lies `offset` bytes into a `T`.
    unsafe { node.cast::<u8>().sub(offset).cast::<T>().as_ptr() }
}

/// Shared reference to a node.
///
/// # Safety
/// `node` must lie inside an item that is borrowed for at least `'n`.
#[inline(always)]
pub(crate) unsafe fn node_ref<'n, T, Tag>(
    node: NonNull<RustyListNode<T, Tag>>,
) -> &'n RustyListNode<T, Tag> {
    // SAFETY: the caller guarantees the item, and so the node, is live for `'n`.
    unsafe { node.as_ref() }
}
