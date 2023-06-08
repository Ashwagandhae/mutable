use std::ops::{Index, IndexMut};

use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct GenId {
    pub index: usize,
    gen: usize,
}
#[derive(Debug, Clone)]
pub struct Collection<Item>
where
    Item: Send,
{
    items: Vec<Option<Item>>,
    removed: Vec<usize>,
    gens: Vec<usize>,
}

#[allow(dead_code)]
impl<Item> Collection<Item>
where
    Item: Send,
{
    pub fn new() -> Collection<Item> {
        Collection {
            items: Vec::new(),
            removed: Vec::new(),
            gens: Vec::new(),
        }
    }
    pub fn push(&mut self, item: Item) -> GenId {
        if let Some(index) = self.removed.pop() {
            self.items[index] = Some(item);
            GenId {
                index,
                gen: self.gens[index],
            }
        } else {
            self.items.push(Some(item));
            self.gens.push(0);
            GenId {
                index: self.items.len() - 1,
                gen: 0,
            }
        }
    }
    fn check_gen(&self, id: GenId) -> bool {
        self.gens[id.index] == id.gen
    }
    pub fn remove(&mut self, id: GenId) -> bool {
        if !self.check_gen(id) {
            return false;
        }
        self.items[id.index] = None;
        self.removed.push(id.index);
        self.gens[id.index] += 1;
        true
    }
    pub fn get(&self, id: GenId) -> Option<&Item> {
        if !self.check_gen(id) {
            return None;
        }
        self.items[id.index].as_ref()
    }
    pub fn get_mut(&mut self, id: GenId) -> Option<&mut Item> {
        if !self.check_gen(id) {
            return None;
        }
        self.items[id.index].as_mut()
    }
    pub fn get_2_mut(
        &mut self,
        id_1: GenId,
        id_2: GenId,
    ) -> (Option<&mut Item>, Option<&mut Item>) {
        // make sure ids are not the same
        if id_1.index == id_2.index {
            return (None, None);
        }
        // put them in right order
        let (left_id, right_id) = if id_1.index < id_2.index {
            (id_1, id_2)
        } else {
            (id_2, id_1)
        };
        let left_gen_valid = self.check_gen(left_id);
        let right_gen_valid = self.check_gen(right_id);
        let (left, right) = self.items.split_at_mut(right_id.index);
        let ret = (
            left[left_id.index].as_mut().filter(|_| left_gen_valid),
            right[0].as_mut().filter(|_| right_gen_valid),
        );
        // put them back in right order
        if id_1.index < id_2.index {
            ret
        } else {
            (ret.1, ret.0)
        }
    }
    pub fn get_index(&self, i: usize) -> Option<&Item> {
        self.items[i].as_ref()
    }
    pub fn get_index_mut(&mut self, i: usize) -> Option<&mut Item> {
        self.items[i].as_mut()
    }
    pub fn get_2_index(&self, i_1: usize, i_2: usize) -> (Option<&Item>, Option<&Item>) {
        // make sure ids are not the same
        if i_1 == i_2 {
            return (None, None);
        }
        // put them in right order
        let (i_1, i_2) = if i_1 < i_2 { (i_1, i_2) } else { (i_2, i_1) };
        let (left, right) = self.items.split_at(i_2);
        (left[i_1].as_ref(), right[0].as_ref())
    }
    pub fn get_2_index_mut(
        &mut self,
        i_1: usize,
        i_2: usize,
    ) -> (Option<&mut Item>, Option<&mut Item>) {
        // make sure ids are not the same
        if i_1 == i_2 {
            return (None, None);
        }
        // put them in right order
        let (i_1, i_2) = if i_1 < i_2 { (i_1, i_2) } else { (i_2, i_1) };
        let (left, right) = self.items.split_at_mut(i_2);
        (left[i_1].as_mut(), right[0].as_mut())
    }
    pub fn iter(&self) -> impl Iterator<Item = &Item> + DoubleEndedIterator {
        self.items.iter().filter_map(|item| item.as_ref())
    }
    // allow reveres
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Item> + DoubleEndedIterator {
        self.items.iter_mut().filter_map(|item| item.as_mut())
    }
    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut Item> {
        self.items.par_iter_mut().filter_map(|item| item.as_mut())
    }
    pub fn iter_with_indices(&self) -> impl Iterator<Item = (usize, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.as_ref().map(|item| (i, item)))
    }
    pub fn iter_with_ids(&self) -> impl Iterator<Item = (GenId, &Item)> {
        self.items.iter().enumerate().filter_map(|(i, item)| {
            item.as_ref().map(|item| {
                (
                    GenId {
                        index: i,
                        gen: self.gens[i],
                    },
                    item,
                )
            })
        })
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.removed.len()
    }
    pub fn full_len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn extend(&mut self, other: &mut Vec<Item>) {
        for item in other.drain(..) {
            self.push(item);
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Item) -> bool,
    {
        for (i, item) in self.items.iter_mut().enumerate() {
            if let Some(inner) = item {
                if !f(inner) {
                    *item = None;
                    self.removed.push(i);
                    self.gens[i] += 1;
                }
            }
        }
    }

    pub fn get_slice(&self) -> &[Option<Item>] {
        &self.items[..]
    }

    pub fn get_mut_slice(&mut self) -> &mut [Option<Item>] {
        &mut self.items[..]
    }
}

impl<Item> Index<GenId> for Collection<Item>
where
    Item: Send,
{
    type Output = Item;

    fn index(&self, index: GenId) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<Item> IndexMut<GenId> for Collection<Item>
where
    Item: Send,
{
    fn index_mut(&mut self, index: GenId) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
