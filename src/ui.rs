use crate::app::{App, Popup, ResourceTab};
use crate::components::container_selector::strip_init_suffix;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

/// 主绘制函数
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 状态栏
            Constraint::Length(1), // 标签栏
            Constraint::Min(0),    // 主内容区
            Constraint::Length(1), // 底部提示
        ])
        .split(area);

    draw_status_bar(f, app, chunks[0]);
    draw_tab_bar(f, app, chunks[1]);
    draw_resource_list(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);

    match app.popup {
        Popup::Detail => draw_detail_popup(f, app),
        Popup::ConfirmDelete => draw_confirm_dialog(f, app),
        Popup::ContextSelector => draw_context_selector(f, app),
        Popup::NamespaceSelector => draw_namespace_selector(f, app),
        Popup::Help => draw_help_popup(f, app),
        Popup::LogViewer => draw_log_viewer(f, app),
        Popup::ContainerSelector => draw_container_selector(f, app),
        Popup::None => {}
    }

    if app.search_mode {
        draw_search_bar(f, app);
    }

    // 消息 Toast（错误优先）
    if let Some(ref err) = app.error_message {
        draw_error_toast(f, err);
    } else if let Some(ref succ) = app.success_message {
        draw_success_toast(f, succ);
    }
}

/// 顶部状态栏
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let context = if app.current_context.is_empty() {
        "未连接"
    } else {
        &app.current_context
    };
    let ns = &app.current_namespace;

    let text = Text::from(Line::from(vec![
        Span::styled(" k8s-tui ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(format!("Context: {} ", context), Style::default().fg(Color::Yellow)),
        Span::styled(format!("NS: {}", ns), Style::default().fg(Color::Green)),
    ]));

    f.render_widget(Paragraph::new(text), area);
}

/// 标签栏
fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tabs = [
        ResourceTab::Pod,
        ResourceTab::ConfigMap,
        ResourceTab::Secret,
        ResourceTab::Deployment,
        ResourceTab::Service,
        ResourceTab::Node,
    ];
    let mut spans = vec![];

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = *tab == app.current_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {}:{} ", i + 1, tab.as_str()), style));
        spans.push(Span::raw(" "));
    }

    let count = app.filtered_resources.len();
    spans.push(Span::styled(
        format!("|  {} resources", count),
        Style::default().fg(Color::DarkGray),
    ));

    f.render_widget(Paragraph::new(Text::from(Line::from(spans))), area);
}

