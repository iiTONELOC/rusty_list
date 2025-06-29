# RustyList

**RustyList** is a safe, intrusive doubly-linked list crate for Rust. It provides a way to embed list nodes directly within your data structures (similar to Linux kernel's `list_head`), allowing efficient in-place list management with no heap allocation. RustyList is **`#![no_std]`** compatible and ensures that all unsafe operations are confined internally, presenting a clean **100% safe** public API.

## Screenshot

![Threads - University of Arizona](./assets/dchains.webp)


## Features

- **No Standard Library Required:** Designed for `#![no_std]` environments, ideal for embedded systems, kernels, and other low-level applications where heap allocation or the standard library may not be available. RustyList allocates no memory on its own.
- **Intrusive Doubly-Linked List:** Instead of storing data in nodes managed by the list, RustyList **intrudes** into your data type by embedding a `RustyListNode` within it. This means no separate node allocations – your struct itself carries the pointers for list links.
- **Safe API (Internal Unsafe):** You do not need to write any `unsafe` code to use RustyList. All pointer arithmetic and aliasing trickery are handled internally.
- **Doubly-Linked:** Every list node contains pointers to both the next and previous nodes in the list.
- **Optional Sorting:** You can provide an ordering function to maintain a sorted list.
- **Versatile Operations:** Includes `push`, `pop`, `insert`, `remove`, and `find_equal`.
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

### Usage

1. Define your struct and embed a `RustyListNode<T>`.
2. Implement `HasRustyNode` for that struct.
3. Create a `RustyList<T>` and set the offset using `.new()` or `.new_with_order()`.
4. Use `insert`, `push`, `pop`, `remove`, `find_equal` as needed.

### Struct Requirements

Example:

```rust
#[repr(C)]
struct MyStruct {
    value: u32,
    node: RustyListNode<MyStruct>,
}

impl HasRustyNode for MyStruct {
    fn rusty_offset() -> usize {
        rusty_offset(|s: &Self| &s.node)
    }
}
```

> 🔒 `#[repr(C)]` is **required** to ensure predictable field layout for offset math.

### Order Function

Used for sorted `insert()` and `find_equal()`:

```rust
fn my_cmp(a: *const MyStruct, b: *const MyStruct) -> i32 {
    unsafe {
        (*a).value.cmp(&(*b).value) as i32
    }
}
```

---

## Examples

### Creating and Using a List

```rust
let mut list = RustyList::<MyItem>::new_with_order(|a, b| a.value.cmp(&b.value));

let mut item = MyItem { value: 42, node: RustyListNode::new() };
list.insert(&mut item);
```

### Insert Sorted Items

```rust
let mut list = RustyList::<MyStruct>::new_with_order(my_cmp);

for item in my_items.iter_mut() {
    list.insert(item) ;
}
```

### Push (Unsorted Append)

```rust
 list.push(&mut my_item);
```

### Pop (Remove Head)

```rust
let item =  list.pop();
```

### Remove Specific Node

```rust
if let Some(ptr) = list.find_equal(&target) {
    list.remove(ptr);
}
list.remove(ptr);
```

---

## Tests

All core operations are covered with unit tests:

- `insert` (sorted + head/tail/middle)
- `remove` (head, tail, middle)
- `push` and `pop` (non-sorted FIFO)
- `find_equal` via order function
- End-to-end integration test with static arrays

To run tests:

```bash
cargo test
```

> Note: `std` is enabled during tests to allow use of `Vec`, `assert_eq!`, etc.

---

## Safety Considerations

- Ensure inserted items remain valid while in the list.
- Only one `RustyListNode` per list per item.
- Not thread-safe by default.

---

## Contact

| Name | Contact |
| --- | --- |
| Anthony Tropeano | [**GitHub**](https://github.com/iiTONELOC) <br> [**Email**](mailto:atropeano@atropeano.com) |

---

## License

This project is distributed under the MIT License. See [`LICENSE`](./LICENSE.md) for details.
