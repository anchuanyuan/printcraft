# Windows 原生编译脚本 (VS 2026 Build Tools + ARM64)
#
# 用法:
#   .\scripts\build-windows.ps1              # release 编译 server
#   .\scripts\build-windows.ps1 -Debug       # debug 编译
#   .\scripts\build-windows.ps1 -Target x86  # 编译 x86_64 版本

param(
    [switch]$Debug,
    [string]$Target = "arm64",
    [string]$Package = "printcraft-server",
    [string]$Test = ""
)

$ErrorActionPreference = "Stop"

# 自动检测 MSVC 版本（取最新）
$msvcBase = "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Tools\MSVC"
if (Test-Path $msvcBase) {
    $msvcVer = (Get-ChildItem $msvcBase | Sort-Object Name -Descending | Select-Object -First 1).Name
    $MSVC = "$msvcBase\$msvcVer"
} else {
    Write-Error "未找到 VS 2026 Build Tools MSVC，请确认已安装 C++ 工作负载"
    exit 1
}

# 自动检测 Windows SDK 版本
$sdkBase = "C:\Program Files (x86)\Windows Kits\10\lib"
if (Test-Path $sdkBase) {
    $sdkVer = (Get-ChildItem $sdkBase | Sort-Object Name -Descending | Select-Object -First 1).Name
    $SDK = "$sdkBase\$sdkVer"
} else {
    Write-Error "未找到 Windows SDK"
    exit 1
}

# 根据目标架构设置环境
if ($Target -eq "arm64") {
    $rustTarget = "aarch64-pc-windows-msvc"
    $env:LIB = "$MSVC\lib\arm64;$SDK\ucrt\arm64;$SDK\um\arm64"
    $env:PATH = "$MSVC\bin\Hostarm64\arm64;$env:PATH"
} elseif ($Target -eq "x86") {
    $rustTarget = "x86_64-pc-windows-msvc"
    $env:LIB = "$MSVC\lib\x64;$SDK\ucrt\x64;$SDK\um\x64"
    $env:PATH = "$MSVC\bin\Hostx64\x64;$env:PATH"
} else {
    Write-Error "不支持的 Target: $Target，可选 arm64 或 x86"
    exit 1
}

$profile = if ($Debug) { "dev" } else { "release" }

# 先构建 SDK（JS 嵌入到 Rust 二进制中）
$sdkDir = Join-Path $PSScriptRoot "..\sdk"
if (Test-Path "$sdkDir\package.json") {
    Write-Host "=== 构建 SDK ===" -ForegroundColor Cyan
    Push-Location $sdkDir
    npm install --silent 2>&1 | Out-Null
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "SDK 构建失败" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host ""
}

Write-Host "=== PrintCraft Windows 编译 ===" -ForegroundColor Cyan
Write-Host "MSVC:     $MSVC"
Write-Host "SDK:      $SDK"
Write-Host "Target:   $rustTarget"
Write-Host "Profile:  $profile"
Write-Host "Package:  $Package"
Write-Host ""

$extraArgs = @()
if (-not $Debug) { $extraArgs += "--release" }

# 运行测试模式
if ($Test -ne "") {
    Write-Host "=== 运行测试 ===" -ForegroundColor Cyan
    Write-Host "Crate: $Test"
    Write-Host ""
    cargo test -p $Test --target $rustTarget -- --nocapture
    exit $LASTEXITCODE
}

cargo build --target $rustTarget -p $Package @extraArgs

if ($LASTEXITCODE -eq 0) {
    $ext = if ($Package -eq "printcraft-server") { "printcraft-server.exe" } else { "printcraft.exe" }
    $outPath = "target\$rustTarget\$profile\$ext"
    if (Test-Path $outPath) {
        $size = (Get-Item $outPath).Length / 1MB
        Write-Host ""
        Write-Host "编译成功! 产物: $outPath ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
} else {
    Write-Host ""
    Write-Host "编译失败" -ForegroundColor Red
    exit 1
}
