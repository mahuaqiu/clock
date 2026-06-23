# 启动参数功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为毫秒时钟应用添加两个可选启动参数 `--text` 和 `--center`，允许用户自定义显示文字和窗口位置。

**Architecture:** 参数在 Rust 端解析，使用 WebviewWindowBuilder 手动创建窗口避免闪烁，文字通过 Tauri 事件传给前端显示。

**Tech Stack:** Tauri 2, Rust, HTML/CSS/JS

---

## 文件结构

- Modify: `ms-clock/src-tauri/tauri.conf.json` - 添加 "create": false
- Modify: `ms-clock/src-tauri/src/lib.rs` - 参数解析、窗口创建、文字传递
- Modify: `ms-clock/src/index.html` - 监听事件、显示 label、调整字体计算

---

## 实现计划

### Task 1: 修改 tauri.conf.json 配置

**Files:**
- Modify: `ms-clock/src-tauri/tauri.conf.json:14-24`

- [ ] **Step 1: 添加 "create": false 配置**

打开 `ms-clock/src-tauri/tauri.conf.json`，找到 `windows` 配置数组中的窗口配置对象，在 `center: true` 后面添加 `"create": false`：

```json
"windows": [
  {
    "title": "毫秒时钟",
    "width": 1200,
    "height": 300,
    "resizable": true,
    "center": true,
    "decorations": false,
    "transparent": false,
    "create": false
  }
],
```

- [ ] **Step 2: 验证配置 JSON 格式正确**

运行: `Get-Content ms-clock/src-tauri/tauri.conf.json | ConvertFrom-Json`
Expected: 正常解析无错误

- [ ] **Step 3: 提交更改**

```bash
git add ms-clock/src-tauri/tauri.conf.json
git commit -m "config: 添加 create:false 阻止自动创建窗口"
```

---

### Task 2: 修改 lib.rs 实现参数解析和窗口创建

**Files:**
- Modify: `ms-clock/src-tauri/src/lib.rs`

- [ ] **Step 1: 添加参数解析函数**

在 `lib.rs` 文件顶部 `use tauri::Manager;` 之后，添加：

```rust
// 解析命令行参数
fn parse_args() -> (Option<String>, Option<(f64, f64)>) {
    let args: Vec<String> = std::env::args().collect();
    let mut text = None;
    let mut center = None;

    for arg in &args {
        if let Some(value) = arg.strip_prefix("--text=") {
            text = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--center=") {
            let parts: Vec<&str> = value.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    center = Some((x, y));
                }
            }
        }
    }

    (text, center)
}
```

- [ ] **Step 2: 修改 run() 函数实现手动窗口创建**

将当前的 `run()` 函数替换为：

```rust
use tauri::{Manager, Emitter, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 先解析参数（在窗口创建之前）
    let (text_value, center_value) = parse_args();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![quit_app])
        .setup(move |app| {
            // 从配置获取窗口信息
            let config = app.config().clone();
            let windows_config = config.app.windows.as_ref();
            
            if let Some(windows) = windows_config {
                if let Some(window_config) = windows.first() {
                    // 使用 WebviewWindowBuilder 手动创建窗口
                    let mut builder = WebviewWindowBuilder::from_config(app, window_config);
                    
                    if let Some((cx, cy)) = center_value {
                        // 窗口逻辑尺寸（从配置获取）
                        let logical_w = window_config.width as f64;
                        let logical_h = window_config.height as f64;
                        let x = cx - logical_w / 2.0;
                        let y = cy - logical_h / 2.0;
                        builder = builder.position(x, y);
                    } else {
                        builder = builder.center();
                    }
                    
                    let window = builder.build().expect("创建窗口失败");
                    window.set_title("毫秒时钟").unwrap();
                    
                    // 发送 text 参数给前端
                    if let Some(text) = &text_value {
                        let _ = window.emit("setup-args", serde_json::json!({ "text": text }));
                    }
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 尝试编译检查语法错误**

运行: `cd ms-clock/src-tauri && cargo check 2>&1`
Expected: 无编译错误（可能有 warnings 但不影响）

- [ ] **Step 4: 提交更改**

```bash
git add ms-clock/src-tauri/src/lib.rs
git commit -m "feat: 添加 --text 和 --center 参数解析和窗口创建逻辑"
```

---

### Task 3: 修改 index.html 实现文字显示

**Files:**
- Modify: `ms-clock/src/index.html`

- [ ] **Step 1: 添加 label 样式**

在 `<style>` 标签中 `#close-btn` 样式之后，添加 label 样式：

