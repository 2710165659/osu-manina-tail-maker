/**
 * osu!mania Tail Maker External Tool
 * Matches OneClickLength.vue functionality (vanilla JS)
 */

// Detect Tauri invoke API — try multiple known paths
let invoke;
(function () {
  // __TAURI_INTERNALS__ is the low-level IPC bridge (always present)
  const ti = window.__TAURI_INTERNALS__;
  if (ti && typeof ti.invoke === 'function') {
    invoke = function (cmd, args) {
      return ti.invoke(cmd, args);
    };
    console.log('[tail-maker] invoke via __TAURI_INTERNALS__.invoke');
    return;
  }

  const t = window.__TAURI__;

  // Log for debugging
  if (t) {
    console.log('[tail-maker] window.__TAURI__ keys:', Object.keys(t).join(', '));
  } else {
    console.log('[tail-maker] window.__TAURI__ is undefined');
  }
  if (ti) {
    console.log('[tail-maker] window.__TAURI_INTERNALS__ keys:', Object.keys(ti).join(', '));
  } else {
    console.log('[tail-maker] window.__TAURI_INTERNALS__ is undefined');
  }

  if (!t) return;

  // Try known paths for invoke
  if (typeof t.invoke === 'function') {
    invoke = t.invoke.bind(t);
  } else if (t.tauri && typeof t.tauri.invoke === 'function') {
    invoke = t.tauri.invoke.bind(t.tauri);
  } else if (t.core && typeof t.core.invoke === 'function') {
    invoke = t.core.invoke.bind(t.core);
  } else {
    for (const key of Object.keys(t)) {
      try {
        const sub = t[key];
        if (sub && typeof sub === 'object' && typeof sub.invoke === 'function') {
          invoke = sub.invoke.bind(sub);
          break;
        }
      } catch (_) {}
    }
  }
  if (invoke) {
    console.log('[tail-maker] invoke found via window.__TAURI__');
  } else {
    console.log('[tail-maker] invoke NOT FOUND');
  }
})();

// ============================================
// Helpers
// ============================================
const $ = (id) => document.getElementById(id);

function presetSrc(path) {
  if (path.startsWith('data:')) return path;
  // Tauri v2: use asset protocol
  return `asset://localhost/${encodeURIComponent(path)}`;
}

function getModeThrow(info) {
  if (state.workMode === 'lazer') {
    return info.lazer_throw > 0 ? info.lazer_throw : '…';
  }
  return info.current_throw;
}

function show(el) { if (el) el.classList.remove('hidden'); }
function hide(el) { if (el) el.classList.add('hidden'); }

// ============================================
// State
// ============================================
const state = {
  filePath: '',
  workMode: 'lazer',
  skinInfo: [],
  throwMap: new Map(),
  loadingInfo: false,
  computingThrows: false,
  keydInfos: [],
  keydChecked: new Set(),
  imageKeyInfos: [],
  presets: [],
  stemPresets: {},
  presetDialogStem: null,
  modifying: false,
};

const throwCache = new Map(); // stem → Promise<number>

// ============================================
// Log
// ============================================
function addLog(message, type = 'info') {
  const logContent = $('log-content');
  const empty = logContent.querySelector('.log-empty');
  if (empty) empty.remove();
  const now = new Date();
  const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
  const line = document.createElement('div');
  line.className = `log-line ${type}`;
  line.innerHTML = `<span class="log-time">${time}</span><span class="log-marker">›</span><span class="log-msg">${message}</span>`;
  logContent.appendChild(line);
  logContent.scrollTop = logContent.scrollHeight;
}

