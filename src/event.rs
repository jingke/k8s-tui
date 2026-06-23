use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

/// 应用事件
#[derive(Clone, Debug)]
pub enum Event {
    /// 定时 tick
    Tick,
    /// 键盘事件
    Key(KeyEvent),
    /// 鼠标事件（滚轮、点击等）
    Mouse(MouseEvent),
    /// 终端尺寸变化
    Resize((), ()),
}

/// 事件处理器：监听键盘输入和定时刷新
pub struct EventHandler {
    tx: Sender<Event>,
}

impl EventHandler {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_millis(250));

        loop {
            interval.tick().await;

            // 发送定时 tick
            let _ = self.tx.send(Event::Tick).await;

            // 非阻塞检查键盘事件
            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    match evt {
                        CrosstermEvent::Key(key) => {
                            let _ = self.tx.send(Event::Key(key)).await;
                        }
                        CrosstermEvent::Mouse(m) => {
                            let _ = self.tx.send(Event::Mouse(m)).await;
                        }
                        CrosstermEvent::Resize(_w, _h) => {
                            let _ = self.tx.send(Event::Resize((), ())).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
