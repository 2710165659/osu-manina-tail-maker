<template>
  <div class="batch-generate">
    <div class="coming-soon">
      <!-- 装饰性背景 -->
      <div class="bg-pattern"></div>

      <div class="icon-stack">
        <svg width="56" height="56" viewBox="0 0 56 56" fill="none">
          <!-- 外层文档 -->
          <rect x="12" y="8" width="28" height="36" rx="3" stroke="currentColor" stroke-width="1.2" opacity="0.3"/>
          <!-- 内层文档 -->
          <rect x="16" y="12" width="28" height="36" rx="3" fill="var(--bg-panel)" stroke="currentColor" stroke-width="1.2"/>
          <!-- 文档内容线条 -->
          <path d="M22 22h16M22 27h16M22 32h10" stroke="currentColor" stroke-width="1" stroke-linecap="round" opacity="0.4"/>
          <!-- 齿轮装饰 -->
          <circle cx="42" cy="42" r="10" fill="var(--bg-surface)" stroke="var(--accent-purple)" stroke-width="1.5"/>
          <circle cx="42" cy="42" r="4" stroke="var(--accent-purple)" stroke-width="1.2"/>
          <!-- 齿轮齿 -->
          <path d="M42 30v2M42 50v2M30 42h2M50 42h2" stroke="var(--accent-purple)" stroke-width="2" stroke-linecap="round"/>
          <path d="M33.5 33.5l1.5 1.5M49 49l1.5 1.5M33.5 50.5l1.5-1.5M49 35l1.5-1.5" stroke="var(--accent-purple)" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </div>

      <div class="text-group">
        <h3 class="coming-title">批量生成工具</h3>
        <p class="coming-desc">支持批量处理多个皮肤文件，自动生成不同配置变体</p>
      </div>

      <div class="status-badge">
        <div class="badge-dot"></div>
        <span>开发中</span>
      </div>

      <!-- 功能预览列表 -->
      <div class="feature-preview">
        <div class="feature-item disabled">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="1" y="1" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.1"/>
          </svg>
          <span>批量导入 osk 文件</span>
        </div>
        <div class="feature-item disabled">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="1" y="1" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.1"/>
          </svg>
          <span>自定义输出配置</span>
        </div>
        <div class="feature-item disabled">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="1" y="1" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.1"/>
          </svg>
          <span>队列管理与进度</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.batch-generate {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.coming-soon {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  position: relative;
  padding: 20px;
}

.bg-pattern {
  position: absolute;
  inset: -40px;
  background:
    radial-gradient(circle at 30% 20%, rgba(183, 108, 241, 0.04) 0%, transparent 50%),
    radial-gradient(circle at 70% 80%, rgba(255, 102, 170, 0.03) 0%, transparent 50%);
  pointer-events: none;
}

.icon-stack {
  color: var(--text-muted);
  position: relative;
  animation: float 6s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

.text-group {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.coming-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: 0.3px;
}

.coming-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  max-width: 280px;
  line-height: 1.6;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: rgba(255, 170, 68, 0.08);
  border: 1px solid rgba(255, 170, 68, 0.2);
  border-radius: 20px;
  font-size: 11px;
  color: #ffaa44;
  font-weight: 500;
}

.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #ffaa44;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.feature-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  width: 100%;
  max-width: 260px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.feature-item.disabled {
  opacity: 0.4;
}

.feature-item svg {
  flex-shrink: 0;
  color: var(--text-muted);
}
</style>