// ============================================
// Mode radios
// ============================================
function setupModeRadios() {
  document.querySelectorAll('#mode-radios .radio-card').forEach(card => {
    card.addEventListener('click', () => {
      document.querySelectorAll('#mode-radios .radio-card').forEach(c => c.classList.remove('active'));
      card.classList.add('active');
      state.workMode = card.dataset.mode;
      addLog(`切换模式: ${state.workMode === 'lazer' ? 'Lazer' : 'Stable'}`, 'info');

      // Recompute throwMap defaults when mode changes
      if (state.skinInfo.length > 0) {
        for (const [k] of state.throwMap) {
          const s = state.skinInfo.find(i => i.keys === k);
          if (s && s.valid) {
            const def = getModeThrow(s);
            state.throwMap.set(k, typeof def === 'number' ? def : s.current_throw);
          }
        }
        // 切到 Lazer 时若投长度未计算则触发计算
        if (state.workMode === 'lazer') {
          const needCompute = state.skinInfo.some(s => s.valid && s.lazer_throw === 0);
          if (needCompute) computeAllThrows();
        }
      }

      renderKeydSection();
      renderThrowSection();
    });
  });
}

// ============================================
// Browse button
// ============================================
async function handleBrowse() {
  try {
    const selected = await invoke('browse_folder');
    if (selected) {
      state.filePath = selected;
      addLog(`已选择：${state.filePath}`, 'info');
      updatePathDisplay();
      await loadAll();
    }
  } catch (e) {
    addLog(`文件选择失败：${e}`, 'error');
  }
}

function updatePathDisplay() {
  const el = $('path-text');
  if (state.filePath) {
    el.textContent = state.filePath;
    el.classList.remove('placeholder');
  } else {
    el.textContent = '请选择皮肤所在文件夹';
    el.classList.add('placeholder');
  }
}

// ============================================
// canModify
// ============================================
function canModify() {
  if (!state.filePath || state.modifying) return false;
  // Key/KeyD 修复：至少勾选一项
  if (state.keydChecked.size > 0) return true;
  // 预设替换：至少一项分配了预设
  if (Object.values(state.stemPresets).filter(Boolean).length > 0) return true;
  // 修改投长度：至少一项有效
  for (const v of state.throwMap.values()) { if (v && v >= 1) return true; }
  return false;
}

function updateModifyBtn() {
  const btn = $('modify-btn');
  const span = $('modify-btn-text');
  if (state.modifying) {
    btn.disabled = true;
    span.textContent = '修改中...';
  } else {
    btn.disabled = !canModify();
    span.textContent = '开始修改';
  }
}

// ============================================
// Key/KeyD section
// ============================================
function toggleKeyd(stem) {
  if (state.keydChecked.has(stem)) state.keydChecked.delete(stem);
  else state.keydChecked.add(stem);
}

function renderKeydSection() {
  const section = $('keyd-section');
  const loading = $('keyd-loading');
  const empty = $('keyd-empty');
  const noPath = $('keyd-no-path');
  const scroll = $('keyd-scroll');
  const grid = $('keyd-grid');
  const label = $('keyd-label');
  const countHint = $('keyd-count-hint');
  const hintBottom = $('keyd-hint-bottom');

  // Hide everything first
  hide(loading); hide(empty); hide(noPath); hide(scroll);
  hide(countHint); hide(hintBottom);

  if (state.workMode !== 'lazer') {
    hide(section);
    return;
  }
  show(section);

  if (!state.filePath) {
    show(noPath);
    return;
  }

  if (state.loadingInfo) {
    show(loading);
    return;
  }

  if (state.keydInfos.length === 0) {
    show(empty);
    return;
  }

  // Show grid
  const checked = state.keydChecked.size;
  label.innerHTML = `Key/KeyD 修复<span> (${checked}/${state.keydInfos.length})</span>`;
  show(countHint);
  countHint.textContent = `共 ${state.keydInfos.length} 张 Key/KeyD 图片`;
  show(scroll);
  show(hintBottom);

  grid.innerHTML = '';
  state.keydInfos.forEach(kd => {
    const item = document.createElement('label');
    item.className = `repair-item${state.keydChecked.has(kd.stem) ? ' active' : ''}`;

    const tags = [];
    if (kd.as_key.length > 0) tags.push('<span class="ri-tag ri-key">Key</span>');
    if (kd.as_keyd.length > 0) tags.push('<span class="ri-tag ri-keyd">KeyD</span>');

    item.innerHTML = `
      <input type="checkbox" ${state.keydChecked.has(kd.stem) ? 'checked' : ''} />
      <span class="ri-stem">${kd.stem}</span>
      ${tags.join('')}
    `;

    item.querySelector('input').addEventListener('change', (e) => {
      if (e.target.checked) state.keydChecked.add(kd.stem);
      else state.keydChecked.delete(kd.stem);
      item.classList.toggle('active', e.target.checked);
      label.innerHTML = `Key/KeyD 修复<span> (${state.keydChecked.size}/${state.keydInfos.length})</span>`;
      updateModifyBtn();
    });

    grid.appendChild(item);
  });
}

