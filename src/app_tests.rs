//! App 状态管理单元测试与集成测试

use crate::app::{App, Popup, ResourceTab};
use crate::k8s::K8sResource;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// 创建一个无 K8s 客户端的 App，用于纯状态测试
fn create_test_app() -> App {
    App::new(None)
}

/// 快速构造 KeyEvent
fn key_char(c: char) -> KeyEvent {
    KeyEvent::from(KeyCode::Char(c))
}

fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::from(code)
}

/// 快速构造一个鼠标事件（列/行设为 0，不参与滚轮路由）
fn mouse(kind: MouseEventKind) -> MouseEvent {
    MouseEvent {
        kind,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    }
}

/// 添加测试资源到 App
fn add_test_resources(app: &mut App, resources: Vec<K8sResource>) {
    app.resources = resources;
    app.apply_filter();
}

// ============== ResourceTab 测试 ==============
#[test]
fn test_resource_tab_as_str() {
    assert_eq!(ResourceTab::Pod.as_str(), "Pod");
    assert_eq!(ResourceTab::ConfigMap.as_str(), "ConfigMap");
    assert_eq!(ResourceTab::Secret.as_str(), "Secret");
}

#[test]
fn test_resource_tab_index() {
    assert_eq!(ResourceTab::Pod.index(), 0);
    assert_eq!(ResourceTab::ConfigMap.index(), 1);
    assert_eq!(ResourceTab::Secret.index(), 2);
}

#[test]
fn test_resource_tab_count() {
    assert_eq!(ResourceTab::count(), 6);
}

#[test]
fn test_resource_tab_from_index() {
    assert_eq!(ResourceTab::from_index(0), Some(ResourceTab::Pod));
    assert_eq!(ResourceTab::from_index(1), Some(ResourceTab::ConfigMap));
    assert_eq!(ResourceTab::from_index(2), Some(ResourceTab::Secret));
    assert_eq!(ResourceTab::from_index(3), Some(ResourceTab::Deployment));
    assert_eq!(ResourceTab::from_index(4), Some(ResourceTab::Service));
    assert_eq!(ResourceTab::from_index(5), Some(ResourceTab::Node));
    assert_eq!(ResourceTab::from_index(6), None);
}

// ============== App 初始化测试 ==============
#[test]
fn test_app_new_without_client() {
    let app = create_test_app();
    assert_eq!(app.current_tab, ResourceTab::Pod);
    assert!(app.resources.is_empty());
    assert!(app.filtered_resources.is_empty());
    assert_eq!(app.popup, Popup::None);
    assert!(!app.search_mode);
    assert_eq!(app.current_namespace, "default");
    assert_eq!(app.current_context, "");
    assert!(app.k8s_client.is_none());
}

#[test]
fn test_app_initial_namespace_list() {
    let app = create_test_app();
    assert_eq!(app.namespaces, vec!["default", "all"]);
}

// ============== 资源列表导航测试 ==============
#[test]
fn test_app_next_item_empty() {
    let mut app = create_test_app();
    app.next_item();
    assert_eq!(app.table_state.selected(), None);
}

#[test]
fn test_app_next_item() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "pod-1".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "pod-2".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
        ],
    );
    app.table_state.select(Some(0));

    app.next_item();
    assert_eq!(app.table_state.selected(), Some(1));

    app.next_item(); // 循环
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn test_app_previous_item() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "pod-1".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "pod-2".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
        ],
    );
    app.table_state.select(Some(0));

    app.previous_item();
    assert_eq!(app.table_state.selected(), Some(1)); // 循环到末尾

    app.previous_item();
    assert_eq!(app.table_state.selected(), Some(0));
}

// ============== Tab 切换测试 ==============
#[tokio::test]
async fn test_app_switch_tab() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));
    app.search_query = "test".to_string();

    app.handle_key_event(key_char('2')).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::ConfigMap);
    assert_eq!(app.table_state.selected(), Some(0));
    assert!(app.search_query.is_empty());
}

#[tokio::test]
async fn test_app_next_tab() {
    let mut app = create_test_app();
    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::ConfigMap);

    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Secret);

    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Deployment);

    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Service);

    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Node);

    app.handle_key_event(key_code(KeyCode::Tab)).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Pod);
}

