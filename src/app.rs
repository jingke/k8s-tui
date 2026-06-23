use crate::components::container_selector::strip_init_suffix;
use crate::components::{ConfirmDialog, ContainerSelector, ContextSelector, HelpPopup, LogViewer, NamespaceSelector, ResourceDetail, SearchBar};
use crate::k8s::{K8sClient, K8sResource};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::widgets::TableState;
use std::time::{Duration, Instant};

/// 每次鼠标滚轮事件在内容弹窗（详情 / 日志）中滚动的行数。
///
/// 介于键盘 `↑/↓`（1 行）与 `PgUp/PgDn`（10 行）之间，给滚轮一个
/// 顺手又精细可控的速度。
const MOUSE_WHEEL_LINES: u16 = 3;

/// 资源类型标签
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceTab {
    Pod,
    ConfigMap,
    Secret,
    Deployment,
    Service,
    Node,
}

impl ResourceTab {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ResourceTab::Pod => "Pod",
            ResourceTab::ConfigMap => "ConfigMap",
            ResourceTab::Secret => "Secret",
            ResourceTab::Deployment => "Deployment",
            ResourceTab::Service => "Service",
            ResourceTab::Node => "Node",
        }
    }

    pub const fn index(&self) -> usize {
        match self {
            ResourceTab::Pod => 0,
            ResourceTab::ConfigMap => 1,
            ResourceTab::Secret => 2,
            ResourceTab::Deployment => 3,
            ResourceTab::Service => 4,
            ResourceTab::Node => 5,
        }
    }

    pub const fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(ResourceTab::Pod),
            1 => Some(ResourceTab::ConfigMap),
            2 => Some(ResourceTab::Secret),
            3 => Some(ResourceTab::Deployment),
            4 => Some(ResourceTab::Service),
            5 => Some(ResourceTab::Node),
            _ => None,
        }
    }

    pub const fn count() -> usize {
        6
    }

    pub const fn supports_logs(&self) -> bool {
        matches!(self, ResourceTab::Pod)
    }

    pub const fn supports_delete(&self) -> bool {
        !matches!(self, ResourceTab::Node)
    }
}

/// 当前弹窗状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Popup {
    None,
    Detail,           // 资源详情
    ConfirmDelete,    // 删除确认
    ContextSelector,  // 上下文选择
    NamespaceSelector,// 命名空间选择
    Help,             // 帮助页
    LogViewer,        // 日志查看
    ContainerSelector,// 容器选择（多容器 Pod 查看日志前）
}

/// 用户友好的错误信息
pub(crate) fn friendly_error(err: &anyhow::Error) -> String {
    let msg = err.to_string();
    if msg.contains("connection refused") || msg.contains("Connection refused") {
        "无法连接到 K8s API 服务器，请检查集群是否可访问".to_string()
    } else if msg.contains("Unauthorized") || msg.contains("unauthorized") {
        "认证失败，请检查 kubeconfig 配置".to_string()
    } else if msg.contains("not found") || msg.contains("NotFound") {
        "请求的资源不存在".to_string()
    } else if msg.contains("timeout") || msg.contains("Timeout") {
        "请求超时，请检查网络连接".to_string()
    } else if msg.contains("kubeconfig") {
        "kubeconfig 配置错误，请检查 ~/.kube/config".to_string()
    } else {
        format!("操作失败: {}", msg)
    }
}

/// 应用状态
pub struct App {
    pub k8s_client: Option<K8sClient>,
    pub current_tab: ResourceTab,
    pub resources: Vec<K8sResource>,
    pub filtered_resources: Vec<usize>, // 存储索引而非克隆资源
    pub table_state: TableState,
    pub popup: Popup,
    pub search_query: String,
    pub search_mode: bool,
    pub namespaces: Vec<String>,
    pub current_namespace: String, // "all" 表示全部命名空间
    pub contexts: Vec<String>,
    pub current_context: String,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub detail_scroll: u16,
    pub log_scroll: u16,
    pub confirm_dialog: ConfirmDialog,
    pub context_selector: ContextSelector,
    pub namespace_selector: NamespaceSelector,
    pub container_selector: ContainerSelector,
    #[allow(dead_code)]
    pub help_popup: HelpPopup,
    #[allow(dead_code)]
    pub log_viewer: LogViewer,
    #[allow(dead_code)]
    pub search_bar: SearchBar,
    pub detail_popup: ResourceDetail,
}