// ============================================
// Preset section
// ============================================
function renderPresetSection() {
  const section = $('preset-section');

  // 无预设图片时隐藏整个区域
  if (state.presets.length === 0) {
    hide(section);
    return;
  }

  const loading = $('preset-loading');
  const empty = $('preset-empty');
  const noPath = $('preset-no-path');
  const scroll = $('preset-scroll');
  const table = $('preset-table');
  const label = $('preset-label');
  const countHint = $('preset-count-hint');
  const hintBottom = $('preset-hint-bottom');

  hide(loading); hide(empty); hide(noPath); hide(scroll);
  hide(countHint); hide(hintBottom);

  if (!state.filePath) {
    show(section);
    show(noPath);
    return;
  }

  if (state.loadingInfo) {
    show(section);
    show(loading);
    return;
  }

  if (state.imageKeyInfos.length === 0) {
    show(section);
    show(empty);
    return;
  }

  show(section);

  const presetCount = Object.values(state.stemPresets).filter(Boolean).length;
  label.innerHTML = `预设替换<span> (${presetCount}/${state.imageKeyInfos.length})</span>`;
  show(countHint);
  countHint.textContent = `共 ${state.imageKeyInfos.length} 张面尾图片可替换`;
  show(scroll);
  show(hintBottom);

  table.innerHTML = '';
  state.imageKeyInfos.forEach(ik => {
    const row = document.createElement('div');
    row.className = 'preset-row';

    const usage = ik.used_by.map(u =>
      `<span class="ps-usage-item">${u.keys}k (列${u.columns.join(',')})</span>`
    ).join('');

    const preset = state.stemPresets[ik.stem];
    row.innerHTML = `
      <span class="psr-stem" title="${ik.image_path}">${ik.stem}</span>
      <div class="psr-usage">${usage}</div>
      <div class="psr-preset">
        ${preset ? `
          <div class="preset-selected" data-stem="${ik.stem}">
            <img src="${presetSrc(preset.image_path)}" class="preset-thumb" />
            <span class="preset-name-sm">${preset.name}</span>
            <button class="preset-clear" data-stem="${ik.stem}">×</button>
          </div>` : `
          <button class="preset-pick-btn" data-stem="${ik.stem}">选择预设</button>`}
      </div>
    `;

    // Events
    const pickBtn = row.querySelector('.preset-pick-btn');
    const selected = row.querySelector('.preset-selected');
    const clearBtn = row.querySelector('.preset-clear');

    if (pickBtn) pickBtn.addEventListener('click', () => openPresetDialog(ik.stem));
    if (selected) selected.addEventListener('click', () => openPresetDialog(ik.stem));
    if (clearBtn) {
      clearBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        state.stemPresets[ik.stem] = null;
        renderPresetSection();
      });
    }

    table.appendChild(row);
  });
}

// ============================================
// Preset modal
// ============================================
function openPresetDialog(stem) {
  state.presetDialogStem = stem;
  $('preset-modal-title').textContent = `选择预设 - ${stem}`;
  const grid = $('preset-modal-grid');
  grid.innerHTML = '';

  state.presets.forEach(p => {
    const card = document.createElement('div');
    card.className = `preset-card${state.stemPresets[stem]?.name === p.name ? ' active' : ''}`;
    card.innerHTML = `
      <div class="preset-img-wrap">
        <img src="${presetSrc(p.image_path)}" class="preset-img" />
      </div>
      <span class="preset-label">${p.name}</span>
    `;
    card.addEventListener('click', () => selectPreset(stem, p));
    grid.appendChild(card);
  });

  show($('preset-modal-overlay'));
}

