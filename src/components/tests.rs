//! 组件模块的单元测试

use super::*;

// ============== ConfirmDialog 测试 ==============
#[test]
fn test_confirm_dialog_new() {
    let dialog = ConfirmDialog::new();
    assert!(dialog.message.is_empty());
}

#[test]
fn test_confirm_dialog_set_message() {
    let mut dialog = ConfirmDialog::new();
    dialog.set_message("确认删除？".to_string());
    assert_eq!(dialog.message, "确认删除？");
}

// ============== ContextSelector 测试 ==============
#[test]
fn test_context_selector_new() {
    let selector = ContextSelector::new();
    assert!(selector.items.is_empty());
    assert_eq!(selector.state, 0);
}

#[test]
fn test_context_selector_set_items() {
    let mut selector = ContextSelector::new();
    selector.set_items(vec!["ctx1".to_string(), "ctx2".to_string()]);
    assert_eq!(selector.items.len(), 2);
    assert_eq!(selector.state, 0);
}

#[test]
fn test_context_selector_next() {
    let mut selector = ContextSelector::new();
    selector.set_items(vec![
        "ctx1".to_string(),
        "ctx2".to_string(),
        "ctx3".to_string(),
    ]);

    assert_eq!(selector.state, 0);
    selector.next();
    assert_eq!(selector.state, 1);
    selector.next();
    assert_eq!(selector.state, 2);
    selector.next(); // 循环到开头
    assert_eq!(selector.state, 0);
}

#[test]
fn test_context_selector_previous() {
    let mut selector = ContextSelector::new();
    selector.set_items(vec![
        "ctx1".to_string(),
        "ctx2".to_string(),
        "ctx3".to_string(),
    ]);

    selector.previous(); // 循环到末尾
    assert_eq!(selector.state, 2);
    selector.previous();
    assert_eq!(selector.state, 1);
}

#[test]
fn test_context_selector_empty_navigation() {
    let mut selector = ContextSelector::new();
    selector.next(); // 空列表不应 panic
    selector.previous();
    assert_eq!(selector.state, 0);
}

#[test]
fn test_context_selector_selected() {
    let mut selector = ContextSelector::new();
    assert_eq!(selector.selected(), None);

    selector.set_items(vec!["ctx1".to_string(), "ctx2".to_string()]);
    assert_eq!(selector.selected(), Some("ctx1".to_string()));

    selector.next();
    assert_eq!(selector.selected(), Some("ctx2".to_string()));
}

// ============== NamespaceSelector 测试 ==============
#[test]
fn test_namespace_selector_new() {
    let selector = NamespaceSelector::new();
    assert!(selector.items.is_empty());
    assert_eq!(selector.state, 0);
}

#[test]
fn test_namespace_selector_set_items() {
    let mut selector = NamespaceSelector::new();
    selector.set_items(vec!["default".to_string(), "kube-system".to_string()]);
    assert_eq!(selector.items.len(), 2);
    assert_eq!(selector.state, 0);
}

#[test]
fn test_namespace_selector_select_item() {
    let mut selector = NamespaceSelector::new();
    selector.set_items(vec![
        "default".to_string(),
        "kube-system".to_string(),
        "all".to_string(),
    ]);

    selector.select_item("kube-system");
    assert_eq!(selector.state, 1);

    selector.select_item("all");
    assert_eq!(selector.state, 2);

    selector.select_item("nonexistent"); // 不存在的项不应改变 state
    assert_eq!(selector.state, 2);
}

#[test]
fn test_namespace_selector_next_and_previous() {
    let mut selector = NamespaceSelector::new();
    selector.set_items(vec!["ns1".to_string(), "ns2".to_string()]);

    selector.next();
    assert_eq!(selector.state, 1);
    selector.next();
    assert_eq!(selector.state, 0);

    selector.previous();
    assert_eq!(selector.state, 1);
    selector.previous();
    assert_eq!(selector.state, 0);
}

#[test]
fn test_namespace_selector_empty_navigation() {
    let mut selector = NamespaceSelector::new();
    selector.next();
    selector.previous();
    assert_eq!(selector.state, 0);
}

#[test]
fn test_namespace_selector_selected() {
    let mut selector = NamespaceSelector::new();
    selector.set_items(vec!["default".to_string(), "kube-system".to_string()]);
    assert_eq!(selector.selected(), Some("default".to_string()));
}

