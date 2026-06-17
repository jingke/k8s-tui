# k8s-tui 架构设计文档

## 1. 概述

k8s-tui 是一个基于 Rust 的终端用户界面（TUI）工具，用于管理 Kubernetes 资源。本文档描述其整体架构、模块划分、数据流和关键技术决策。

## 2. 架构目标

- **低延迟**：启动时间 < 500ms，操作响应 < 100ms
- **高响应**：异步处理所有 K8s API 调用，不阻塞 UI
- **可维护性**：清晰的模块边界，便于扩展新资源类型
- **跨平台**：支持 Linux、macOS、Windows（WSL2 / 原生）

## 3. 模块结构

```
src/
├── main.rs           # 程序入口：初始化终端、K8s 客户端、事件循环
├── app.rs            # 应用状态管理（App 结构体、事件处理、业务逻辑）
├── app_tests.rs      # 单元测试和集成测试
├── config.rs         # 配置系统（TOML 持久化用户偏好）
├── event.rs          # 事件系统（键盘输入、定时刷新、窗口调整）
├── k8s.rs            # K8s API 客户端封装
├── ui.rs             # TUI 渲染主入口
└── components/       # 可复用 UI 组件
    ├── confirm_dialog.rs
    ├── context_selector.rs
    ├── help_popup.rs
    ├── log_viewer.rs
    ├── namespace_selector.rs
    ├── resource_detail.rs
    └── search_bar.rs
```

## 4. 核心组件

### 4.1 App（应用状态）

`App` 结构体是应用的核心状态容器，包含：

- **当前视图状态**：`current_tab`、`popup`、`search_mode`
- **资源数据**：`resources`、`filtered_resources`、`table_state`
- **K8s 连接**：`k8s_client`、`current_context`、`current_namespace`
- **UI 组件状态**：各个弹窗组件的实例
- **配置**：`Config` 结构体（TOML 持久化）

`App` 负责：
- 处理所有键盘事件
- 管理资源列表的刷新和过滤
- 协调弹窗的打开和关闭
- 调用 K8s 客户端执行实际操作

### 4.2 K8sClient（K8s 客户端封装）

`K8sClient` 封装了 `kube` crate，提供简化的 API：

- `list_pods(namespace)` — 列出 Pod
- `list_configmaps(namespace)` — 列出 ConfigMap
- `list_secrets(namespace)` — 列出 Secret
- `list_deployments(namespace)` — 列出 Deployment
- `list_services(namespace)` — 列出 Service
- `list_nodes()` — 列出 Node
- `list_namespaces()` — 列出命名空间
- `get_*_yaml(name, namespace)` — 获取资源 YAML
- `delete_*(name, namespace)` — 删除资源
- `get_pod_logs(name, namespace)` — 获取 Pod 日志
- `switch_context(context)` — 切换 K8s Context

### 4.3 EventHandler（事件处理器）

`EventHandler` 运行在独立的 tokio 任务中，负责：

- 每 250ms 发送一次 `Tick` 事件
- 监听键盘输入并发送 `Key` 事件
- 监听终端尺寸变化并发送 `Resize` 事件

通过 `tokio::sync::mpsc` 通道与应用主循环通信。

### 4.4 UI 渲染

`ui.rs` 是渲染主入口，根据 `App` 状态绘制：

- 顶部状态栏（工具名、Context、Namespace）
- 标签栏（Pod / ConfigMap / Secret / Deployment / Service / Node）
- 资源列表（使用 `ratatui::widgets::Table`）
- 底部快捷键提示
- 弹窗（详情、确认、选择器、帮助、日志）

## 5. 数据流

```
用户按键 ──▶ EventHandler ──▶ mpsc 通道 ──▶ run_app()
                                              │
                                              ▼
                                         App::handle_key_event()
                                              │
                                              ▼
                              ┌─────────────────────────────┐
                              │  更新 UI 状态 / 调用 K8sClient │
                              └─────────────────────────────┘
                                              │
                                              ▼
                                    terminal.draw(ui::draw)
                                              │
                                              ▼
                                           屏幕刷新
```