function selectPreset(stem, preset) {
  state.stemPresets[stem] = preset;
  state.presetDialogStem = null;
  hide($('preset-modal-overlay'));
  addLog(`${stem} 选择预设: ${preset.name}`, 'info');
  renderPresetSection();
}

function closePresetDialog() {
  state.presetDialogStem = null;
  hide($('preset-modal-overlay'));
}

// ============================================
// Throw section
// ============================================
function toggleKey(k) {
  const info = state.skinInfo.find(s => s.keys === k);
  if (!info || !info.valid) return;
  if (state.throwMap.has(k)) {
    state.throwMap.delete(k);
  } else {
    const def = getModeThrow(info);
    state.throwMap.set(k, typeof def === 'number' ? def : info.current_throw);
  }
}

function getUniqueKeyInfos() {
  const seen = new Set();
  return state.skinInfo
    .filter(s => { if (seen.has(s.keys)) return false; seen.add(s.keys); return true; })
    .sort((a, b) => a.keys - b.keys);
}

function renderThrowSection() {
  const loading = $('throw-loading');
  const empty = $('throw-empty');
  const noPath = $('throw-no-path');
  const scroll = $('throw-scroll');
  const grid = $('throw-grid');
  const label = $('throw-label');
  const countHint = $('throw-count-hint');
  const computingHint = $('throw-computing');
  const hintBottom = $('throw-hint-bottom');

  hide(loading); hide(empty); hide(noPath); hide(scroll);
  hide(countHint); hide(computingHint); hide(hintBottom);

  if (!state.filePath) {
    show(noPath);
    return;
  }

  if (state.loadingInfo) {
    show(loading);
    return;
  }

  if (state.skinInfo.length === 0) {
    show(empty);
    return;
  }

  const unique = getUniqueKeyInfos();
  if (state.computingThrows) show(computingHint);

  label.innerHTML = `修改投长度<span> (${state.throwMap.size}/${unique.length})</span>`;
  show(countHint);
  countHint.textContent = `共 ${unique.length} 个键数`;
  show(scroll);
  show(hintBottom);

  grid.innerHTML = '';
  unique.forEach(info => {
    const card = document.createElement('label');
    card.className = `throw-card${state.throwMap.has(info.keys) ? ' active' : ''}${!info.valid ? ' invalid' : ''}`;

    const checked = state.throwMap.has(info.keys);
    const val = state.throwMap.get(info.keys) ?? '';
    const origText = info.valid ? `原: ${getModeThrow(info)}` : '不合规';

    card.innerHTML = `
      <input type="checkbox" ${checked ? 'checked' : ''} ${!info.valid ? 'disabled' : ''} />
      <span class="tc-keys">${info.keys}k</span>
      <input type="number" class="tc-input" value="${val}" ${!checked || !info.valid ? 'disabled' : ''} placeholder="-" min="1" />
      <span class="tc-orig">${origText}</span>
    `;

    // Checkbox change
    const cb = card.querySelector('input[type="checkbox"]');
    cb.addEventListener('change', () => {
      if (cb.checked) {
        const def = getModeThrow(info);
        state.throwMap.set(info.keys, typeof def === 'number' ? def : info.current_throw);
      } else {
        state.throwMap.delete(info.keys);
      }
      renderThrowSection();
      updateModifyBtn();
    });

    // Number input
    const numInput = card.querySelector('input[type="number"]');
    numInput.addEventListener('input', () => {
      const v = Number(numInput.value);
      if (v > 0) state.throwMap.set(info.keys, v);
      updateModifyBtn();
    });

    grid.appendChild(card);
  });

  updateModifyBtn();
}

