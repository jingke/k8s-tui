/// 命名空间选择器组件
#[derive(Clone, Debug)]
pub struct NamespaceSelector {
    pub items: Vec<String>,
    pub state: usize,
}

impl NamespaceSelector {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: 0,
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.state = 0;
    }

    pub fn select_item(&mut self, item: &str) {
        if let Some(idx) = self.items.iter().position(|i| i == item) {
            self.state = idx;
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.state = (self.state + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            self.state = if self.state == 0 {
                self.items.len() - 1
            } else {
                self.state - 1
            };
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.items.get(self.state).cloned()
    }
}
