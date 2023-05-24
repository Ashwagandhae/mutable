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
    pub fn push(&mut self, item: Item) {
        if let Some(index) = self.removed.pop() {
            self.items[index] = Some(item);
        } else {
            self.items.push(Some(item));
        }
    }
    pub fn remove(&mut self, index: usize) {
        self.items[index] = None;
        self.removed.push(index);
    }
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items[index].as_ref()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items[index].as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter_map(|item| item.as_ref())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Item> {
        self.items.iter_mut().filter_map(|item| item.as_mut())
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
}
