#!/usr/bin/env node

/**
 * 构建检测脚本
 * 检测 src-tauri-tools/src/ 目录下的文件变化
 * 仅当有更新或无构建产物时才编译
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const TOOLS_DIR = path.join(__dirname, '..', 'src-tauri-tools');
const SRC_DIR = path.join(TOOLS_DIR, 'src');
const FRONTEND_DIR = path.join(TOOLS_DIR, 'frontend');
const TARGET_DIR = path.join(TOOLS_DIR, 'target', 'release');
const EXE_NAME = 'tail-maker-external.exe';

// 获取目录下所有文件的最新修改时间
function getLatestModTime(dir) {
  let latest = 0;

  function walk(currentDir) {
    const entries = fs.readdirSync(currentDir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
      } else {
        const stat = fs.statSync(fullPath);
        if (stat.mtimeMs > latest) {
          latest = stat.mtimeMs;
        }
      }
    }
  }

  walk(dir);
  return latest;
}

// 检查构建产物是否存在
function buildArtifactExists() {
  const exePath = path.join(TARGET_DIR, EXE_NAME);
  return fs.existsSync(exePath);
}

// 获取构建产物的修改时间
function getBuildArtifactTime() {
  const exePath = path.join(TARGET_DIR, EXE_NAME);
  if (!fs.existsSync(exePath)) {
    return 0;
  }
  return fs.statSync(exePath).mtimeMs;
}

// 执行构建
function build() {
  console.log('🔨 检测到代码更新或无构建产物，开始编译小工具...');
  try {
    execSync('cargo build --release', {
      cwd: TOOLS_DIR,
      stdio: 'inherit',
    });
    console.log('✅ 小工具编译完成');
  } catch (e) {
    console.error('❌ 小工具编译失败:', e.message);
    process.exit(1);
  }
}

// 主逻辑
function main() {
  // 检查源目录是否存在
  if (!fs.existsSync(SRC_DIR)) {
    console.log('⚠️  src-tauri-tools/src 目录不存在，跳过构建');
    return;
  }

  // 获取源代码最新修改时间
  const srcModTime = getLatestModTime(SRC_DIR);
  const frontendModTime = fs.existsSync(FRONTEND_DIR) ? getLatestModTime(FRONTEND_DIR) : 0;
  const latestSrcModTime = Math.max(srcModTime, frontendModTime);

  // 检查构建产物
  if (!buildArtifactExists()) {
    console.log('📦 构建产物不存在，需要编译');
    build();
    return;
  }

  // 比较时间戳
  const buildTime = getBuildArtifactTime();
  if (latestSrcModTime > buildTime) {
    console.log('📝 检测到代码更新，需要重新编译');
    build();
  } else {
    console.log('✨ 小工具已是最新，跳过编译');
  }
}

main();