// ============== 搜索模式测试 ==============
#[tokio::test]
async fn test_app_search_mode_enter_and_exit() {
    let mut app = create_test_app();
    app.handle_key_event(key_char('/')).await.unwrap();
    assert!(app.search_mode);

    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert!(!app.search_mode);
    assert!(app.search_query.is_empty());
}

#[tokio::test]
async fn test_app_search_filter() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "nginx-frontend".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "redis-backend".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "postgres-db".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "3h".to_string(),
                extra: vec![],
            },
        ],
    );
    assert_eq!(app.filtered_resources.len(), 3);

    app.search_mode = true;
    app.handle_key_event(key_char('n')).await.unwrap();
    app.handle_key_event(key_char('g')).await.unwrap();
    app.handle_key_event(key_char('i')).await.unwrap();
    app.handle_key_event(key_char('n')).await.unwrap();
    app.handle_key_event(key_char('x')).await.unwrap();

    assert_eq!(app.search_query, "nginx");
    assert_eq!(app.filtered_resources.len(), 1);
    assert_eq!(app.resources[app.filtered_resources[0]].name, "nginx-frontend");
}

#[tokio::test]
async fn test_app_search_backspace() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "abc".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "abcd".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
        ],
    );

    app.search_mode = true;
    app.handle_key_event(key_char('a')).await.unwrap();
    app.handle_key_event(key_char('b')).await.unwrap();
    app.handle_key_event(key_char('c')).await.unwrap();
    app.handle_key_event(key_char('d')).await.unwrap();
    assert_eq!(app.filtered_resources.len(), 1);

    app.handle_key_event(key_code(KeyCode::Backspace)).await.unwrap();
    assert_eq!(app.search_query, "abc");
    assert_eq!(app.filtered_resources.len(), 2);
}

// ============== 弹窗状态测试 ==============
#[tokio::test]
async fn test_app_popup_help() {
    let mut app = create_test_app();
    app.handle_key_event(key_char('?')).await.unwrap();
    assert_eq!(app.popup, Popup::Help);

    app.handle_key_event(key_char('?')).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

#[tokio::test]
async fn test_app_popup_namespace_selector() {
    let mut app = create_test_app();
    app.handle_key_event(key_char('n')).await.unwrap();
    assert_eq!(app.popup, Popup::NamespaceSelector);
    assert_eq!(app.namespace_selector.items, vec!["default", "all"]);
}

#[tokio::test]
async fn test_app_popup_context_selector() {
    let mut app = create_test_app();
    app.contexts = vec!["minikube".to_string(), "docker-desktop".to_string()];
    app.handle_key_event(key_char('c')).await.unwrap();
    assert_eq!(app.popup, Popup::ContextSelector);
    assert_eq!(
        app.context_selector.items,
        vec!["minikube", "docker-desktop"]
    );
}

#[tokio::test]
async fn test_app_popup_confirm_delete_without_selection() {
    let mut app = create_test_app();
    app.handle_key_event(key_char('d')).await.unwrap();
    assert_eq!(app.popup, Popup::None); // 无选中资源时不弹出
}

#[tokio::test]
async fn test_app_popup_confirm_delete_with_selection() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));

    app.handle_key_event(key_char('d')).await.unwrap();
    assert_eq!(app.popup, Popup::ConfirmDelete);
    assert_eq!(app.confirm_dialog.message, "确认删除该资源？");
}

