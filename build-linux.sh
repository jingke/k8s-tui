#!/bin/bash
# 交叉编译脚本：从 macOS 编译 Linux 可执行文件

set -e

echo "=== k8s-tui Linux 交叉编译 ==="
echo ""

# 支持的架构
ARCH="${1:-x86_64}"

if [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-musl"
    echo "目标架构: x86_64 (Intel/AMD 64位)"
elif [ "$ARCH" = "aarch64" ]; then
    TARGET="aarch64-unknown-linux-musl"
    echo "目标架构: aarch64 (ARM 64位)"
else
    echo "用法: $0 [x86_64|aarch64]"
    echo "默认: x86_64"
    exit 1
fi

echo "目标平台: $TARGET"
echo ""

# 检查工具链
if ! command -v x86_64-linux-musl-gcc &> /dev/null; then
    echo "错误: 未找到 musl-cross 工具链"
    echo "请安装: brew install FiloSottile/musl-cross/musl-cross"
    exit 1
fi

# 检查 Rust target
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "安装 Rust target: $TARGET"
    rustup target add "$TARGET"
fi

echo "开始编译..."
cargo build --release --target "$TARGET"

echo ""
echo "=== 编译完成 ==="
echo "输出文件: target/$TARGET/release/k8s-tui"
echo "文件信息:"
file "target/$TARGET/release/k8s-tui"
echo ""
echo "文件大小:"
ls -lh "target/$TARGET/release/k8s-tui"
echo ""
echo "依赖检查 (应为静态链接):"
if command -v ldd &> /dev/null; then
    ldd "target/$TARGET/release/k8s-tui" 2>&1 || echo "✓ 静态链接，无动态依赖"
else
    echo "ℹ ldd 不可用，跳过依赖检查"
fi