## 6. 关键技术决策

### 6.1 为什么选择 ratatui + crossterm？

- **ratatui** 是 Rust 生态中最成熟的 TUI 框架，提供丰富的组件和灵活的布局
- **crossterm** 提供跨平台终端控制，支持 Windows、macOS、Linux

### 6.2 为什么选择 kube + k8s-openapi？

- **kube** 是 Rust 官方推荐的 K8s 客户端，支持异步 API、Informer、Controller 等高级特性
- **k8s-openapi** 提供完整的 K8s API 类型定义，跟随官方 schema 更新

### 6.3 异步架构

- 使用 `tokio` 作为异步运行时
- K8s API 调用全部异步执行，避免阻塞 UI 渲染
- 事件循环通过通道解耦，保证主循环的响应性

### 6.4 状态管理

- 采用集中式状态管理，`App` 结构体持有所有状态
- 通过 `match self.popup` 实现弹窗状态机
- 搜索过滤在本地完成，不重新请求 API

## 7. 扩展指南

### 7.1 添加新的资源类型

1. 在 `ResourceTab` 枚举中添加新变体
2. 在 `ResourceTab` 的 `as_str()` 和 `index()` 方法中添加对应分支
3. 在 `K8sClient` 中添加 `list_*` 和 `delete_*` 方法（或使用泛型 `list_resources<T>()`）
4. 在 `App::refresh_resources` 中添加对应分支
5. 在 `App::delete_resource` 中添加对应分支
6. 在 `App::load_resource_detail` 中添加对应分支
7. 在 `ui.rs` 的 `draw_resource_list` 中添加列定义和单元格渲染
8. 在 `ui.rs` 的 `draw_tab_bar` 中添加标签
9. 在 `ui.rs` 的 `help_popup` 中更新快捷键提示
10. 在 `app.rs` 的 `handle_key_event` 中添加数字快捷键
11. 更新 `ResourceTab::count()` 和 `from_index()` 方法
12. 更新测试以覆盖新资源类型

### 7.2 添加新的弹窗

1. 在 `Popup` 枚举中添加新变体
2. 创建对应的组件文件（参考 `components/log_viewer.rs`）
3. 在 `App::handle_key_event` 中添加打开逻辑
4. 在 `ui.rs` 的 `draw` 函数中添加渲染分支

## 8. 性能考虑

- 资源列表使用 `Vec<K8sResource>` 存储，过滤时存储索引 `Vec<usize>` 而非克隆资源，减少内存分配
- 自动刷新间隔默认 2 秒，避免频繁请求 API Server
- 搜索过滤在本地完成，时间复杂度 O(n)
- 未来大集群场景可考虑实现虚拟滚动和分页

## 9. 依赖清单

核心依赖：
- `ratatui` 0.29 — TUI 框架
- `crossterm` 0.28 — 跨平台终端控制
- `kube` 0.98 — Kubernetes Rust 客户端
- `k8s-openapi` 0.24 — K8s API 类型定义
- `tokio` — 异步运行时
- `anyhow` — 错误处理
- `chrono` — 时间格式化
- `serde` + `serde_yaml` — 序列化
- `toml` — 配置解析
- `dirs` — 跨平台配置目录

## 10. 安全考虑

- Secret 详情查看时显示完整 YAML，但不在本地持久化
- 删除操作必须二次确认
- 日志查看只读，不可修改
- 所有 K8s 认证通过标准 kubeconfig 完成

## 11. 交叉编译

本项目支持从 macOS 交叉编译 Linux 静态链接二进制文件：

```bash
# 编译 x86_64 版本
./build-linux.sh x86_64

# 编译 ARM64 版本
./build-linux.sh aarch64
```

使用 `musl-cross` 工具链，产物静态链接，可在任何 Linux 发行版直接运行。
详见项目 README 和 `build-linux.sh` 脚本。