// ============================================
// Load all data
// ============================================
async function loadAll() {
  state.loadingInfo = true;
  state.throwMap.clear();
  throwCache.clear();
  Object.keys(state.stemPresets).forEach(k => delete state.stemPresets[k]);
  state.keydChecked.clear();
  state.keydInfos = [];
  state.imageKeyInfos = [];
  state.presets = [];
  state.skinInfo = [];

  // Phase 1: Key/KeyD
  await loadKeydList();
  // Phase 2: Presets
  await loadPresetList();
  // Phase 3: Throw info + computation
  await loadThrowInfo();

  state.loadingInfo = false;

  renderKeydSection();
  renderPresetSection();
  renderThrowSection();
  updateModifyBtn();
}

async function loadKeydList() {
  if (state.workMode !== 'lazer') return;
  addLog('=== 检测 Key、KeyD ===', 'info');
  try {
    const kd = await invoke('get_keyd_list', { skinRoot: state.filePath });
    state.keydInfos = kd;
    addLog(`已加载 ${kd.length} 个 Key/KeyD 图片`, 'success');
  } catch (e) { addLog(`Key/KeyD 列表加载失败: ${e}`, 'warning'); state.keydInfos = []; }
}

async function loadPresetList() {
  addLog('=== 加载预设 ===', 'info');
  try {
    const ik = await invoke('get_image_key_info', { skinRoot: state.filePath });
    state.imageKeyInfos = ik;
    addLog(`已加载 ${ik.length} 个图片关联`, 'info');
  } catch (e) { addLog(`图片关联加载失败: ${e}`, 'warning'); state.imageKeyInfos = []; }

  try {
    const p = await invoke('load_presets');
    state.presets = p;
    if (p.length > 0) addLog(`已加载 ${p.length} 个预设`, 'success');
    else addLog('未找到预设图片', 'info');
  } catch (e) { addLog(`预设加载失败: ${e}`, 'warning'); }
}

async function loadThrowInfo() {
  addLog('=== 计算投长度 ===', 'info');
  try {
    const info = await invoke('get_skin_throw_info', { skinRoot: state.filePath });
    state.skinInfo = info;
    addLog('皮肤信息读取完成', 'success');

    const keySet = new Set(info.map(s => s.keys));
    const keys = [...keySet].sort((a, b) => a - b);

    if (keys.length > 0) {
      addLog(`检测到键数: ${keys.map(k => k + 'k').join(', ')}`, 'info');
      for (const s of info.filter(i => !i.valid)) {
        addLog(`⚠ ${s.keys}k ${s.stem}: 高度 ${s.height}px，不满足 >5000，不可修改`, 'warning');
      }
    } else {
      addLog('未找到任何 NoteImage#L 面尾定义', 'warning');
    }

    await computeAllThrows();
  } catch (e) {
    addLog(`读取皮肤信息失败：${e}`, 'error');
  }
}

async function computeAllThrows() {
  if (state.workMode !== 'lazer') return;

  // Dedup by stem only
  const seenStems = new Set();
  const stems = [];

  for (const s of state.skinInfo) {
    if (!s.valid) continue;
    if (seenStems.has(s.stem)) continue;
    seenStems.add(s.stem);
    stems.push(s.stem);
  }

  if (stems.length === 0) {
    addLog('无需计算投长度', 'info');
    return;
  }

  state.computingThrows = true;
  renderThrowSection();

  try {
    // Batch compute all stems at once
    addLog(`正在计算 ${stems.length} 个 stem 的 Lazer 投长度...`, 'info');
    const results = await invoke('compute_lazer_throws', { skinRoot: state.filePath, stems });

    for (const [stem, lt] of results) {
      for (const x of state.skinInfo) {
        if (x.stem === stem) x.lazer_throw = lt;
      }
      const keyList = [...new Set(state.skinInfo.filter(x => x.stem === stem).map(x => x.keys))]
        .sort((a, b) => a - b).map(k => k + 'k').join(', ');
      addLog(`  ✓ ${stem} (${keyList}) 投长度: ${lt}`, 'success');
    }
    addLog('投长度计算完成', 'success');

    // 同步已勾选的 throwMap
    for (const [k] of state.throwMap) {
      const s = state.skinInfo.find(i => i.keys === k);
      if (s && s.valid && s.lazer_throw > 0) {
        state.throwMap.set(k, s.lazer_throw);
      }
    }
  } catch (e) {
    addLog(`投长度计算失败: ${e}`, 'warning');
  }

  state.computingThrows = false;
  renderThrowSection();
}