```css
/* 顶部文字 label */
#label {
    position: absolute;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    font-family: 'Courier New', 'Consolas', monospace;
    color: #000000;
    text-align: center;
    white-space: nowrap;
    cursor: default;
}
```

- [ ] **Step 2: 添加 label HTML 元素**

在 `<body>` 中 `#clock` 元素之后，添加 `#label` 元素：

```html
<div id="label" data-tauri-drag-region></div>
```

- [ ] **Step 3: 修改 JavaScript 实现**

在 `<script>` 标签中：

1. 在 `const closeBtn = ...` 之后添加 `const labelEl = document.getElementById('label');`

2. 修改 `calcFontSize()` 函数，考虑 label 高度：

```javascript
function calcFontSize() {
    const w = window.innerWidth;
    const h = window.innerHeight;
    let labelHeight = 0;
    
    // 如果 label 可见，计算其高度
    if (labelEl && labelEl.style.display !== 'none') {
        labelHeight = labelEl.offsetHeight + 20; // 20px margin
    }
    
    const byWidth = w / (12 * 0.62);
    const byHeight = (h - labelHeight) * 0.8;
    return Math.floor(Math.min(byWidth, byHeight));
}
```

3. 修改 `tick()` 函数，在时间更新后更新 label 字体大小：

```javascript
function tick() {
    const now = new Date();
    const h = String(now.getHours()).padStart(2, '0');
    const m = String(now.getMinutes()).padStart(2, '0');
    const s = String(now.getSeconds()).padStart(2, '0');
    const ms = String(now.getMilliseconds()).padStart(3, '0');
    clockEl.textContent = h + ':' + m + ':' + s + '.' + ms;
    clockEl.style.fontSize = calcFontSize() + 'px';
    
    // 更新 label 字体大小（60% 时钟字体大小）
    if (labelEl && labelEl.textContent) {
        labelEl.style.fontSize = (calcFontSize() * 0.6) + 'px';
    }
    
    requestAnimationFrame(tick);
}
```

4. 在 `tick();` 之前添加事件监听：

```javascript
// 监听 Tauri 事件接收 setup-args
if (window.__TAURI__) {
    window.__TAURI__.event.listen('setup-args', function(event) {
        const payload = event.payload;
        if (payload.text) {
            labelEl.textContent = payload.text;
            labelEl.style.display = 'block';
            // 触发重新计算字体大小
            clockEl.style.fontSize = calcFontSize() + 'px';
            labelEl.style.fontSize = (calcFontSize() * 0.6) + 'px';
        }
    });
}
```

- [ ] **Step 4: 验证 HTML 语法**

使用任意方式检查 HTML 文件语法正确性

- [ ] **Step 5: 提交更改**

```bash
git add ms-clock/src/index.html
git commit -m "feat: 添加前端 label 显示和事件监听逻辑"
```

---

### Task 4: 测试完整功能

**Files:**
- Test: 启动应用测试各种参数组合

- [ ] **Step 1: 测试默认启动（无参数）**

运行: `cd ms-clock && cargo tauri dev`
Expected: 窗口居中显示，无 label

- [ ] **Step 2: 测试 --text 参数**

运行: `cd ms-clock && cargo tauri dev -- --text=北京时间`
Expected: 窗口居中显示，上方显示"北京时间"文字

- [ ] **Step 3: 测试 --center 参数**

运行: `cd ms-clock && cargo tauri dev -- --center=100,100`
Expected: 窗口中心在屏幕 (100, 100) 位置，无 label

- [ ] **Step 4: 测试两个参数同时使用**

运行: `cd ms-clock && cargo tauri dev -- --text="北京时间" --center=100,100`
Expected: 窗口中心在 (100, 100)，上方显示"北京时间"

- [ ] **Step 5: 测试参数格式错误**

运行: `cd ms-clock && cargo tauri dev -- --center=abc`
Expected: 静默忽略，窗口默认居中

- [ ] **Step 6: 提交测试完成**

```bash
git add .
git commit -m "test: 验证启动参数功能正常工作"
```

---

## 总结

本计划包含 4 个任务，共 16 个步骤。完成所有步骤后，毫秒时钟应用将支持：

1. `--text=xxx` 参数：在时钟上方显示自定义文字
2. `--center=x,y` 参数：自定义窗口打开位置
3. 两个参数均可选，不传则保持现状
4. 参数格式错误时静默忽略

**预期最终提交：** 包含完整功能的 4-5 个 commit