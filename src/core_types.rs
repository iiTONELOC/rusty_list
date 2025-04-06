// core_types.rs
// This file contains the core types and traits used in the RustyList library.
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// A node that gets embedded inside a struct to make it linkable in a RustyList.
///
/// This is like `struct list_head` in Linux — it doesn’t own data, it just connects items.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct RustyListNode<T> {
    pub dynamic: bool,
    pub _marker: PhantomData<T>,
    pub prev: Option<NonNull<RustyListNode<T>>>,
    pub next: Option<NonNull<RustyListNode<T>>>,
}

/// A doubly linked intrusive list.
/// `T` is the type that contains a `RustyListNode<T>` inside it.
#[derive(Debug)]
#[repr(C)]
pub struct RustyList<T> {
    pub len: usize,
    pub dynamic: bool,    
    pub head: Option<NonNull<RustyListNode<T>>>,
    pub tail: Option<NonNull<RustyListNode<T>>>,
    
    /// Offset (in bytes) from &T to the embedded `RustyListNode<T>`.
    pub offset: usize,

    /// Optional sort/comparison function.
    /// Like in C: returns `< 0`, `0`, or `> 0` for ordering two items.
    pub order_function: Option<fn(*const T, *const T) -> i32>,
}

/// Trait that must be implemented by any struct that embeds a `RustyListNode<T>`.
/// Provides the offset to the node so that the list can navigate from a node to the parent struct.
pub trait HasRustyNode {
    fn rusty_offset() -> usize;
}



/// Generic offsetof-style function.
/// Gets the offset of a field inside a struct at compile time.
/// Similar to `offsetof(T, field)` in C.
#[inline(always)]
pub fn rusty_offset<T, F>(field: fn(&T) -> &F) -> usize {
    let uninit = MaybeUninit::<T>::uninit();
    let base = uninit.as_ptr() as *const u8;

    // SAFETY: We are not reading the memory, just getting the address of a field.
    let field_ptr = field(unsafe { &*uninit.as_ptr() }) as *const F as *const u8;

    field_ptr as usize - base as usize
}

/// SAFELY go from a pointer to the embedded node to a pointer to the container `T`.
/// This is like `container_of()` in C.
#[inline(always)]
pub unsafe fn rusty_container_of<T>(node: *const RustyListNode<T>, offset: usize) -> *const T {
    unsafe { (node as *const u8).sub(offset) as *const T }
}

/// Mutable version of `rusty_container_of`.
#[inline(always)]
pub unsafe fn rusty_container_of_mut<T>(node: *mut RustyListNode<T>, offset: usize) -> *mut T {
    unsafe { (node as *mut u8).sub(offset) as *mut T }
}
