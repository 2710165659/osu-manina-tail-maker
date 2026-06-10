# run-dev.ps1 - 开发模式启动脚本
# 自动检测外部工具是否需要重新编译，然后启动 tauri dev

$ErrorActionPreference = "Stop"

# 路径配置 (脚本位于项目根目录)
$ProjectRoot = $PSScriptRoot
$TauriDir = Join-Path $ProjectRoot "tauri-tail-maker"
$ToolsDir = Join-Path $ProjectRoot "tauri-tail-maker-external"
$SharedDir = Join-Path $ProjectRoot "shared"
$ExePath = Join-Path $ToolsDir "target" "release" "tail-maker-external.exe"
$SrcDir = Join-Path $ToolsDir "src"
$FrontendDir = Join-Path $ToolsDir "frontend"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  osu!mania Tail Maker - 开发模式" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 检查源目录是否存在
if (-not (Test-Path $SrcDir)) {
    Write-Host "⚠️  外部工具源码目录不存在，跳过工具编译" -ForegroundColor Yellow
} else {
    $needBuild = $false

    # 检查构建产物是否存在
    if (-not (Test-Path $ExePath)) {
        Write-Host "📦 构建产物不存在，需要编译" -ForegroundColor Yellow
        $needBuild = $true
    } else {
        # 获取构建产物时间
        $buildTime = (Get-Item $ExePath).LastWriteTime

        # 获取所有源文件的最新修改时间
        $srcLatest = if (Test-Path $SrcDir) {
            (Get-ChildItem -Path $SrcDir, $SharedDir -Recurse -File -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                Select-Object -First 1).LastWriteTime
        } else { [datetime]::MinValue }

        $frontendLatest = if (Test-Path $FrontendDir) {
            (Get-ChildItem -Path $FrontendDir -Recurse -File -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                Select-Object -First 1).LastWriteTime
        } else { [datetime]::MinValue }

        $latestSrcTime = if ($srcLatest -gt $frontendLatest) { $srcLatest } else { $frontendLatest }

        if ($latestSrcTime -and ($latestSrcTime -gt $buildTime)) {
            Write-Host "📝 检测到代码更新，需要重新编译" -ForegroundColor Yellow
            $needBuild = $true
        } else {
            Write-Host "✨ 小工具已是最新，跳过编译" -ForegroundColor Green
        }
    }

    if ($needBuild) {
        Write-Host "🔨 开始编译小工具..." -ForegroundColor Cyan
        Push-Location $ProjectRoot
        try {
            cargo build --release -p tail-maker-external
            if ($LASTEXITCODE -ne 0) {
                throw "cargo build --release -p tail-maker-external 失败 (exit code: $LASTEXITCODE)"
            }
            Write-Host "✅ 小工具编译完成" -ForegroundColor Green
        } finally {
            Pop-Location
        }
    }
}

Write-Host ""
Write-Host "🚀 启动 Tauri 开发服务器..." -ForegroundColor Cyan
Push-Location $TauriDir
try {
    npm run tauri dev
} finally {
    Pop-Location
}
