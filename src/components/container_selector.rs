/// Init 容器名称的显示后缀
pub const INIT_CONTAINER_SUFFIX: &str = " (init)";

/// 容器选择器组件（用于在多容器 Pod 中选择查看日志的容器）
#[derive(Clone, Debug)]
pub struct ContainerSelector {
    pub items: Vec<String>,
    pub state: usize,
}

impl ContainerSelector {
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

/// 去除显示用的 "(init)" 后缀，返回实际容器名
pub fn strip_init_suffix(display_name: &str) -> &str {
    display_name
        .strip_suffix(INIT_CONTAINER_SUFFIX)
        .unwrap_or(display_name)
}
