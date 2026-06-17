# k8s-tui 用户手册

## 目录

1. [安装](#安装)
2. [启动](#启动)
3. [界面介绍](#界面介绍)
4. [基本操作](#基本操作)
5. [高级功能](#高级功能)
6. [常见问题](#常见问题)

---

## 安装

### 环境要求

- Rust 1.70+
- 有效的 Kubernetes 集群访问权限
- `kubeconfig` 文件（默认路径 `~/.kube/config`）

### 从源码安装

```bash
git clone <repository-url>
cd k8s-tui
cargo build --release
```

编译完成后，二进制文件在 `target/release/k8s-tui`。

---

## 启动

```bash
./target/release/k8s-tui
```

启动后，工具会自动：
1. 读取 `~/.kube/config`
2. 使用当前 context 连接集群
3. 加载默认命名空间（`default`）的 Pod 列表

如果无法连接 K8s，界面会显示 "Context: 未连接"，但工具仍可启动。

---

## 界面介绍

```
┌─────────────────────────────────────────────────────────────┐
│  k8s-tui           │  Context: minikube  │  NS: default     │  ← 状态栏
├─────────────────────────────────────────────────────────────┤
│  [1:Pod]  [2:ConfigMap]  [3:Secret]  [4:Deployment]  [5:Service]  [6:Node]  │  12 resources  │  ← 标签栏
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  NAME              STATUS   RESTARTS   AGE        NODE      │
│  ────────────────────────────────────────────────────────   │  ← 资源列表
│ ▶ nginx-pod        Running  0          2h         node-1    │
│   redis-pod        Running  1          5h         node-2    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  ↑/↓: Navigate  Enter: Details  d: Delete  /: Search       │  ← 底部提示
│  l: Logs  c: Context  n: Namespace  q: Quit  ?: Help       │
└─────────────────────────────────────────────────────────────┘
```

### 状态栏

显示工具名称、当前 Context 和当前命名空间。

### 标签栏

显示六个资源类型标签：
- `1:Pod` — Pod 列表
- `2:ConfigMap` — ConfigMap 列表
- `3:Secret` — Secret 列表
- `4:Deployment` — Deployment 列表
- `5:Service` — Service 列表
- `6:Node` — Node 列表

### 资源列表

显示当前资源类型的所有资源。选中行会高亮显示。

### 底部提示栏

显示当前可用的快捷键。

---

## 基本操作

### 导航

| 按键 | 功能 |
|------|------|
| `↑` / `↓` | 上下移动选中行 |
| `j` / `k` | Vim 风格上下移动 |
| `Tab` | 切换到下一个资源标签 |
| `1` / `2` / `3` / `4` / `5` / `6` | 直接切换到 Pod / ConfigMap / Secret / Deployment / Service / Node |

### 查看资源详情

1. 使用 `↑` / `↓` 选中资源
2. 按 `Enter`
3. 详情弹窗会显示该资源的 YAML 内容
4. 按 `Esc` 关闭详情弹窗

在详情弹窗中：
- 按 `↑` / `↓` 滚动内容
- 按 `d` 删除该资源
- 按 `l` 查看 Pod 日志（仅 Pod）

### 删除资源

1. 选中要删除的资源
2. 按 `d`
3. 确认对话框会弹出
4. 按 `y` 确认删除，或按 `n` / `Esc` 取消

**注意**：删除操作不可恢复，请谨慎操作。Node 资源不支持删除。

### 切换命名空间

1. 按 `n`
2. 使用 `↑` / `↓` 选择命名空间
3. 按 `Enter` 确认

选择 `all` 可查看所有命名空间的资源。

### 切换 Context

1. 按 `c`
2. 使用 `↑` / `↓` 选择 Context
3. 按 `Enter` 确认

### 搜索过滤

1. 按 `/` 进入搜索模式
2. 输入资源名称关键字
3. 列表会实时过滤
4. 按 `Enter` 或 `Esc` 退出搜索模式

按 `Backspace` 可删除搜索字符。

### 手动刷新

按 `r` 手动刷新当前资源列表。

### 退出工具

按 `q` 退出 k8s-tui。

---

## 高级功能

### 查看 Pod 日志

1. 切换到 Pod 标签（`1`）
2. 选中目标 Pod
3. 按 `l`
4. 日志查看器会显示该 Pod 的日志

**注意**：日志功能仅支持 Pod 资源。

在日志查看器中：
- `↑` / `↓` — 逐行滚动
- `PgUp` / `PgDn` — 翻页
- `Esc` / `q` — 关闭日志查看器

### 实时刷新

资源列表默认每 2 秒自动刷新一次。刷新间隔可在配置中调整（后续版本支持）。

### 帮助页面

按 `?` 打开帮助页面，查看所有可用快捷键。

---

## 常见问题

### Q: 启动后显示 "Context: 未连接"

A: 请检查：
- `~/.kube/config` 是否存在且有效
- `kubectl get nodes` 是否能正常执行
- 当前 context 是否正确设置

### Q: 无法看到某些命名空间的资源

A: 请确认你的用户账号有权限访问该命名空间。k8s-tui 使用 kubeconfig 中的认证信息，权限与 `kubectl` 相同。

### Q: 删除资源失败

A: 请检查：
- 是否有该资源的删除权限
- 资源是否被其他对象引用（如被 Deployment 管理的 Pod）
- 错误信息会显示在屏幕底部，5 秒后自动消失
- 成功操作也会显示提示，同样自动消失

### Q: 日志查看器为空

A: 可能原因：
- Pod 尚未产生日志
- Pod 有多个容器，默认查看第一个容器的日志
- 没有查看日志的权限

### Q: 界面显示异常

A: 请确保你的终端支持 ANSI 转义序列。推荐终端：
- iTerm2（macOS）
- Windows Terminal（Windows）
- alacritty（跨平台）
- GNOME Terminal（Linux）

### Q: 如何调整刷新间隔？

A: 当前版本刷新间隔固定为 2 秒。后续版本将支持通过配置文件调整。

### Q: 如何在 Linux 服务器上运行？

A: 本项目支持在 macOS 上交叉编译 Linux 静态链接二进制文件：

```bash
# 安装交叉编译工具链（仅需一次）
brew install FiloSottile/musl-cross/musl-cross
rustup target add x86_64-unknown-linux-musl

# 编译
./build-linux.sh x86_64

# 产物：target/x86_64-unknown-linux-musl/release/k8s-tui
# 静态链接，可在任何 Linux 发行版直接运行
```

也支持编译 ARM64 版本（aarch64-unknown-linux-musl）。详见项目 README。

---

## 反馈

遇到问题或有功能建议？欢迎在 GitHub 提交 Issue！
