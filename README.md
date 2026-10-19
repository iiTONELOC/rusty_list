# RustyList

**RustyList** is an intrusive doubly-linked list crate for Rust. It embeds list nodes directly within your data structures (similar to Linux kernel's `list_head`), allowing constant-time linking with no heap allocation. RustyList is **`#![no_std]`** compatible. Every operation is safe: a list borrows each linked item for its own lifetime, so items cannot move or be dropped while linked. Items are shared, so one item can be in several lists at once through one node per list. The one `unsafe impl` is `HasRustyNode`, where you state each node's offset.

## Screenshot

![RustyList](https://raw.githubusercontent.com/iiTONELOC/rusty_list/main/assets/dchains.webp)


## Features

- **No Standard Library Required:** Designed for `#![no_std]` environments, ideal for embedded systems, kernels, and other low-level applications where heap allocation or the standard library may not be available. RustyList allocates no memory on its own.
- **Intrusive Doubly-Linked List:** Instead of storing data in nodes managed by the list, RustyList **intrudes** into your data type by embedding a `RustyListNode` within it. This means no separate node allocations – your struct itself carries the pointers for list links.
- **Compiler-Enforced Lifetime:** `RustyList<'a, T, Tag>` holds each linked item as `&'a T`. The borrow checker guarantees the item outlives its membership and does not move while linked. All pointer arithmetic is handled internally.
- **Multiple Lists per Item:** Embed one `RustyListNode<T, Tag>` per list and pick it with a type tag. A node already linked is refused with `ListError::AlreadyLinked`, and unlinking through the wrong list is refused with `ListError::NotLinked`.
- **Constant-Time Linking:** `push`, `pop`, `insert` without an order function, and `remove` by identity are O(1). Sorted `insert` and `find_equal` are O(n).
- **Doubly-Linked:** Every list node contains pointers to both the next and previous nodes in the list.
- **Optional Sorting:** You can provide an ordering function to maintain a sorted list.
- **Versatile Operations:** Includes `push`, `pop`, `insert`, `remove`, `find_equal`, and `iter`.
- **Static Allocation Friendly:** Use with statically or stack-allocated structures.

## Use Cases

- Embedded systems  
- OS kernels  
- Real-time schedulers  
- Static object pools  
- Custom allocators  
- Performance-sensitive queues or resource managers  

---

## Getting Started

### Install

```bash
cargo add rusty_list
```

Requires Rust 1.85 or newer. No dependencies.

### Usage

1. Define your struct and embed a `RustyListNode<T>`, one per list it can be in.
2. Implement `HasRustyNode` for that struct with `offset_of!`, once per node.
3. Create a `RustyList<T>` using `.new()` or `.new_with_order()`.
4. Use `insert`, `push`, `pop`, `remove`, `find_equal`, `iter` as needed.
5. Put fields you change while linked in a `Cell`.

### Struct Requirements

Example:

```rust
#[repr(C)]
struct MyStruct {
    value: u32,
    node: RustyListNode<MyStruct>,
}

// SAFETY: `node` is the embedded `RustyListNode<Self>` and `offset_of!` reports its offset.
unsafe impl HasRustyNode for MyStruct {
    fn rusty_offset() -> usize {
        core::mem::offset_of!(Self, node)
    }
}
```

> 🔒 `#[repr(C)]` is **required** to ensure predictable field layout for offset math. The `unsafe impl` is your statement that `rusty_offset` is correct; the list trusts it.

### One Item in Several Lists

Give each list its own node and a tag type. Each list reads its own offset, so a `Ready` list cannot be handed the `All` node.

```rust
#[derive(Debug, PartialEq)]
struct Ready;
#[derive(Debug, PartialEq)]
struct All;

#[repr(C)]
struct Task {
    pid: u8,
    state: Cell<u8>,
    ready: RustyListNode<Task, Ready>,
    all: RustyListNode<Task, All>,
}

// SAFETY: `ready` is the embedded `RustyListNode<Self, Ready>` and `offset_of!` reports its offset.
unsafe impl HasRustyNode<Ready> for Task {
    fn rusty_offset() -> usize {
        core::mem::offset_of!(Self, ready)
    }
}

// SAFETY: `all` is the embedded `RustyListNode<Self, All>` and `offset_of!` reports its offset.
unsafe impl HasRustyNode<All> for Task {
    fn rusty_offset() -> usize {
        core::mem::offset_of!(Self, all)
    }
}

let mut ready = RustyList::<Task, Ready>::new();
let mut all = RustyList::<Task, All>::new_with_order(by_pid);

all.push(&task)?;
ready.push(&task)?;
ready.remove(&task)?;
```

The default tag is `()`, so a struct with one node needs no tag at all. Derive `Debug` and `PartialEq` on a tag only when the struct that holds the node derives them.

### Order Function

Used for sorted `insert()` and `find_equal()`:

```rust
fn my_cmp(a: &MyStruct, b: &MyStruct) -> core::cmp::Ordering {
    a.value.cmp(&b.value)
}
```

---

## Examples

### Creating and Using a List

```rust
let mut list = RustyList::<MyStruct>::new_with_order(my_cmp);

let item = MyStruct { value: 42, node: RustyListNode::new() };
list.insert(&item)?;
```

`item` is now borrowed by `list` and cannot move or drop while `list` is in use. A second `insert` of the same item returns `ListError::AlreadyLinked`.

### Insert Sorted Items

```rust
let mut list = RustyList::<MyStruct>::new_with_order(my_cmp);

for item in &my_items {
    list.insert(item)?;
}
```

### Push (Unsorted Append)

```rust
list.push(&my_item)?;
```

### Pop (Remove Head)

```rust
if let Some(item) = list.pop() {
    println!("{}", item.value);
}
```

### Find by Comparator, Remove by Identity

```rust
let target = MyStruct { value: 42, node: RustyListNode::new() };

if let Some(found) = list.find_equal(&target) {
    list.remove(found)?;
}
```

`find_equal` uses the list's order function and returns `None` on a list without one. `remove` takes the item itself and returns `ListError::NotLinked` if that item is not linked into this list.

### Complexity

| Operation | Cost |
| --- | --- |
| `push` | O(1) |
| `pop` | O(1) |
| `insert`, no order function | O(1) |
| `insert`, with order function | O(n) |
| `remove` | O(1) |
| `find_equal` | O(n) |
| `iter` | O(1) per item |
| `len`, `is_empty` | O(1) |

### Iterate

```rust
for item in &list {
    println!("{}", item.value);
}

let total: u32 = list.iter().map(|item| item.value).sum();
```

`iter` yields `&T` from head to tail and borrows the list while it lives.

---

## Tests

All core operations are covered with unit tests:

- `insert` (sorted + head/tail/middle)
- `remove` (head, tail, middle)
- `push` and `pop` (non-sorted FIFO)
- `find_equal` via order function, including misses, empty lists, and lists without an order function
- `remove` by identity, including non-members, empty lists, and an item linked into a different list
- A linked node is refused by `insert` and `push` on any list
- One item linked into two lists at once through two nodes
- Popped and removed items can be linked again
- `iter` head to tail, on empty lists, after removal, and through `for` over `&list`
- Equal keys keep insertion order
- End-to-end integration test with static arrays
- The whole suite under Miri with strict provenance

```bash
cargo +nightly miri test
```

The [repository](https://github.com/iiTONELOC/rusty_list) also carries a libFuzzer target in `fuzz/` that checks random operation sequences against a model. It is not part of the published crate.

```bash
cargo +nightly fuzz run list_ops -- -max_total_time=300
```

To run tests:

```bash
cargo test
```

> Note: `std` is enabled during tests to allow use of `Vec`, `assert_eq!`, etc.

---

## Safety Considerations

- The compiler enforces that an item outlives its membership and does not move while linked.
- Each node links into one list at a time. Every list has a unique id, and a linked node records the id of the list that holds it. A second link is refused with `ListError::AlreadyLinked`. `remove` compares the node's id with its own before touching any pointer, so an unlink through the wrong list is refused with `ListError::NotLinked` and cannot splice two lists together.
- Linked items are shared. Change their fields through `Cell` or another interior-mutability type.
- A wrong `rusty_offset` is undefined behavior, which is why `HasRustyNode` is an `unsafe trait`.
- Not thread-safe and not interrupt-safe. `RustyList` and `RustyListNode` are `!Send` and `!Sync`, so the compiler refuses to share them across threads. The compiler cannot see interrupts, so on a single core an interrupt handler that touches a list mid-update will corrupt it. Disable interrupts or take a lock around every operation. Each operation is a few pointer writes with no allocation and no blocking, so the critical section is short.

---

## Contact

| Name | Contact |
| --- | --- |
| Anthony Tropeano | [**GitHub**](https://github.com/iiTONELOC) <br> [**Email**](mailto:anthonytropeano@me.com) |

---

## License

This project is distributed under the MIT License. See [`LICENSE`](./LICENSE.md) for details.