// ============================================
// Modify
// ============================================
async function handleModify() {
  if (!canModify()) return;
  state.modifying = true;
  updateModifyBtn();

  addLog(`文件：${state.filePath}`, 'info');
  addLog(`开始修改... 模式: ${state.workMode}`, 'info');

  const entries = [...state.throwMap.entries()].sort((a, b) => a[0] - b[0]);
  const throws = entries.map(([k, v]) => [k, v]);

  const presetList = Object.entries(state.stemPresets)
    .filter(([, v]) => v !== null && v !== undefined)
    .map(([stem, v]) => [stem, v.name]);

  const keydStems = [...state.keydChecked];

  try {
    const result = await invoke('convert_tail', {
      config: {
        skin_root: state.filePath,
        mode: state.workMode,
        throws,
        presets: presetList,
        keyd_stems: keydStems,
      },
    });

    for (const line of result.logs) {
      const type = line.startsWith('  ✓') ? 'success'
        : line.includes('⚠') || line.startsWith('  ✗') ? 'warning'
        : 'info';
      addLog(line, type);
    }
    if (result.success) {
      addLog('修改完成！', 'success');
      // 重新加载投长度信息
      await loadThrowInfo();
      // 同步已勾选键数的 throwMap
      for (const [k] of state.throwMap) {
        const s = state.skinInfo.find(i => i.keys === k);
        if (s && s.valid) {
          const def = getModeThrow(s);
          state.throwMap.set(k, typeof def === 'number' ? def : s.current_throw);
        }
      }
      renderThrowSection();
    } else {
      addLog(`修改失败: ${result.message}`, 'error');
    }
  } catch (e) {
    addLog(`修改失败：${e}`, 'error');
  } finally {
    state.modifying = false;
    updateModifyBtn();
  }
}

// ============================================
// Init
// ============================================
async function init() {
  addLog('正在初始化...', 'info');
  if (typeof invoke !== 'function') {
    addLog('致命错误：Tauri invoke 不可用', 'error');
    addLog('window.__TAURI__ keys: ' + (window.__TAURI__ ? Object.keys(window.__TAURI__).join(', ') : 'undefined'), 'warning');
    addLog('window.__TAURI_INTERNALS__ keys: ' + (window.__TAURI_INTERNALS__ ? Object.keys(window.__TAURI_INTERNALS__).join(', ') : 'undefined'), 'warning');
    return;
  }

  // Setup event listeners
  setupModeRadios();
  $('browse-btn').addEventListener('click', handleBrowse);
  $('modify-btn').addEventListener('click', handleModify);
  $('preset-modal-close').addEventListener('click', closePresetDialog);
  $('preset-modal-overlay').addEventListener('mousedown', (e) => {
    if (e.target === $('preset-modal-overlay')) closePresetDialog();
  });

  // GitHub link
  const githubLink = $('github-link');
  if (githubLink) {
    githubLink.addEventListener('click', () => {
      invoke('open_url', { url: 'https://github.com/2710165659/osu-manina-tail-maker' });
    });
  }

  // Auto-find skin root
  try {
    const skinResult = await invoke('find_skin_root');
    if (skinResult.success) {
      state.filePath = skinResult.path;
      addLog(`找到皮肤目录: ${state.filePath}`, 'success');
      updatePathDisplay();
      await loadAll();
    } else {
      addLog(skinResult.message, 'warning');
      addLog('请点击"浏览"选择皮肤文件夹', 'info');
    }
  } catch (e) {
    addLog(`初始化失败: ${e}`, 'error');
    addLog('请点击"浏览"选择皮肤文件夹', 'info');
  }

  // Always render initial state (show placeholders)
  renderKeydSection();
  renderPresetSection();
  renderThrowSection();
  updateModifyBtn();
}

document.addEventListener('DOMContentLoaded', init);