impl App {
    pub fn new(k8s_client: Option<K8sClient>) -> Self {
        let mut app = Self {
            k8s_client,
            current_tab: ResourceTab::Pod,
            resources: Vec::new(),
            filtered_resources: Vec::new(),
            table_state: TableState::default(),
            popup: Popup::None,
            search_query: String::new(),
            search_mode: false,
            namespaces: vec!["default".to_string(), "all".to_string()],
            current_namespace: "default".to_string(),
            contexts: Vec::new(),
            current_context: String::new(),
            last_refresh: Instant::now() - Duration::from_secs(10),
            refresh_interval: Duration::from_secs(2),
            error_message: None,
            success_message: None,
            detail_scroll: 0,
            log_scroll: 0,
            confirm_dialog: ConfirmDialog::new(),
            context_selector: ContextSelector::new(),
            namespace_selector: NamespaceSelector::new(),
            container_selector: ContainerSelector::new(),
            help_popup: HelpPopup::new(),
            log_viewer: LogViewer::new(),
            search_bar: SearchBar::new(),
            detail_popup: ResourceDetail::new(),
        };

        if let Some(client) = &app.k8s_client {
            app.current_context = client.current_context.clone();
            app.contexts = client.contexts.clone();
        }

        app
    }

    /// 定时刷新逻辑
    pub async fn on_tick(&mut self) {
        if self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_resources().await;
            self.last_refresh = Instant::now();
        }
    }

    /// 刷新资源列表
    pub async fn refresh_resources(&mut self) {
        let Some(client) = &self.k8s_client else { return };

        let ns = if self.current_namespace == "all" {
            None
        } else {
            Some(self.current_namespace.clone())
        };

        let resource_result = match self.current_tab {
            ResourceTab::Pod => client.list_pods(ns).await,
            ResourceTab::ConfigMap => client.list_configmaps(ns).await,
            ResourceTab::Secret => client.list_secrets(ns).await,
            ResourceTab::Deployment => client.list_deployments(ns).await,
            ResourceTab::Service => client.list_services(ns).await,
            ResourceTab::Node => client.list_nodes().await,
        };

        // 刷新命名空间列表（后台，失败不影响主流程）
        if let Ok(ns_list) = client.list_namespaces().await {
            self.namespaces = ns_list;
            if !self.namespaces.iter().any(|n| n == "all") {
                self.namespaces.insert(0, "all".to_string());
            }
        }

        match resource_result {
            Ok(resources) => {
                self.resources = resources;
                self.apply_filter();
                self.clear_messages();
            }
            Err(e) => {
                self.set_error(friendly_error(&e));
            }
        }
    }

    /// 应用搜索过滤（存储索引而非克隆）
    pub(crate) fn apply_filter(&mut self) {
        self.filtered_resources.clear();

        if self.search_query.is_empty() {
            self.filtered_resources.extend(0..self.resources.len());
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_resources.extend(
                self.resources
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.name.to_lowercase().contains(&query))
                    .map(|(i, _)| i),
            );
        }

        // 保持选中位置有效
        let len = self.filtered_resources.len();
        if len == 0 {
            self.table_state.select(None);
        } else {
            let current = self.table_state.selected().unwrap_or(0);
            if current >= len {
                self.table_state.select(Some(len - 1));
            } else if self.table_state.selected().is_none() {
                self.table_state.select(Some(0));
            }
        }
    }

    /// 获取当前选中的资源（引用）
    pub fn selected_resource(&self) -> Option<&K8sResource> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_resources.get(i))
            .and_then(|&idx| self.resources.get(idx))
    }

    /// 获取当前选中的资源索引
    fn selected_resource_index(&self) -> Option<usize> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_resources.get(i).copied())
    }

    /// 处理键盘事件
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // 搜索模式优先
        if self.search_mode {
            return self.handle_search_keys(key);
        }

        // 弹窗模式
        match &self.popup {
            Popup::Detail => return self.handle_detail_keys(key).await,
            Popup::ConfirmDelete => return self.handle_confirm_keys(key).await,
            Popup::ContextSelector => return self.handle_context_keys(key).await,
            Popup::NamespaceSelector => return self.handle_namespace_keys(key).await,
            Popup::Help => {
                if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc {
                    self.popup = Popup::None;
                }
                return Ok(false);
            }
            Popup::LogViewer => return self.handle_log_keys(key).await,
            Popup::ContainerSelector => return self.handle_container_keys(key).await,
            Popup::None => {}
        }

        // 正常列表视图
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.popup = Popup::Help,
            KeyCode::Char('r') => self.refresh_resources().await,
            KeyCode::Char('c') => {
                self.popup = Popup::ContextSelector;
                self.context_selector.set_items(self.contexts.clone());
            }
            KeyCode::Char('n') => {
                self.popup = Popup::NamespaceSelector;
                self.namespace_selector.set_items(self.namespaces.clone());
                self.namespace_selector.select_item(&self.current_namespace);
            }
            KeyCode::Char('/') => self.search_mode = true,
            KeyCode::Char('d') => {
                if self.current_tab.supports_delete() {
                    self.show_delete_confirm();
                }
            }
            KeyCode::Char('l') => {
                if self.current_tab.supports_logs() {
                    self.show_logs().await;
                }
            }
            KeyCode::Tab => self.next_tab(),
            KeyCode::Char('1') => self.switch_tab(ResourceTab::Pod),
            KeyCode::Char('2') => self.switch_tab(ResourceTab::ConfigMap),
            KeyCode::Char('3') => self.switch_tab(ResourceTab::Secret),
            KeyCode::Char('4') => self.switch_tab(ResourceTab::Deployment),
            KeyCode::Char('5') => self.switch_tab(ResourceTab::Service),
            KeyCode::Char('6') => self.switch_tab(ResourceTab::Node),
            KeyCode::Down | KeyCode::Char('j') => self.next_item(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_item(),
            KeyCode::Enter => self.show_detail().await,
            _ => {}
        }

        Ok(false)
    }

    /// 搜索模式按键
    fn handle_search_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                self.apply_filter();
            }
            KeyCode::Enter => {
                self.search_mode = false;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filter();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filter();
            }
            _ => {}
        }
        Ok(false)
    }

    /// 详情弹窗按键
    async fn handle_detail_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => self.popup = Popup::None,
            KeyCode::Char('d') => self.show_delete_confirm(),
            KeyCode::Char('l') => self.show_logs().await,
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.detail_scroll = self.detail_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.detail_scroll = self.detail_scroll.saturating_sub(10);
            }
            _ => {}
        }
        Ok(false)
    }

    /// 确认对话框按键
    async fn handle_confirm_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.popup = Popup::None;
                if let Some(idx) = self.selected_resource_index() {
                    let name = self.resources[idx].name.clone();
                    let namespace = self.resources[idx].namespace.clone();
                    let res = K8sResource {
                        name,
                        namespace,
                        status: String::new(),
                        age: String::new(),
                        extra: vec![],
                    };
                    self.delete_resource(&res).await;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.popup = Popup::None;
            }
            _ => {}
        }
        Ok(false)
    }

    /// Context 选择器按键
    async fn handle_context_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => self.popup = Popup::None,
            KeyCode::Enter => {
                if let Some(ctx) = self.context_selector.selected() {
                    self.switch_context(ctx).await;
                }
                self.popup = Popup::None;
            }
            KeyCode::Down | KeyCode::Char('j') => self.context_selector.next(),
            KeyCode::Up | KeyCode::Char('k') => self.context_selector.previous(),
            _ => {}
        }
        Ok(false)
    }

    /// 容器选择器按键
    async fn handle_container_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.popup = Popup::None,
            KeyCode::Enter => {
                if let Some(display_name) = self.container_selector.selected() {
                    let container = strip_init_suffix(&display_name).to_string();
                    self.log_viewer.set_container(Some(container));
                    self.popup = Popup::LogViewer;
                    self.log_scroll = 0;
                    self.load_logs().await;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.container_selector.next(),
            KeyCode::Up | KeyCode::Char('k') => self.container_selector.previous(),
            _ => {}
        }
        Ok(false)
    }

    /// Namespace 选择器按键
    async fn handle_namespace_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => self.popup = Popup::None,
            KeyCode::Enter => {
                if let Some(ns) = self.namespace_selector.selected() {
                    self.current_namespace = ns;
                    self.refresh_resources().await;
                }
                self.popup = Popup::None;
            }
            KeyCode::Down | KeyCode::Char('j') => self.namespace_selector.next(),
            KeyCode::Up | KeyCode::Char('k') => self.namespace_selector.previous(),
            _ => {}
        }
        Ok(false)
    }

    /// 日志查看器按键
    async fn handle_log_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.popup = Popup::None,
            KeyCode::Char('c') => self.open_container_selector().await,
            KeyCode::Down | KeyCode::Char('j') => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.log_scroll = self.log_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.log_scroll = self.log_scroll.saturating_sub(10);
            }
            _ => {}
        }
        Ok(false)
    }

    /// 鼠标事件入口：当前仅处理滚轮（点击留待后续按 Rect 命中扩展）
    ///
    /// 滚轮在不同弹窗下路由到不同的滚动状态：
    /// - 无弹窗：滚动资源列表（每格 1 行）
    /// - 详情 / 日志：滚动正文（每格 [`MOUSE_WHEEL_LINES`] 行）
    /// - 选择器（命名空间 / 上下文 / 容器）：移动选中项（每格 1 项）
    /// - 其他弹窗（确认、帮助）：忽略
    pub async fn handle_mouse_event(&mut self, m: MouseEvent) -> Result<()> {
        match (self.popup.clone(), m.kind) {
            (Popup::None, MouseEventKind::ScrollDown) => self.next_item(),
            (Popup::None, MouseEventKind::ScrollUp) => self.previous_item(),

            (Popup::Detail, MouseEventKind::ScrollDown) => {
                self.detail_scroll = self.detail_scroll.saturating_add(MOUSE_WHEEL_LINES);
            }
            (Popup::Detail, MouseEventKind::ScrollUp) => {
                self.detail_scroll = self.detail_scroll.saturating_sub(MOUSE_WHEEL_LINES);
            }

            (Popup::LogViewer, MouseEventKind::ScrollDown) => {
                self.log_scroll = self.log_scroll.saturating_add(MOUSE_WHEEL_LINES);
            }
            (Popup::LogViewer, MouseEventKind::ScrollUp) => {
                self.log_scroll = self.log_scroll.saturating_sub(MOUSE_WHEEL_LINES);
            }

            (Popup::NamespaceSelector, MouseEventKind::ScrollDown) => {
                self.namespace_selector.next();
            }
            (Popup::NamespaceSelector, MouseEventKind::ScrollUp) => {
                self.namespace_selector.previous();
            }

            (Popup::ContextSelector, MouseEventKind::ScrollDown) => {
                self.context_selector.next();
            }
            (Popup::ContextSelector, MouseEventKind::ScrollUp) => {
                self.context_selector.previous();
            }

            (Popup::ContainerSelector, MouseEventKind::ScrollDown) => {
                self.container_selector.next();
            }
            (Popup::ContainerSelector, MouseEventKind::ScrollUp) => {
                self.container_selector.previous();
            }

            _ => {}
        }
        Ok(())
    }

    // --- 辅助方法 ---

    fn show_delete_confirm(&mut self) {
        if self.selected_resource().is_some() {
            self.popup = Popup::ConfirmDelete;
            self.confirm_dialog.set_message("确认删除该资源？".to_string());
        }
    }

    async fn show_logs(&mut self) {
        if self.current_tab != ResourceTab::Pod {
            return;
        }
        let (name, namespace) = self.selected_resource()
            .map(|res| (res.name.clone(), res.namespace.clone()))
            .unwrap_or_default();
        if name.is_empty() {
            return;
        }
        self.log_viewer.set_pod_name(name);
        self.log_viewer.set_namespace(namespace);
        self.log_viewer.set_container(None);
        self.log_scroll = 0;
        let containers = self.fetch_containers().await;
        if containers.len() > 1 {
            self.container_selector.set_items(containers);
            self.popup = Popup::ContainerSelector;
            return;
        }
        if let Some(only) = containers.into_iter().next() {
            self.log_viewer.set_container(Some(strip_init_suffix(&only).to_string()));
        }
        self.popup = Popup::LogViewer;
        self.load_logs().await;
    }

    /// 从日志查看器内重新打开容器选择器（多容器时切换容器）
    async fn open_container_selector(&mut self) {
        let containers = self.fetch_containers().await;
        if containers.len() <= 1 {
            self.set_error("该 Pod 只有一个容器".to_string());
            return;
        }
        self.container_selector.set_items(containers);
        self.popup = Popup::ContainerSelector;
    }

    /// 拉取当前日志查看器目标 Pod 的容器列表（失败返回空列表）
    async fn fetch_containers(&mut self) -> Vec<String> {
        let Some(client) = &self.k8s_client else {
            return Vec::new();
        };
        let pod_name = self.log_viewer.pod_name.clone();
        let namespace = self.log_viewer.namespace.clone();
        match client.get_pod_containers(&pod_name, &namespace).await {
            Ok(containers) => containers,
            Err(e) => {
                self.set_error(friendly_error(&e));
                Vec::new()
            }
        }
    }

    async fn show_detail(&mut self) {
        let (name, namespace) = self.selected_resource()
            .map(|res| (res.name.clone(), res.namespace.clone()))
            .unwrap_or_default();
        if name.is_empty() {
            return;
        }
        self.popup = Popup::Detail;
        self.detail_scroll = 0;
        let res = crate::k8s::K8sResource {
            name,
            namespace,
            status: String::new(),
            age: String::new(),
            extra: vec![],
        };
        self.load_resource_detail(&res).await;
    }

    fn next_tab(&mut self) {
        let next = (self.current_tab.index() + 1) % ResourceTab::count();
        if let Some(tab) = ResourceTab::from_index(next) {
            self.switch_tab(tab);
        }
    }

    fn switch_tab(&mut self, tab: ResourceTab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            self.table_state.select(Some(0));
            self.search_query.clear();
            self.filtered_resources.clear();
        }
    }

    pub(crate) fn next_item(&mut self) {
        let len = self.filtered_resources.len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) if i >= len - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub(crate) fn previous_item(&mut self) {
        let len = self.filtered_resources.len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(0) => len - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// 切换 K8s Context
    async fn switch_context(&mut self, context: String) {
        let Some(client) = &mut self.k8s_client else { return };
        match client.switch_context(&context).await {
            Ok(_) => {
                self.current_context = context;
                self.current_namespace = "default".to_string();
                self.refresh_resources().await;
                self.set_success("切换 Context 成功".to_string());
            }
            Err(e) => {
                self.set_error(friendly_error(&e));
            }
        }
    }

    /// 删除资源
    async fn delete_resource(&mut self, res: &K8sResource) {
        let Some(client) = &self.k8s_client else { return };

        let result = match self.current_tab {
            ResourceTab::Pod => client.delete_pod(&res.name, &res.namespace).await,
            ResourceTab::ConfigMap => client.delete_configmap(&res.name, &res.namespace).await,
            ResourceTab::Secret => client.delete_secret(&res.name, &res.namespace).await,
            ResourceTab::Deployment => client.delete_deployment(&res.name, &res.namespace).await,
            ResourceTab::Service => client.delete_service(&res.name, &res.namespace).await,
            ResourceTab::Node => {
                self.set_error("Node 资源不支持删除".to_string());
                return;
            }
        };

        match result {
            Ok(_) => {
                self.set_success("删除成功".to_string());
                self.refresh_resources().await;
            }
            Err(e) => {
                self.set_error(friendly_error(&e));
            }
        }
    }

    /// 加载资源详情
    async fn load_resource_detail(&mut self, res: &K8sResource) {
        let Some(client) = &self.k8s_client else {
            self.detail_popup.set_content("未连接到 K8s 集群".to_string());
            return;
        };

        let detail = match self.current_tab {
            ResourceTab::Pod => client.get_pod_yaml(&res.name, &res.namespace).await,
            ResourceTab::ConfigMap => client.get_configmap_yaml(&res.name, &res.namespace).await,
            ResourceTab::Secret => client.get_secret_yaml(&res.name, &res.namespace).await,
            ResourceTab::Deployment => client.get_deployment_yaml(&res.name, &res.namespace).await,
            ResourceTab::Service => client.get_service_yaml(&res.name, &res.namespace).await,
            ResourceTab::Node => client.get_node_yaml(&res.name).await,
        };

        match detail {
            Ok(yaml) => self.detail_popup.set_content(yaml),
            Err(e) => self.detail_popup.set_content(friendly_error(&e)),
        }
    }

    /// 加载 Pod 日志
    async fn load_logs(&mut self) {
        let Some(client) = &self.k8s_client else {
            self.log_viewer.set_logs("未连接到 K8s 集群".to_string());
            return;
        };

        let pod_name = self.log_viewer.pod_name.clone();
        let namespace = self.log_viewer.namespace.clone();
        let container = self.log_viewer.container.clone();
        match client.get_pod_logs(&pod_name, &namespace, container.as_deref()).await {
            Ok(logs) => self.log_viewer.set_logs(logs),
            Err(e) => self.log_viewer.set_logs(friendly_error(&e)),
        }
    }

    // --- 消息管理 ---

    pub(crate) fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.success_message = None;
    }

    pub(crate) fn set_success(&mut self, msg: String) {
        self.success_message = Some(msg);
        self.error_message = None;
    }

    pub(crate) fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// 从配置文件加载设置
    #[allow(dead_code)]
    pub fn load_config(&mut self) {
        if let Ok(config) = crate::config::Config::load() {
            if let Some(ctx) = config.context {
                self.current_context = ctx;
            }
            if let Some(ns) = config.namespace {
                self.current_namespace = ns;
            }
            if let Some(tab_str) = config.resource_tab {
                match tab_str.as_str() {
                    "Pod" => self.current_tab = ResourceTab::Pod,
                    "ConfigMap" => self.current_tab = ResourceTab::ConfigMap,
                    "Secret" => self.current_tab = ResourceTab::Secret,
                    "Deployment" => self.current_tab = ResourceTab::Deployment,
                    "Service" => self.current_tab = ResourceTab::Service,
                    "Node" => self.current_tab = ResourceTab::Node,
                    _ => {}
                }
            }
            self.refresh_interval = Duration::from_secs(config.refresh_interval);
        }
    }

    /// 保存当前设置到配置文件
    #[allow(dead_code)]
    pub fn save_config(&self) -> anyhow::Result<()> {
        let config = crate::config::Config {
            context: Some(self.current_context.clone()),
            namespace: Some(self.current_namespace.clone()),
            resource_tab: Some(self.current_tab.as_str().to_string()),
            refresh_interval: self.refresh_interval.as_secs(),
            log_lines: 500,
            theme: "dark".to_string(),
        };
        config.save()
    }
}
