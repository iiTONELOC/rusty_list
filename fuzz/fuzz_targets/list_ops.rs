#![no_main]

use core::cmp::Ordering;
use libfuzzer_sys::fuzz_target;
use rusty_list::{HasRustyNode, RustyList, RustyListNode};

const POOL: usize = 8;
const VALUES: [u8; POOL] = [0, 1, 2, 0, 1, 2, 0, 1];
const OPS: u8 = 6;

#[derive(Debug, PartialEq)]
struct Alt;

#[repr(C)]
struct Item {
    value: u8,
    index: usize,
    main: RustyListNode<Item>,
    alt: RustyListNode<Item, Alt>,
}

// SAFETY: `main` is the embedded `RustyListNode<Self>` and `offset_of!` reports its offset.
unsafe impl HasRustyNode for Item {
    fn rusty_offset() -> usize {
        core::mem::offset_of!(Self, main)
    }
}

// SAFETY: `alt` is the embedded `RustyListNode<Self, Alt>` and `offset_of!` reports its offset.
unsafe impl HasRustyNode<Alt> for Item {
    fn rusty_offset() -> usize {
        core::mem::offset_of!(Self, alt)
    }
}

fn cmp(a: &Item, b: &Item) -> Ordering {
    a.value.cmp(&b.value)
}

fn probe(value: u8) -> Item {
    Item {
        value,
        index: usize::MAX,
        main: RustyListNode::new(),
        alt: RustyListNode::new(),
    }
}

struct Model {
    order: Vec<usize>,
    sorted: bool,
}

impl Model {
    fn contains(&self, index: usize) -> bool {
        self.order.contains(&index)
    }

    fn insert(&mut self, index: usize) {
        let value = VALUES[index];
        let before_tail = self.sorted && self.order.last().is_some_and(|&i| value < VALUES[i]);
        let at = if before_tail {
            self.order
                .iter()
                .position(|&i| value < VALUES[i])
                .unwrap_or(self.order.len())
        } else {
            self.order.len()
        };
        self.order.insert(at, index);
    }

    fn remove(&mut self, index: usize) -> bool {
        match self.order.iter().position(|&i| i == index) {
            Some(at) => {
                self.order.remove(at);
                true
            }
            None => false,
        }
    }

    fn pop(&mut self) -> Option<usize> {
        if self.order.is_empty() {
            None
        } else {
            Some(self.order.remove(0))
        }
    }

    fn find(&self, value: u8) -> Option<usize> {
        if !self.sorted {
            return None;
        }
        self.order.iter().copied().find(|&i| VALUES[i] == value)
    }
}

fn step<'a, Tag>(
    list: &mut RustyList<'a, Item, Tag>,
    model: &mut Model,
    item: &'a Item,
    kind: u8,
) {
    let index = item.index;
    let value = item.value;
    match kind {
        0 => {
            let linked = list.insert(item).is_ok();
            assert_eq!(linked, !model.contains(index));
            if linked {
                model.insert(index);
            }
        }
        1 => {
            let linked = list.push(item).is_ok();
            assert_eq!(linked, !model.contains(index));
            if linked {
                model.order.push(index);
            }
        }
        2 => assert_eq!(list.pop().map(|i| i.index), model.pop()),
        3 => assert_eq!(list.remove(item).is_ok(), model.remove(index)),
        4 => {
            let found = list.find_equal(&probe(value)).map(|i| i.index);
            assert_eq!(found, model.find(value));
        }
        _ => {
            let seen: Vec<usize> = list.iter().map(|i| i.index).collect();
            assert_eq!(seen, model.order);
        }
    }
    assert_eq!(list.len(), model.order.len());
    assert_eq!(list.is_empty(), model.order.is_empty());
}

fn make_list<'a, Tag>(sorted: bool) -> RustyList<'a, Item, Tag>
where
    Item: HasRustyNode<Tag>,
{
    if sorted {
        RustyList::new_with_order(cmp)
    } else {
        RustyList::new()
    }
}

fn drain<Tag>(list: &mut RustyList<'_, Item, Tag>, model: &Model) {
    let mut drained = Vec::new();
    while let Some(item) = list.pop() {
        drained.push(item.index);
    }
    assert_eq!(drained, model.order);
}

fuzz_target!(|data: &[u8]| {
    let Some((&mode, ops)) = data.split_first() else {
        return;
    };
    let pool: [Item; POOL] = core::array::from_fn(|index| Item {
        value: VALUES[index],
        index,
        main: RustyListNode::new(),
        alt: RustyListNode::new(),
    });
    let mut main = make_list::<()>(mode & 1 == 1);
    let mut alt = make_list::<Alt>(mode & 2 == 2);
    let mut main_model = Model {
        order: Vec::new(),
        sorted: mode & 1 == 1,
    };
    let mut alt_model = Model {
        order: Vec::new(),
        sorted: mode & 2 == 2,
    };
    for &op in ops {
        let item = &pool[usize::from(op) % POOL];
        let kind = (op / POOL as u8) % OPS;
        if (op / POOL as u8) / OPS % 2 == 0 {
            step(&mut main, &mut main_model, item, kind);
        } else {
            step(&mut alt, &mut alt_model, item, kind);
        }
    }
    drain(&mut main, &main_model);
    drain(&mut alt, &alt_model);
});