#[tokio::test]
async fn test_app_confirm_delete_cancel() {
    let mut app = create_test_app();
    app.popup = Popup::ConfirmDelete;
    app.confirm_dialog.set_message("确认删除？".to_string());

    app.handle_key_event(key_char('n')).await.unwrap();
    assert_eq!(app.popup, Popup::None);

    app.popup = Popup::ConfirmDelete;
    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

#[tokio::test]
async fn test_app_detail_popup_scroll() {
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    app.detail_scroll = 5;

    app.handle_key_event(key_code(KeyCode::Down)).await.unwrap();
    assert_eq!(app.detail_scroll, 6);

    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    assert_eq!(app.detail_scroll, 5);

    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    assert_eq!(app.detail_scroll, 0); // saturating_sub
}

#[tokio::test]
async fn test_app_detail_popup_page_scroll() {
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    app.detail_scroll = 3;

    app.handle_key_event(key_code(KeyCode::PageDown))
        .await
        .unwrap();
    assert_eq!(app.detail_scroll, 13);

    app.handle_key_event(key_code(KeyCode::PageUp)).await.unwrap();
    assert_eq!(app.detail_scroll, 3);

    app.handle_key_event(key_code(KeyCode::PageUp)).await.unwrap();
    assert_eq!(app.detail_scroll, 0); // saturating_sub clamps at 0
}

// ============== 鼠标事件测试 ==============

#[tokio::test]
async fn test_app_mouse_wheel_scrolls_resource_list() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "pod-a".into(),
                namespace: "default".into(),
                status: "Running".into(),
                age: "1m".into(),
                extra: vec![],
            },
            K8sResource {
                name: "pod-b".into(),
                namespace: "default".into(),
                status: "Running".into(),
                age: "1m".into(),
                extra: vec![],
            },
        ],
    );
    app.table_state.select(Some(0));

    app.handle_mouse_event(mouse(MouseEventKind::ScrollDown))
        .await
        .unwrap();
    assert_eq!(app.table_state.selected(), Some(1));

    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    assert_eq!(app.table_state.selected(), Some(0));
}

#[tokio::test]
async fn test_app_mouse_wheel_scrolls_detail_popup() {
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    app.detail_scroll = 5;

    app.handle_mouse_event(mouse(MouseEventKind::ScrollDown))
        .await
        .unwrap();
    assert_eq!(app.detail_scroll, 8); // +3

    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    assert_eq!(app.detail_scroll, 5); // -3

    // 触底 saturating_sub
    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    assert_eq!(app.detail_scroll, 0);
}

#[tokio::test]
async fn test_app_mouse_wheel_scrolls_log_viewer() {
    let mut app = create_test_app();
    app.popup = Popup::LogViewer;
    app.log_scroll = 4;

    app.handle_mouse_event(mouse(MouseEventKind::ScrollDown))
        .await
        .unwrap();
    assert_eq!(app.log_scroll, 7); // +3

    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    assert_eq!(app.log_scroll, 4);
}

#[tokio::test]
async fn test_app_mouse_wheel_moves_namespace_selector() {
    let mut app = create_test_app();
    app.popup = Popup::NamespaceSelector;
    app.namespace_selector
        .set_items(vec!["default".into(), "kube-system".into(), "all".into()]);

    app.handle_mouse_event(mouse(MouseEventKind::ScrollDown))
        .await
        .unwrap();
    assert_eq!(app.namespace_selector.state, 1);

    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();
    assert_eq!(app.namespace_selector.state, 0);
}

#[tokio::test]
async fn test_app_mouse_click_is_currently_ignored() {
    // 当前阶段只接 wheel；其他事件（点击、移动）必须无副作用。
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    app.detail_scroll = 7;

    app.handle_mouse_event(mouse(MouseEventKind::Down(
        crossterm::event::MouseButton::Left,
    )))
    .await
    .unwrap();
    app.handle_mouse_event(mouse(MouseEventKind::Moved))
        .await
        .unwrap();

    assert_eq!(app.detail_scroll, 7);
    assert_eq!(app.popup, Popup::Detail);
}

#[tokio::test]
async fn test_app_mouse_wheel_ignored_in_confirm_dialog() {
    // 确认与帮助弹窗的滚轮应被忽略，以免误操作。
    let mut app = create_test_app();
    app.popup = Popup::ConfirmDelete;
    let before = app.popup.clone();

    app.handle_mouse_event(mouse(MouseEventKind::ScrollDown))
        .await
        .unwrap();
    app.handle_mouse_event(mouse(MouseEventKind::ScrollUp))
        .await
        .unwrap();

    assert_eq!(app.popup, before);
}

// ============== 日志查看器测试 ==============
#[tokio::test]
async fn test_app_log_viewer_popup() {
    let mut app = create_test_app();
    app.current_tab = ResourceTab::Pod;
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "nginx-123".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));

    app.handle_key_event(key_char('l')).await.unwrap();
    assert_eq!(app.popup, Popup::LogViewer);
    assert_eq!(app.log_viewer.pod_name, "nginx-123");
    assert_eq!(app.log_viewer.namespace, "default");
}

