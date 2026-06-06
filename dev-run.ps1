# Tauri 开发环境配置并启动
# 使用方法：.\dev-run.ps1

# 启动本地 HTTP 服务器提供 skia 预编译二进制文件（从项目目录）
Write-Host "启动本地文件服务器..." -ForegroundColor Yellow
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$httpJob = Start-Job -ScriptBlock {
    param($path)
    Set-Location "$path\.skia-binaries"
    python -m http.server 8000
} -ArgumentList $projectRoot
Start-Sleep -Seconds 2

$env:SKIA_BINARIES_URL="http://localhost:8000/skia-binaries-ec00cf219c4901d785ed-x86_64-pc-windows-msvc-textlayout.tar.gz"
Write-Host "✓ 环境变量已设置:" -ForegroundColor Green
Write-Host "  SKIA_BINARIES_URL = $env:SKIA_BINARIES_URL" -ForegroundColor Cyan
Write-Host ""

Write-Host "正在启动 Tauri 开发服务器..." -ForegroundColor Yellow
Write-Host ""

try {
    npm run tauri dev
} finally {
    Write-Host ""
    Write-Host "清理：停止本地文件服务器..." -ForegroundColor Yellow
    Stop-Job $httpJob
    Remove-Job $httpJob
}