// ============== LogViewer 测试 ==============
#[test]
fn test_log_viewer_new() {
    let viewer = LogViewer::new();
    assert!(viewer.pod_name.is_empty());
    assert!(viewer.namespace.is_empty());
    assert!(viewer.logs.is_empty());
}

#[test]
fn test_log_viewer_setters() {
    let mut viewer = LogViewer::new();
    viewer.set_pod_name("nginx-123".to_string());
    viewer.set_namespace("default".to_string());
    viewer.set_logs("line1\nline2\nline3".to_string());

    assert_eq!(viewer.pod_name, "nginx-123");
    assert_eq!(viewer.namespace, "default");
    assert_eq!(viewer.logs, "line1\nline2\nline3");
}

#[test]
fn test_log_viewer_lines() {
    let mut viewer = LogViewer::new();
    viewer.set_logs("line1\nline2\nline3".to_string());
    let lines = viewer.lines();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");
}

#[test]
fn test_log_viewer_empty_lines() {
    let viewer = LogViewer::new();
    let lines = viewer.lines();
    assert!(lines.is_empty());
}

#[test]
fn test_log_viewer_container() {
    let mut viewer = LogViewer::new();
    assert_eq!(viewer.container, None);
    viewer.set_container(Some("app".to_string()));
    assert_eq!(viewer.container, Some("app".to_string()));
    viewer.set_container(None);
    assert_eq!(viewer.container, None);
}

// ============== ContainerSelector 测试 ==============
#[test]
fn test_container_selector_new() {
    let selector = ContainerSelector::new();
    assert!(selector.items.is_empty());
    assert_eq!(selector.state, 0);
}

#[test]
fn test_container_selector_set_items_resets_state() {
    let mut selector = ContainerSelector::new();
    selector.set_items(vec!["app".to_string(), "sidecar".to_string()]);
    selector.next();
    assert_eq!(selector.state, 1);
    selector.set_items(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(selector.items.len(), 3);
    assert_eq!(selector.state, 0);
}

#[test]
fn test_container_selector_next_and_previous_wrap() {
    let mut selector = ContainerSelector::new();
    selector.set_items(vec!["app".to_string(), "sidecar".to_string()]);
    selector.next();
    assert_eq!(selector.state, 1);
    selector.next();
    assert_eq!(selector.state, 0);
    selector.previous();
    assert_eq!(selector.state, 1);
}

#[test]
fn test_container_selector_empty_navigation() {
    let mut selector = ContainerSelector::new();
    selector.next();
    selector.previous();
    assert_eq!(selector.state, 0);
    assert_eq!(selector.selected(), None);
}

#[test]
fn test_container_selector_selected() {
    let mut selector = ContainerSelector::new();
    selector.set_items(vec!["app".to_string(), "sidecar".to_string()]);
    assert_eq!(selector.selected(), Some("app".to_string()));
    selector.next();
    assert_eq!(selector.selected(), Some("sidecar".to_string()));
}

#[test]
fn test_strip_init_suffix() {
    use super::container_selector::strip_init_suffix;
    assert_eq!(strip_init_suffix("init-db (init)"), "init-db");
    assert_eq!(strip_init_suffix("app"), "app");
    assert_eq!(strip_init_suffix("weird (init) name"), "weird (init) name");
}

// ============== ResourceDetail 测试 ==============
#[test]
fn test_resource_detail_new() {
    let detail = ResourceDetail::new();
    assert!(detail.content.is_empty());
}

#[test]
fn test_resource_detail_set_content() {
    let mut detail = ResourceDetail::new();
    detail.set_content("apiVersion: v1\nkind: Pod".to_string());
    assert_eq!(detail.content, "apiVersion: v1\nkind: Pod");
}

#[test]
fn test_resource_detail_lines() {
    let mut detail = ResourceDetail::new();
    detail.set_content("line1\nline2".to_string());
    let lines = detail.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
}

// ============== HelpPopup 测试 ==============
#[test]
fn test_help_popup_content() {
    let content = HelpPopup::content();
    assert!(!content.is_empty());
    assert!(content.iter().any(|line| line.contains("快捷键帮助")));
    assert!(content.iter().any(|line| line.contains("导航:")));
    assert!(content.iter().any(|line| line.contains("操作:")));
}

#[test]
fn test_search_bar_new() {
    let _bar = SearchBar::new();
}