#[tokio::test]
async fn test_app_log_viewer_non_pod_tab() {
    let mut app = create_test_app();
    app.current_tab = ResourceTab::ConfigMap;
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "my-config".to_string(),
            namespace: "default".to_string(),
            status: "-".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));

    app.handle_key_event(key_char('l')).await.unwrap();
    assert_eq!(app.popup, Popup::None); // 非 Pod 标签不打开日志
}

#[tokio::test]
async fn test_app_log_scroll() {
    let mut app = create_test_app();
    app.popup = Popup::LogViewer;
    app.log_scroll = 3;

    app.handle_key_event(key_code(KeyCode::Down)).await.unwrap();
    assert_eq!(app.log_scroll, 4);

    app.handle_key_event(key_code(KeyCode::PageDown))
        .await
        .unwrap();
    assert_eq!(app.log_scroll, 14);

    app.handle_key_event(key_code(KeyCode::PageUp)).await.unwrap();
    assert_eq!(app.log_scroll, 4);
}

// ============== 退出测试 ==============
#[tokio::test]
async fn test_app_quit() {
    let mut app = create_test_app();
    let should_quit = app.handle_key_event(key_char('q')).await.unwrap();
    assert!(should_quit);
}

// ============== selected_resource 测试 ==============
#[test]
fn test_app_selected_resource() {
    let mut app = create_test_app();
    assert_eq!(app.selected_resource(), None);

    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));

    assert_eq!(app.selected_resource().unwrap().name, "pod-1");
}

// ============== apply_filter 保持选中位置测试 ==============
#[test]
fn test_app_apply_filter_keeps_selection() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "aaa".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "bbb".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "ccc".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "3h".to_string(),
                extra: vec![],
            },
        ],
    );
    app.table_state.select(Some(2));
    app.search_query = "b".to_string();
    app.apply_filter();

    // 过滤后只剩 bbb，选中位置应被限制在有效范围
    assert_eq!(app.filtered_resources.len(), 1);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn test_app_apply_filter_empty_result() {
    let mut app = create_test_app();
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));
    app.search_query = "zzz".to_string();
    app.apply_filter();

    assert!(app.filtered_resources.is_empty());
    assert_eq!(app.table_state.selected(), None);
}