/// 资源列表
fn draw_resource_list(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    let (header_cells, column_widths): (&[&str], Vec<Constraint>) = match app.current_tab {
        ResourceTab::Pod => (
            &["NAME", "STATUS", "RESTARTS", "AGE", "NODE"],
            vec![
                Constraint::Percentage(35),
                Constraint::Percentage(15),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(26),
            ],
        ),
        ResourceTab::ConfigMap => (
            &["NAME", "NAMESPACE", "DATA", "AGE"],
            vec![
                Constraint::Percentage(40),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ],
        ),
        ResourceTab::Secret => (
            &["NAME", "NAMESPACE", "TYPE", "DATA", "AGE"],
            vec![
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(12),
                Constraint::Percentage(18),
            ],
        ),
        ResourceTab::Deployment => (
            &["NAME", "NAMESPACE", "READY", "AGE", "STRATEGY"],
            vec![
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(12),
                Constraint::Percentage(23),
            ],
        ),
        ResourceTab::Service => (
            &["NAME", "NAMESPACE", "TYPE", "CLUSTER-IP", "AGE"],
            vec![
                Constraint::Percentage(25),
                Constraint::Percentage(18),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(22),
            ],
        ),
        ResourceTab::Node => (
            &["NAME", "STATUS", "VERSION", "AGE"],
            vec![
                Constraint::Percentage(35),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        ),
    };

    let header = Row::new(
        header_cells.iter().map(|h| Span::styled(*h, header_style))
    )
    .height(1);

    let rows: Vec<Row> = app
        .filtered_resources
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let res = &app.resources[idx];
            let is_selected = app.table_state.selected() == Some(i);
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_color = match res.status.as_str() {
                "Running" => Color::Green,
                "Pending" => Color::Yellow,
                "Error" | "Failed" | "CrashLoopBackOff" => Color::Red,
                "Completed" | "Succeeded" => Color::Blue,
                _ => Color::Gray,
            };

            let cells: Vec<Span> = match app.current_tab {
                ResourceTab::Pod => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.status.clone(), style.fg(status_color)),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                    Span::styled(res.age.clone(), style),
                    Span::styled(res.extra.get(1).cloned().unwrap_or_default(), style),
                ],
                ResourceTab::ConfigMap => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.namespace.clone(), style),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                    Span::styled(res.age.clone(), style),
                ],
                ResourceTab::Secret => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.namespace.clone(), style),
                    Span::styled(res.status.clone(), style),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                    Span::styled(res.age.clone(), style),
                ],
                ResourceTab::Deployment => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.namespace.clone(), style),
                    Span::styled(res.status.clone(), style),
                    Span::styled(res.age.clone(), style),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                ],
                ResourceTab::Service => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.namespace.clone(), style),
                    Span::styled(res.status.clone(), style),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                    Span::styled(res.age.clone(), style),
                ],
                ResourceTab::Node => vec![
                    Span::styled(res.name.clone(), style),
                    Span::styled(res.status.clone(), style),
                    Span::styled(res.extra.first().cloned().unwrap_or_default(), style),
                    Span::styled(res.age.clone(), style),
                ],
            };

            Row::new(cells).height(1).style(style)
        })
        .collect();

    let table = Table::new(rows, column_widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

/// 底部提示栏
fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let text = Text::from(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(":Navigate "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":Details "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(":Delete "),
        Span::styled("l", Style::default().fg(Color::Yellow)),
        Span::raw(":Logs "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(":Search "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":Context "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":NS "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":Refresh "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(":Help"),
    ]));

    f.render_widget(Paragraph::new(text), area);
}

/// 详情弹窗
fn draw_detail_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 资源详情 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let lines: Vec<Line> = app
        .detail_popup
        .lines()
        .iter()
        .skip(app.detail_scroll as usize)
        .map(|line| Line::from(Span::raw(*line)))
        .collect();

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    let hint_area = Rect {
        x: area.x + 2,
        y: area.y + area.height - 1,
        width: area.width - 4,
        height: 1,
    };
    let hint = Paragraph::new(Text::from(Line::from(vec![
        Span::styled(
            "[↑/↓] Scroll  [PgUp/PgDn] Page  [l] Logs  [d] Delete  [Esc] Close",
            Style::default().fg(Color::Gray),
        ),
    ])));
    f.render_widget(hint, hint_area);
}

/// 确认对话框
fn draw_confirm_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 确认删除 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            app.confirm_dialog.message.clone(),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "按 [y] 确认删除 / [n] 或 [Esc] 取消",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    f.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center), area);
}

/// Context 选择器
fn draw_context_selector(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 切换 Context ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<Line> = app
        .context_selector
        .items
        .iter()
        .enumerate()
        .map(|(i, ctx)| {
            let is_selected = i == app.context_selector.state;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if ctx == &app.current_context {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("  {}  ", ctx), style))
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(items)).block(block), area);
}

/// 命名空间选择器
fn draw_namespace_selector(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 切换命名空间 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<Line> = app
        .namespace_selector
        .items
        .iter()
        .enumerate()
        .map(|(i, ns)| {
            let is_selected = i == app.namespace_selector.state;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if ns == &app.current_namespace {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("  {}  ", ns), style))
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(items)).block(block), area);
}

