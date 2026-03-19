use super::filter::filter_items;

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub label: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerOutcome {
    Selected(String),
    Cancelled,
}

pub struct PickerState {
    pub title: String,
    pub items: Vec<PickerItem>,
    labels: Vec<String>,
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub cursor: usize,
}

impl PickerState {
    pub fn new(title: String, items: Vec<PickerItem>) -> Self {
        let labels: Vec<String> = items.iter().map(|item| item.label.clone()).collect();
        let filtered_indices = filter_items(&labels, "");
        Self {
            title,
            items,
            labels,
            query: String::new(),
            filtered_indices,
            cursor: 0,
        }
    }

    pub fn refilter(&mut self) {
        self.filtered_indices = filter_items(&self.labels, &self.query);
        if self.cursor >= self.filtered_indices.len() {
            self.cursor = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn push_char(&mut self, ch: char) {
        self.query.push(ch);
        self.refilter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.refilter();
    }

    pub fn move_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        if self.cursor == 0 {
            self.cursor = self.filtered_indices.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        self.cursor = (self.cursor + 1) % self.filtered_indices.len();
    }

    pub fn selected_key(&self) -> Option<&str> {
        let &item_index = self.filtered_indices.get(self.cursor)?;
        Some(&self.items[item_index].key)
    }

    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }
}
