/// 确认对话框组件
#[derive(Clone, Debug)]
pub struct ConfirmDialog {
    pub message: String,
}

impl ConfirmDialog {
    pub fn new() -> Self {
        Self {
            message: String::new(),
        }
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = msg;
    }
}
