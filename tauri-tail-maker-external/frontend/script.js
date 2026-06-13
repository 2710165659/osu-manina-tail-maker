/**
 * osu!mania Tail Maker External Tool - Script
 */
const { invoke } = window.__TAURI__.core;

// ============================================
// State
// ============================================
const state = {
  mode: 'lazer',
  presets: [],
  keys: [],
  keydInfos: [],       // { stem, as_key: [], as_keyd: [] }
  imageKeyInfos: [],   // { stem, image_path, used_by: [{keys, columns}] }
  skinThrowInfos: [],  // { keys, current_throw, lazer_throw, valid, ... }
  keydChecked: new Set(),
  stemPresets: {},
  throwMap: {},         // keys → target throw
  skinRoot: null,
  initialized: false,
};

// ============================================
// DOM
// ============================================
const $ = (id) => document.getElementById(id);

// ============================================
// Log
// ============================================
function addLog(message, type = "info") {
  const logContent = $("log-content");
  const empty = logContent.querySelector(".log-empty");
  if (empty) empty.remove();
  const now = new Date();
  const time = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}:${now.getSeconds().toString().padStart(2, "0")}`;
  const line = document.createElement("div");
  line.className = `log-line ${type}`;
  line.innerHTML = `<span class="log-time">${time}</span><span class="log-marker">›</span><span class="log-msg">${message}</span>`;
  logContent.appendChild(line);
  logContent.scrollTop = logContent.scrollHeight;
}

// ============================================
// Helpers
// ============================================
function getModeThrow(info) {
  return state.mode === 'lazer' ? info.lazer_throw : info.current_throw;
}

// ============================================
// Mode radios
// ============================================
function setupModeRadios() {
  document.querySelectorAll('#mode-radios .radio-card').forEach(card => {
    card.addEventListener('click', () => {
      document.querySelectorAll('#mode-radios .radio-card').forEach(c => c.classList.remove('active'));
      card.classList.add('active');
      state.mode = card.dataset.mode;
      renderKeydSection();
      renderThrowKeys();
    });
  });
}

// ============================================
// Key/KeyD section
// ============================================
function renderKeydSection() {
  const section = $('keyd-section');
  if (state.mode !== 'lazer' || state.keydInfos.length === 0) {
    section.classList.add('hidden');
    return;
  }
  section.classList.remove('hidden');

  const list = $('keyd-list');
  list.innerHTML = '';
  state.keydInfos.forEach(kd => {
    const row = document.createElement('div');
    row.className = `keyd-row${state.keydChecked.has(kd.stem) ? ' active' : ''}`;
    const tags = [];
    if (kd.as_key.length > 0) tags.push(`<span class="kd-tag kd-key">Key: ${kd.as_key.map(k => k + 'k').join(', ')}</span>`);
    if (kd.as_keyd.length > 0) tags.push(`<span class="kd-tag kd-keyd">KeyD: ${kd.as_keyd.map(k => k + 'k').join(', ')}</span>`);
    row.innerHTML = `
      <label class="kdr-check">
        <input type="checkbox" ${state.keydChecked.has(kd.stem) ? 'checked' : ''} />
        <span class="kdr-stem">${kd.stem}</span>
      </label>
      <div class="kdr-usage">${tags.join(' ')}</div>
    `;
    row.querySelector('input').addEventListener('change', (e) => {
      if (e.target.checked) state.keydChecked.add(kd.stem);
      else state.keydChecked.delete(kd.stem);
      row.classList.toggle('active', e.target.checked);
    });
    list.appendChild(row);
  });
}

// ============================================
// Preset section (by stem)
// ============================================
function renderPresetSection() {
  const section = $('preset-section');
  if (state.presets.length === 0 || state.imageKeyInfos.length === 0) {
    section.classList.add('hidden');
    return;
  }
  section.classList.remove('hidden');

  const list = $('preset-stem-list');
  list.innerHTML = '';
  state.imageKeyInfos.forEach(ik => {
    const row = document.createElement('div');
    row.className = 'preset-row';
    const usage = ik.used_by.map(u => `<span class="ps-usage-item">${u.keys}k (列${u.columns.join(',')})</span>`).join(' ');
    const preset = state.stemPresets[ik.stem];
    row.innerHTML = `
      <span class="psr-stem" title="${ik.image_path}">${ik.stem}</span>
      <div class="psr-usage">${usage}</div>
      <div class="psr-preset">
        ${preset ? `<div class="preset-selected" data-stem="${ik.stem}">
          <img src="asset://localhost/${encodeURIComponent(preset.image_path)}" class="preset-thumb" />
          <span class="preset-name-sm">${preset.name}</span>
          <button class="preset-clear" data-stem="${ik.stem}">×</button>
        </div>` : `<button class="preset-pick-btn" data-stem="${ik.stem}">选择预设</button>`}
      </div>
    `;
    // Click preset picker
    row.querySelector('.preset-pick-btn')?.addEventListener('click', () => openPresetPicker(ik.stem));
    row.querySelector('.preset-selected')?.addEventListener('click', () => openPresetPicker(ik.stem));
    row.querySelector('.preset-clear')?.addEventListener('click', (e) => {
      e.stopPropagation();
      delete state.stemPresets[ik.stem];
      renderPresetSection();
    });
    list.appendChild(row);
  });
}

// ============================================
// Preset picker (simple dropdown-like inline modal)
// ============================================
let currentPresetStem = null;

function openPresetPicker(stem) {
  currentPresetStem = stem;
  // Build grid overlay
  const existing = document.querySelector('.preset-picker-overlay');
  if (existing) existing.remove();

  const overlay = document.createElement('div');
  overlay.className = 'preset-picker-overlay';
  const grid = document.createElement('div');
  grid.className = 'preset-picker-grid';
  state.presets.forEach(p => {
    const card = document.createElement('div');
    card.className = `preset-picker-card${state.stemPresets[stem]?.name === p.name ? ' active' : ''}`;
    card.innerHTML = `
      <div class="preset-picker-img-wrap"><img src="asset://localhost/${encodeURIComponent(p.image_path)}" class="preset-picker-img" /></div>
      <span class="preset-picker-label">${p.name}</span>
    `;
    card.addEventListener('click', () => {
      state.stemPresets[stem] = p;
      addLog(`${stem} 选择预设: ${p.name}`, 'info');
      overlay.remove();
      renderPresetSection();
    });
    grid.appendChild(card);
  });
  overlay.appendChild(grid);
  overlay.addEventListener('mousedown', (e) => { if (e.target === overlay) overlay.remove(); });
  document.body.appendChild(overlay);
}

// ============================================
// Throw keys section
// ============================================
function renderThrowKeys() {
  const list = $('keys-throw-list');
  if (state.keys.length === 0) {
    list.innerHTML = '<span style="color: var(--text-muted); font-size: 12px;">未找到键数配置</span>';
    return;
  }

  list.innerHTML = '';
  state.keys.forEach(key => {
    const info = state.skinThrowInfos.find(s => s.keys === key);
    if (!info) return;
    const row = document.createElement('div');
    row.className = `key-row${state.throwMap[key] !== undefined ? ' active' : ''}${!info.valid ? ' invalid' : ''}`;
    const checked = state.throwMap[key] !== undefined;
    const val = state.throwMap[key] ?? getModeThrow(info);
    row.innerHTML = `
      <label class="kr-check">
        <input type="checkbox" ${checked ? 'checked' : ''} ${!info.valid ? 'disabled' : ''} />
        <span class="kr-keys">${key}k</span>
      </label>
      <div class="kr-current">
        ${!info.valid ? '<span class="badge-invalid">不合规</span>' : `<span>${getModeThrow(info)}px</span>`}
      </div>
      <div class="kr-target">
        ${checked && info.valid ? `<div class="target-input-wrap"><input type="number" class="target-input" value="${val}" min="1" /><span class="target-suffix">px</span></div>` : '<span class="kr-na">-</span>'}
      </div>
    `;
    const cb = row.querySelector('input[type=checkbox]');
    cb.addEventListener('change', () => {
      if (cb.checked) {
        state.throwMap[key] = getModeThrow(info);
      } else {
        delete state.throwMap[key];
      }
      renderThrowKeys();
    });
    const input = row.querySelector('.target-input');
    if (input) {
      input.addEventListener('input', () => {
        const v = parseInt(input.value, 10);
        if (v > 0) state.throwMap[key] = v;
      });
    }
    list.appendChild(row);
  });
  updateConvertBtn();
}

// ============================================
// Convert
// ============================================
function updateConvertBtn() {
  const btn = $('convert-btn');
  btn.disabled = !(state.skinRoot && Object.keys(state.throwMap).length > 0);
}

async function handleConvert() {
  if ($('convert-btn').disabled) return;

  addLog('开始转换任务...', 'info');
  addLog(`模式: ${state.mode}`, 'info');

  const throws = Object.entries(state.throwMap).map(([k, v]) => [parseInt(k), v]);
  const presets = Object.entries(state.stemPresets).filter(([, v]) => v != null).map(([stem, v]) => [stem, v.name]);
  const keydStems = [...state.keydChecked];

  try {
    const result = await invoke('convert_tail', {
      config: {
        skin_root: state.skinRoot,
        mode: state.mode,
        throws,
        presets,
        keyd_stems: keydStems,
      },
    });
    for (const line of result.logs) {
      const type = line.startsWith('  ✓') ? 'success' : line.includes('⚠') || line.startsWith('  ✗') ? 'warning' : 'info';
      addLog(line, type);
    }
    addLog(result.message, result.success ? 'success' : 'error');
  } catch (e) {
    addLog(`转换失败: ${e}`, 'error');
  }
}

// ============================================
// Init
// ============================================
async function init() {
  addLog('正在初始化...', 'info');
  setupModeRadios();
  $('convert-btn').addEventListener('click', handleConvert);

  // GitHub link
  const githubLink = $('github-link');
  if (githubLink) {
    githubLink.addEventListener('click', () => {
      invoke('open_url', { url: 'https://github.com/2710165659/osu-manina-tail-maker' });
    });
  }

  try {
    // 1. Find skin root
    const skinResult = await invoke('find_skin_root');
    if (!skinResult.success) {
      addLog(skinResult.message, 'error');
      addLog('请将程序放在皮肤目录附近', 'warning');
      $('convert-btn').disabled = true;
      return;
    }
    state.skinRoot = skinResult.path;
    addLog(`找到皮肤目录: ${state.skinRoot}`, 'success');

    // 2. Find keys
    const keyResult = await invoke('find_keys', { skinRoot: state.skinRoot });
    if (keyResult.success) {
      state.keys = keyResult.keys;
      addLog(keyResult.message, 'info');
    } else {
      addLog(keyResult.message, 'error');
    }

    // 3. Load throw info
    addLog('正在读取皮肤信息...', 'info');
    state.skinThrowInfos = await invoke('get_skin_throw_info', { skinRoot: state.skinRoot });
    addLog('皮肤信息读取完成', 'success');

    // Lazy compute lazer throws if in lazer mode
    if (state.mode === 'lazer') {
      const validStems = state.skinThrowInfos.filter(s => s.valid).map(s => s.stem);
      if (validStems.length > 0) {
        try {
          addLog('正在计算 Lazer 投长度...', 'info');
          const lazerResults = await invoke('compute_lazer_throws', { skinRoot: state.skinRoot, stems: validStems });
          for (const [stem, lt] of lazerResults) {
            const s = state.skinThrowInfos.find(i => i.stem === stem);
            if (s) s.lazer_throw = lt;
          }
          addLog('Lazer 投长度计算完成', 'success');
        } catch (e) {
          addLog(`Lazer 投长度计算失败: ${e}`, 'warning');
        }
      }
    }

    // 4. Load image-key info
    try {
      state.imageKeyInfos = await invoke('get_image_key_info', { skinRoot: state.skinRoot });
    } catch (_) {}
    // 5. Load Key/KeyD info
    try {
      state.keydInfos = await invoke('get_keyd_list', { skinRoot: state.skinRoot });
      state.keydInfos.forEach(kd => state.keydChecked.add(kd.stem));
    } catch (_) {}
    // 6. Load presets
    addLog('正在加载预设...', 'info');
    try {
      state.presets = await invoke('load_presets');
      addLog(`已加载 ${state.presets.length} 个预设`, 'info');
    } catch (_) {}

    // Render sections
    renderKeydSection();
    renderPresetSection();
    renderThrowKeys();

    state.initialized = true;
    updateConvertBtn();
  } catch (e) {
    addLog(`初始化失败: ${e}`, 'error');
    $('convert-btn').disabled = true;
  }
}

document.addEventListener('DOMContentLoaded', init);