/// 帮助弹窗
fn draw_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" 快捷键帮助 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let content = crate::components::HelpPopup::content();
    let lines: Vec<Line> = content
        .iter()
        .map(|line| {
            if line.is_empty() {
                Line::from("")
            } else if line.ends_with(':') {
                Line::from(Span::styled(line.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            } else {
                let parts: Vec<&str> = line.splitn(2, "    ").collect();
                if parts.len() == 2 {
                    Line::from(vec![
                        Span::styled(parts[0], Style::default().fg(Color::Cyan)),
                        Span::raw(parts[1]),
                    ])
                } else {
                    Line::from(Span::raw(line.clone()))
                }
            }
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(lines)).block(block), area);
}

/// 日志查看器
fn draw_log_viewer(f: &mut Frame, app: &App) {
    let area = centered_rect(85, 85, f.area());
    f.render_widget(Clear, area);

    let container_label = app
        .log_viewer
        .container
        .as_deref()
        .unwrap_or("default");
    let title = format!(
        " Pod 日志: {} / {} [容器: {}] ",
        app.log_viewer.namespace, app.log_viewer.pod_name, container_label
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let lines: Vec<Line> = app
        .log_viewer
        .lines()
        .iter()
        .skip(app.log_scroll as usize)
        .map(|line| Line::from(Span::raw(*line)))
        .collect();

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    let hint_area = Rect {
        x: area.x + 2,
        y: area.y + area.height - 1,
        width: area.width - 4,
        height: 1,
    };
    let hint = Paragraph::new(Text::from(Line::from(vec![
        Span::styled("[↑/↓] 滚动  [PgUp/PgDn] 翻页  [c] 切换容器  [Esc/q] 关闭", Style::default().fg(Color::Gray)),
    ])));
    f.render_widget(hint, hint_area);
}

/// 容器选择器
fn draw_container_selector(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);

    let title = format!(" 选择容器 ({}) ", app.log_viewer.pod_name);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let current = app.log_viewer.container.as_deref();
    let mut lines: Vec<Line> = app
        .container_selector
        .items
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let is_selected = i == app.container_selector.state;
            let is_current = current.map_or(false, |c| strip_init_suffix(container) == c);
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("  {}  ", container), style))
        })
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[↑/↓] 选择  [Enter] 查看日志  [Esc] 取消",
        Style::default().fg(Color::Gray),
    )));

    f.render_widget(Paragraph::new(Text::from(lines)).block(block), area);
}

/// 搜索栏
fn draw_search_bar(f: &mut Frame, app: &App) {
    let area = Rect {
        x: 0,
        y: f.area().height - 2,
        width: f.area().width,
        height: 1,
    };

    let text = Text::from(Line::from(vec![
        Span::styled("搜索: ", Style::default().fg(Color::Yellow)),
        Span::styled(app.search_query.clone(), Style::default().fg(Color::White)),
        Span::styled("_", Style::default().fg(Color::Cyan)),
    ]));

    f.render_widget(Paragraph::new(text).style(Style::default().bg(Color::DarkGray)), area);
}

/// 错误 Toast
fn draw_error_toast(f: &mut Frame, message: &str) {
    let area = Rect {
        x: f.area().width.saturating_sub(50) / 2,
        y: f.area().height.saturating_sub(3),
        width: 50,
        height: 1,
    };

    let text = Text::from(Line::from(Span::styled(
        format!(" ⚠ {} ", message),
        Style::default().fg(Color::Black).bg(Color::Yellow),
    )));

    f.render_widget(Paragraph::new(text), area);
}

/// 成功 Toast
fn draw_success_toast(f: &mut Frame, message: &str) {
    let area = Rect {
        x: f.area().width.saturating_sub(50) / 2,
        y: f.area().height.saturating_sub(3),
        width: 50,
        height: 1,
    };

    let text = Text::from(Line::from(Span::styled(
        format!(" ✓ {} ", message),
        Style::default().fg(Color::Black).bg(Color::Green),
    )));

    f.render_widget(Paragraph::new(text), area);
}

/// 计算居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
