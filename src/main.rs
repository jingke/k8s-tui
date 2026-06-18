use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

#[cfg(test)]
mod app_tests;

mod app;
mod components;
mod config;
mod event;
mod k8s;
mod ui;

use app::App;
use event::{Event, EventHandler};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 设置终端
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建事件通道
    let (tx, mut rx) = mpsc::channel::<Event>(100);

    // 初始化 K8s 客户端
    let k8s_client = match k8s::K8sClient::new().await {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::warn!("无法连接 K8s: {}", e);
            None
        }
    };

    // 初始化应用状态
    let mut app = App::new(k8s_client);

    // 启动事件处理器（键盘 + 定时刷新）
    let event_handler = EventHandler::new(tx.clone());
    let _event_task = tokio::spawn(async move {
        event_handler.run().await;
    });

    // 初始加载数据
    app.refresh_resources().await;

    // 主循环
    let res = run_app(&mut terminal, &mut app, &mut rx).await;

    // 恢复终端
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("错误: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: &mut mpsc::Receiver<Event>,
) -> Result<()> {
    loop {
        // 渲染 UI
        terminal.draw(|f| ui::draw(f, app))?;

        // 阻塞等待下一个事件，避免空转占用 CPU
        let Some(event) = rx.recv().await else {
            return Ok(());
        };
        match event {
            Event::Tick => {
                app.on_tick().await;
            }
            Event::Key(key) => {
                if app.handle_key_event(key).await? {
                    return Ok(());
                }
            }
            Event::Resize(_, _) => {}
        }
    }
}
