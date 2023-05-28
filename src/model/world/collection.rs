#[derive(Debug, Clone)]
pub struct Collection<Item> {
    items: Vec<Option<Item>>,
    removed: Vec<usize>,
}

impl<Item> Collection<Item> {
    pub fn new() -> Collection<Item> {
        Collection {
            items: Vec::new(),
            removed: Vec::new(),
        }
    }
    pub fn push(&mut self, item: Item) -> usize {
        if let Some(id) = self.removed.pop() {
            self.items[id] = Some(item);
            id
        } else {
            self.items.push(Some(item));
            self.items.len() - 1
        }
    }
    pub fn remove(&mut self, id: usize) {
        self.items[id] = None;
        self.removed.push(id);
    }
    pub fn get(&self, id: usize) -> Option<&Item> {
        self.items[id].as_ref()
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Item> {
        self.items[id].as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Item> + DoubleEndedIterator {
        self.items.iter().filter_map(|item| item.as_ref())
    }
    // allow reveres
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Item> + DoubleEndedIterator {
        self.items.iter_mut().filter_map(|item| item.as_mut())
    }
    pub fn iter_with_indices(&self) -> impl Iterator<Item = (usize, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.as_ref().map(|item| (i, item)))
    }
    pub fn len(&self) -> usize {
        self.items.len() - self.removed.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        self.items.clear();
        self.removed.clear();
    }
    pub fn extend(&mut self, other: &mut Vec<Item>) {
        for item in other.drain(..) {
            self.push(item);
        }
    }
    pub fn get_vec(&self) -> &Vec<Option<Item>> {
        &self.items
    }
    pub fn remove_when<F>(&mut self, mut f: F)
    where
        F: FnMut(&Item) -> bool,
    {
        for (i, item) in self.items.iter_mut().enumerate() {
            if let Some(inner) = item {
                if f(inner) {
                    *item = None;
                    self.removed.push(i);
                }
            }
        }
    }
}