// ============== Namespace 选择器交互测试 ==============
#[tokio::test]
async fn test_app_namespace_selector_navigation_and_select() {
    let mut app = create_test_app();
    app.namespaces = vec![
        "default".to_string(),
        "kube-system".to_string(),
        "all".to_string(),
    ];
    app.handle_key_event(key_char('n')).await.unwrap();
    assert_eq!(app.popup, Popup::NamespaceSelector);

    app.handle_key_event(key_code(KeyCode::Down)).await.unwrap();
    assert_eq!(app.namespace_selector.state, 1);

    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    assert_eq!(app.namespace_selector.state, 0);

    app.handle_key_event(key_code(KeyCode::Enter)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
    assert_eq!(app.current_namespace, "default");
}

#[tokio::test]
async fn test_app_namespace_selector_esc() {
    let mut app = create_test_app();
    app.popup = Popup::NamespaceSelector;
    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

// ============== Context 选择器交互测试 ==============
#[tokio::test]
async fn test_app_context_selector_navigation() {
    let mut app = create_test_app();
    app.contexts = vec![
        "ctx-a".to_string(),
        "ctx-b".to_string(),
        "ctx-c".to_string(),
    ];
    app.handle_key_event(key_char('c')).await.unwrap();
    assert_eq!(app.popup, Popup::ContextSelector);

    app.handle_key_event(key_code(KeyCode::Down)).await.unwrap();
    assert_eq!(app.context_selector.state, 1);

    app.handle_key_event(key_code(KeyCode::Up)).await.unwrap();
    assert_eq!(app.context_selector.state, 0);

    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

// ============== Detail 弹窗内按键测试 ==============
#[tokio::test]
async fn test_app_detail_popup_esc() {
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

#[tokio::test]
async fn test_app_detail_popup_delete_trigger_confirm() {
    let mut app = create_test_app();
    app.popup = Popup::Detail;
    add_test_resources(
        &mut app,
        vec![K8sResource {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1h".to_string(),
            extra: vec![],
        }],
    );
    app.table_state.select(Some(0));

    app.handle_key_event(key_char('d')).await.unwrap();
    assert_eq!(app.popup, Popup::ConfirmDelete);
}

// ============== 综合集成测试：模拟用户操作流程 ==============
#[tokio::test]
async fn test_user_flow_navigate_and_search() {
    let mut app = create_test_app();

    // 1. 初始在 Pod 标签
    assert_eq!(app.current_tab, ResourceTab::Pod);

    // 2. 切换到 ConfigMap
    app.handle_key_event(key_char('2')).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::ConfigMap);

    // 3. 切回 Pod
    app.handle_key_event(key_char('1')).await.unwrap();
    assert_eq!(app.current_tab, ResourceTab::Pod);

    // 4. 设置资源并导航
    add_test_resources(
        &mut app,
        vec![
            K8sResource {
                name: "nginx".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "1h".to_string(),
                extra: vec![],
            },
            K8sResource {
                name: "redis".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                age: "2h".to_string(),
                extra: vec![],
            },
        ],
    );
    app.table_state.select(Some(0));

    app.next_item();
    assert_eq!(app.table_state.selected(), Some(1));

    // 5. 进入搜索模式
    app.handle_key_event(key_char('/')).await.unwrap();
    assert!(app.search_mode);

    app.handle_key_event(key_char('n')).await.unwrap();
    app.handle_key_event(key_char('g')).await.unwrap();
    app.handle_key_event(key_char('i')).await.unwrap();
    app.handle_key_event(key_char('n')).await.unwrap();
    app.handle_key_event(key_code(KeyCode::Enter)).await.unwrap();
    assert!(!app.search_mode);
    assert_eq!(app.filtered_resources.len(), 1);
    assert_eq!(app.resources[app.filtered_resources[0]].name, "nginx");

    // 6. 查看详情
    app.handle_key_event(key_code(KeyCode::Enter)).await.unwrap();
    assert_eq!(app.popup, Popup::Detail);

    // 7. 关闭详情
    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);

    // 8. 退出
    let should_quit = app.handle_key_event(key_char('q')).await.unwrap();
    assert!(should_quit);
}

#[tokio::test]
async fn test_user_flow_help_and_close() {
    let mut app = create_test_app();
    app.handle_key_event(key_char('?')).await.unwrap();
    assert_eq!(app.popup, Popup::Help);

    app.handle_key_event(key_code(KeyCode::Esc)).await.unwrap();
    assert_eq!(app.popup, Popup::None);
}

// ============== 消息管理测试 ==============
#[test]
fn test_app_message_management() {
    let mut app = create_test_app();

    app.set_error("连接失败".to_string());
    assert_eq!(app.error_message, Some("连接失败".to_string()));
    assert_eq!(app.success_message, None);

    app.set_success("操作成功".to_string());
    assert_eq!(app.success_message, Some("操作成功".to_string()));
    assert_eq!(app.error_message, None);

    app.clear_messages();
    assert_eq!(app.error_message, None);
    assert_eq!(app.success_message, None);
}

// ============== friendly_error 测试 ==============
#[test]
fn test_friendly_error_messages() {
    use anyhow::anyhow;

    assert_eq!(
        crate::app::friendly_error(&anyhow!("connection refused")),
        "无法连接到 K8s API 服务器，请检查集群是否可访问"
    );
    assert_eq!(
        crate::app::friendly_error(&anyhow!("Unauthorized")),
        "认证失败，请检查 kubeconfig 配置"
    );
    assert_eq!(
        crate::app::friendly_error(&anyhow!("not found")),
        "请求的资源不存在"
    );
    assert_eq!(
        crate::app::friendly_error(&anyhow!("timeout")),
        "请求超时，请检查网络连接"
    );
    assert_eq!(
        crate::app::friendly_error(&anyhow!("kubeconfig error")),
        "kubeconfig 配置错误，请检查 ~/.kube/config"
    );
    assert!(
        crate::app::friendly_error(&anyhow!("unknown error"))
            .starts_with("操作失败:")
    );
}
