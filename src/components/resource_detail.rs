/// 资源详情弹窗组件
#[derive(Clone, Debug)]
pub struct ResourceDetail {
    pub content: String,
}

impl ResourceDetail {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }
}
