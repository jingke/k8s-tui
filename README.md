# k8s-tui

一个用 Rust 编写的轻量级 Kubernetes 资源管理 TUI（终端用户界面）工具。

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

## 功能特性

- **资源浏览**：列表展示 Pod、ConfigMap、Secret、Deployment、Service、Node
- **资源详情**：查看选中资源的 YAML 详情
- **资源删除**：支持删除操作，带二次确认
- **上下文切换**：快速切换 K8s Context（集群）
- **命名空间切换**：支持所有命名空间或指定命名空间
- **实时刷新**：资源列表每 2 秒自动刷新
- **日志查看**：查看 Pod 日志（只读，支持翻页）
- **搜索过滤**：按资源名称实时过滤
- **键盘驱动**：所有操作无需鼠标

## 截图

```
┌─────────────────────────────────────────────────────────────┐
│  k8s-tui           │  Context: minikube  │  NS: default     │
├─────────────────────────────────────────────────────────────┤
│  [1:Pod]  [2:ConfigMap]  [3:Secret]  [4:Deployment]  [5:Service]  [6:Node]  │  12 resources       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  NAME              STATUS   RESTARTS   AGE        NODE      │
│  ────────────────────────────────────────────────────────   │
│ ▶ nginx-pod        Running  0          2h         node-1    │
│   redis-pod        Running  1          5h         node-2    │
│   failed-pod       Error    3          1d         node-1    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  ↑/↓: Navigate  Enter: Details  d: Delete  /: Search       │
│  l: Logs  c: Context  n: Namespace  q: Quit  ?: Help       │
└─────────────────────────────────────────────────────────────┘
```

## 安装

### 环境要求

- Rust 1.70 或更高版本
- 有效的 `kubeconfig` 文件（通常位于 `~/.kube/config`）
- 可访问的 Kubernetes 集群

### 从源码编译

#### macOS / Linux 本地编译

```bash
git clone <repository-url>
cd k8s-tui
cargo build --release
```

编译完成后，二进制文件位于 `target/release/k8s-tui`。

#### macOS 交叉编译 Linux 可执行文件

在 macOS 上编译出可在 Linux 服务器运行的静态链接二进制文件：

```bash
# 1. 安装交叉编译工具链（仅需一次）
brew install FiloSottile/musl-cross/musl-cross

# 2. 添加 Rust target
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# 3. 编译 x86_64 版本（Intel/AMD 服务器）
./build-linux.sh x86_64

# 4. 编译 aarch64 版本（ARM 服务器，如 AWS Graviton）
./build-linux.sh aarch64
```

编译产物：
- `target/x86_64-unknown-linux-musl/release/k8s-tui` — x86_64 Linux
- `target/aarch64-unknown-linux-musl/release/k8s-tui` — ARM64 Linux

特点：
- **静态链接**：不依赖系统 glibc，可在任何 Linux 发行版运行
- **单文件分发**：无需安装依赖，直接复制到服务器运行

### 运行

```bash
./target/release/k8s-tui
```

## 快速开始

1. 确保 `kubectl` 可以正常连接到你的集群：
   ```bash
   kubectl get nodes
   ```

2. 启动 `k8s-tui`：
   ```bash
   ./target/release/k8s-tui
   ```

3. 使用键盘快捷键浏览和管理资源。

## 快捷键

| 按键 | 功能 |
|------|------|
| `↑` / `↓` 或 `j` / `k` | 移动选中行 |
| `Enter` | 查看资源详情 |
| `Tab` / `1` / `2` / `3` / `4` / `5` / `6` | 切换资源标签（Pod/ConfigMap/Secret/Deployment/Service/Node） |
| `d` | 删除选中资源（需确认） |
| `l` | 查看 Pod 日志 |
| `/` | 进入搜索模式 |
| `Esc` | 关闭弹窗 / 退出搜索 |
| `c` | 切换 K8s Context |
| `n` | 切换命名空间 |
| `r` | 手动刷新 |
| `q` | 退出工具 |
| `?` | 打开帮助页 |

## 技术栈

- [ratatui](https://github.com/ratatui/ratatui) — TUI 框架
- [crossterm](https://github.com/crossterm-rs/crossterm) — 跨平台终端控制
- [kube](https://github.com/kube-rs/kube) — Kubernetes Rust 客户端
- [k8s-openapi](https://github.com/Arnavion/k8s-openapi-rs) — K8s API 类型定义
- [tokio](https://tokio.rs/) — 异步运行时

## 开发

### 运行测试

```bash
cargo test
```

### 开发模式运行

```bash
cargo run
```

## 项目结构

```
k8s-tui/
├── src/
│   ├── main.rs           # 程序入口
│   ├── app.rs            # 应用状态管理
│   ├── app_tests.rs      # 单元测试和集成测试
│   ├── config.rs         # 配置系统（TOML 持久化）
│   ├── event.rs          # 事件处理（键盘 + 定时器）
│   ├── k8s.rs            # K8s API 客户端封装
│   ├── ui.rs             # TUI 渲染
│   └── components/       # 可复用 UI 组件
│       ├── confirm_dialog.rs
│       ├── context_selector.rs
│       ├── help_popup.rs
│       ├── log_viewer.rs
│       ├── namespace_selector.rs
│       ├── resource_detail.rs
│       └── search_bar.rs
├── Cargo.toml
├── build-linux.sh        # Linux 交叉编译脚本
└── README.md
```

## 路线图

- [x] Pod / ConfigMap / Secret / Deployment / Service / Node 浏览和删除
- [x] 资源详情查看
- [x] Context / Namespace 切换
- [x] 实时刷新
- [x] Pod 日志查看
- [x] 搜索过滤
- [x] 支持更多资源类型（Deployment、Service、Node）
- [ ] 资源编辑功能
- [ ] Pod exec / port-forward
- [ ] 自定义资源（CRD）支持
- [ ] 插件系统

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License
