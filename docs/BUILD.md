# PrintCraft 构建指南

## 环境准备

### macOS (开发机)

```bash
# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 Windows 交叉编译 target
rustup target add x86_64-pc-windows-msvc

# 安装 cargo-xwin (Windows 交叉编译)
cargo install cargo-xwin

# Node.js (SDK 构建)
brew install node
```

### 代理配置（网络受限时）

```bash
export https_proxy=http://127.0.0.1:7897
export http_proxy=http://127.0.0.1:7897
export all_proxy=socks5://127.0.0.1:7897
```

## 构建命令

### macOS 本地构建

```bash
# 构建 server + cli
cargo build --release -p printcraft-server -p printcraft-cli

# 产物位置
ls -lh target/release/printcraft-server
ls -lh target/release/printcraft
```

### Windows 交叉编译 (macOS → Windows)

```bash
# 交叉编译 server + cli
cargo xwin build --target x86_64-pc-windows-msvc -p printcraft-server -p printcraft-cli --release

# 产物位置
ls -lh target/x86_64-pc-windows-msvc/release/printcraft-server.exe
ls -lh target/x86_64-pc-windows-msvc/release/printcraft.exe
```

### Windows 原生编译 (Windows ARM64 / x64)

需要 Visual Studio 2026 Build Tools + C++ 桌面开发工作负载。

**推荐：用构建脚本（自动检测 MSVC/SDK 版本）：**

```powershell
# 在项目根目录执行
.\scripts\build-windows.ps1                # ARM64 release
.\scripts\build-windows.ps1 -Debug         # ARM64 debug
.\scripts\build-windows.ps1 -Target x86    # x86_64 release
```

**手动编译（脚本不可用时）：**

```powershell
# 设置 MSVC 和 SDK 路径（根据实际安装版本调整）
$MSVC = "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Tools\MSVC\14.52.36328"
$SDK = "C:\Program Files (x86)\Windows Kits\10\lib\10.0.26100.0"

# ARM64 编译
$env:LIB = "$MSVC\lib\arm64;$SDK\ucrt\arm64;$SDK\um\arm64"
$env:PATH = "$MSVC\bin\Hostarm64\arm64;$env:PATH"
cargo build --release -p printcraft-server

# x86_64 编译
$env:LIB = "$MSVC\lib\x64;$SDK\ucrt\x64;$SDK\um\x64"
$env:PATH = "$MSVC\bin\Hostx64\x64;$env:PATH"
cargo build --release --target x86_64-pc-windows-msvc -p printcraft-server
```

**Cargo 镜像（%USERPROFILE%\\.cargo\\config.toml）：**

```toml
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
```

### 构建 SDK

```bash
cd sdk
npm install
npm run build

# 产物位置
ls -lh dist/printcraft.js
```

### 一键构建全部

```bash
# macOS 二进制
cargo build --release -p printcraft-server -p printcraft-cli

# Windows 二进制
cargo xwin build --target x86_64-pc-windows-msvc -p printcraft-server -p printcraft-cli --release

# SDK
cd sdk && npm install && npm run build && cd ..

# 打包到桌面
mkdir -p ~/Desktop/printcraft-release/macos
mkdir -p ~/Desktop/printcraft-release/windows
mkdir -p ~/Desktop/printcraft-release/sdk

cp target/release/printcraft-server ~/Desktop/printcraft-release/macos/
cp target/release/printcraft ~/Desktop/printcraft-release/macos/

cp target/x86_64-pc-windows-msvc/release/printcraft-server.exe ~/Desktop/printcraft-release/windows/
cp target/x86_64-pc-windows-msvc/release/printcraft.exe ~/Desktop/printcraft-release/windows/

cp sdk/dist/printcraft.js ~/Desktop/printcraft-release/sdk/

echo "构建完成！产物在 ~/Desktop/printcraft-release/"
```

## 测试

```bash
# Rust 测试
cargo test --workspace --lib --tests

# SDK 测试
cd sdk && npm test
```

## 常见问题

### cargo-xwin 安装失败

```bash
# 解压 .gitmodules 权限问题，需要关闭 sandbox
cargo install cargo-xwin
```

### 交叉编译找不到 Windows SDK

cargo-xwin 会自动下载 MSVC CRT，首次运行会下载约 300MB，之后缓存。

### 网络问题

确保代理开启，或使用国内镜像：

```bash
# Rust 镜像 (~/.zshrc 或 ~/.bashrc)
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# Cargo 镜像 (~/.cargo/config.toml)
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse://rsproxy.cn/crates.io-index"
```

## 发布流程

```bash
# 1. 构建全部产物
# (执行上面的"一键构建全部")

# 2. 提交代码
git add .
git commit -m "release: v0.1.0"
git push

# 3. 打 tag 触发 GitHub Actions
git tag v0.1.0
git push origin v0.1.0

# GitHub Actions 会自动:
# - 在 windows-latest 编译 Windows 二进制
# - 在 macos-latest 编译 macOS 二进制
# - 构建 SDK
# - 创建 GitHub Release 并上传产物
```

## 产物清单

| 平台 | 文件 | 大小 |
|------|------|------|
| Windows x64 | printcraft-server.exe | ~6.4M |
| Windows x64 | printcraft.exe (CLI) | ~3.3M |
| macOS ARM64 | printcraft-server | ~5M |
| macOS ARM64 | printcraft (CLI) | ~2.5M |
| 跨平台 | sdk/printcraft.js | ~8KB (gzip) |
