/**
 * osu!mania Tail Maker External Tool - Script
 */

const { invoke } = window.__TAURI__.core;

// ============================================
// State
// ============================================
const state = {
  presets: [],
  keys: [],
  selectedPreset: null,
  selectedKeys: new Set(),
  skinRoot: null,
  initialized: false,
};

// ============================================
// DOM Elements
// ============================================
const $ = (id) => document.getElementById(id);
const presetSection = $("preset-section");
const presetGrid = $("preset-grid");
const keyGrid = $("key-grid");
const throwLengthInput = $("throw-length");
const convertBtn = $("convert-btn");
const logContent = $("log-content");

// ============================================
// Log System
// ============================================
function addLog(message, type = "info") {
  const empty = logContent.querySelector(".log-empty");
  if (empty) empty.remove();

  const now = new Date();
  const time = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}:${now.getSeconds().toString().padStart(2, "0")}`;

  const line = document.createElement("div");
  line.className = `log-line ${type}`;
  line.innerHTML = `
    <span class="log-time">${time}</span>
    <span class="log-marker">›</span>
    <span class="log-msg">${message}</span>
  `;

  logContent.appendChild(line);
  logContent.scrollTop = logContent.scrollHeight;
}

// ============================================
// Preset Rendering
// ============================================
function renderPresets() {
  if (state.presets.length === 0) {
    presetSection.classList.add("hidden");
    return;
  }

  presetSection.classList.remove("hidden");
  presetGrid.innerHTML = "";

  state.presets.forEach((preset) => {
    const card = document.createElement("div");
    card.className = `preset-card${state.selectedPreset === preset.name ? " active" : ""}`;
    card.dataset.name = preset.name;
    card.innerHTML = `
      <div class="preset-radio">
        <div class="preset-radio-inner"></div>
      </div>
      <div class="preset-image-wrap">
        <img class="preset-image" src="asset://localhost/${encodeURIComponent(preset.image_path)}" alt="${preset.name}" loading="lazy" />
      </div>
      <span class="preset-name" title="${preset.name}">${preset.name}</span>
    `;

    card.addEventListener("click", () => {
      if (state.selectedPreset === preset.name) {
        state.selectedPreset = null;
      } else {
        state.selectedPreset = preset.name;
      }
      renderPresets();
    });

    presetGrid.appendChild(card);
  });
}

// ============================================
// Key Rendering
// ============================================
function renderKeys() {
  keyGrid.innerHTML = "";

  if (state.keys.length === 0) {
    keyGrid.innerHTML =
      '<span style="color: var(--text-muted); font-size: 12px;">未找到键数配置</span>';
    return;
  }

  state.keys.forEach((key) => {
    const label = `${key}k`;
    const card = document.createElement("div");
    card.className = `key-card${state.selectedKeys.has(key) ? " active" : ""}`;
    card.dataset.key = key;
    card.innerHTML = `
      <div class="checkbox-box">
        ${state.selectedKeys.has(key) ? '<svg width="10" height="10" viewBox="0 0 10 10" fill="none"><path d="M2 5l2.5 2.5L8 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>' : ""}
      </div>
      <span class="key-label">${label}</span>
    `;

    card.addEventListener("click", () => {
      if (state.selectedKeys.has(key)) {
        state.selectedKeys.delete(key);
      } else {
        state.selectedKeys.add(key);
      }
      renderKeys();
    });

    keyGrid.appendChild(card);
  });
}

// ============================================
// Update Convert Button State
// ============================================
function updateConvertBtn() {
  const hasSkin = state.skinRoot !== null;
  const hasKeys = state.selectedKeys.size > 0;
  const length = parseInt(throwLengthInput.value, 10);
  const hasLength = length > 0;

  convertBtn.disabled = !(hasSkin && hasKeys && hasLength);
}

// ============================================
// Convert Action
// ============================================
async function handleConvert() {
  if (convertBtn.disabled) return;

  const length = parseInt(throwLengthInput.value, 10);
  const selectedKeysArr = Array.from(state.selectedKeys).sort((a, b) => a - b);

  addLog("开始转换任务...", "info");

  if (state.selectedPreset) {
    addLog(`使用预设: ${state.selectedPreset}`, "info");
  }

  addLog(`目标键数: ${selectedKeysArr.map((k) => k + "k").join(", ")}`, "info");
  addLog(`投长度: ${length}px`, "info");

  try {
    const result = await invoke("convert_tail", {
      config: {
        skin_root: state.skinRoot,
        keys: selectedKeysArr,
        throw_length: length,
        preset: state.selectedPreset,
      },
    });

    if (result.success) {
      addLog(result.message, "success");
      result.processed_keys.forEach((key) => {
        addLog(`✓ ${key}k 转换完成`, "success");
      });
    } else {
      addLog(result.message, "error");
    }
  } catch (e) {
    addLog(`转换失败: ${e}`, "error");
  }
}

// ============================================
// Event Listeners
// ============================================
convertBtn.addEventListener("click", handleConvert);

throwLengthInput.addEventListener("input", updateConvertBtn);
throwLengthInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    handleConvert();
  }
});

// ============================================
// Find Skin Root
// ============================================
async function findSkinRoot() {
  try {
    const result = await invoke("find_skin_root");
    if (result.success) {
      return result.path;
    }
    addLog(result.message, "warning");
    return null;
  } catch (e) {
    addLog(`查找皮肤目录失败: ${e}`, "error");
    return null;
  }
}

// ============================================
// Init - 页面加载时自动初始化
// ============================================
async function init() {
  addLog("正在初始化...", "info");

  try {
    // 1. 寻找皮肤根目录
    const skinRoot = await findSkinRoot();

    if (!skinRoot) {
      addLog("未找到皮肤根目录（skin.ini）", "error");
      addLog("请将程序放在皮肤目录附近或手动选择", "warning");
      convertBtn.disabled = true;
      return;
    }

    state.skinRoot = skinRoot;
    addLog(`找到皮肤目录: ${skinRoot}`, "success");

    // 2. 查找键数
    const keyResult = await invoke("find_keys", { skinRoot });
    if (keyResult.success) {
      state.keys = keyResult.keys;
      renderKeys();
      addLog(keyResult.message, "info");
    } else {
      addLog(keyResult.message, "error");
    }

    // 3. 加载预设
    const presets = await invoke("load_presets", { skinRoot });
    state.presets = presets;
    renderPresets();
    if (presets.length > 0) {
      addLog(`已加载 ${presets.length} 个预设`, "info");
    }

    state.initialized = true;
    updateConvertBtn();
  } catch (e) {
    addLog(`初始化失败: ${e}`, "error");
    convertBtn.disabled = true;
  }
}

// 页面加载完成后初始化
document.addEventListener("DOMContentLoaded", init);
