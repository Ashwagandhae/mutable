use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct GenId {
    pub index: usize,
    gen: usize,
}
#[derive(Debug, Clone)]
pub struct Collection<Item> {
    items: Vec<Option<Item>>,
    removed: Vec<usize>,
    gens: Vec<usize>,
}

impl<Item> Collection<Item> {
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
        if id_1 == id_2 {
            return (None, None);
        }
        // put them in right order
        let (id_1, id_2) = if id_1.index < id_2.index {
            (id_1, id_2)
        } else {
            (id_2, id_1)
        };
        let gen_1_valid = self.check_gen(id_1);
        let gen_2_valid = self.check_gen(id_2);
        let (left, right) = self.items.split_at_mut(id_2.index);
        (
            left[id_1.index].as_mut().filter(|_| gen_1_valid),
            right[0].as_mut().filter(|_| gen_2_valid),
        )
    }
    pub fn get_index(&self, i: usize) -> Option<&Item> {
        self.items[i].as_ref()
    }
    pub fn get_index_mut(&mut self, i: usize) -> Option<&mut Item> {
        self.items[i].as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Item> + DoubleEndedIterator {
        self.items.iter().filter_map(|item| item.as_ref())
    }
    // allow reveres
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Item> + DoubleEndedIterator {
        self.items.iter_mut().filter_map(|item| item.as_mut())
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

impl<Item> Index<GenId> for Collection<Item> {
    type Output = Item;

    fn index(&self, index: GenId) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<Item> IndexMut<GenId> for Collection<Item> {
    fn index_mut(&mut self, index: GenId) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
