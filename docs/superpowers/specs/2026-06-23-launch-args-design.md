# 启动参数功能设计

## 概述

为毫秒时钟应用添加两个可选启动参数：`--text` 和 `--center`，允许用户自定义显示文字和窗口位置。

## 参数规格

| 参数 | 格式 | 示例 | 默认行为 |
|------|------|------|----------|
| `--text` | `--text=内容` | `--text=北京时间` | 不显示文字 |
| `--center` | `--center=x,y` | `--center=500,300` | 窗口居中 |

- 两个参数均可选，不传则保持现状
- `--text` 值包含空格时需用引号包裹：`--text="北京时间 UTC+8"`
- `--center` 的 x,y 为窗口中心点的屏幕逻辑坐标（像素），允许负值（多显示器副屏场景）
- 参数格式错误时静默忽略，使用默认行为
- `--center` 不做坐标范围校验，由用户负责传入合理值

## 架构：Rust 端解析

参数在 Rust 端解析，不引入 `clap`，手动解析 `std::env::args()`。理由：
- 只有 2 个简单参数，不值得加依赖
- 位置设置在 Rust 端可避免窗口闪烁
- 启动参数解析是原生层的职责

## 实现细节

### 1. 参数解析（`lib.rs`）

在 `setup` 闭包中解析 `std::env::args()`：

```rust
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

### 2. 窗口创建与位置设置（`lib.rs`）

**关键设计**：使用 `WebviewWindowBuilder` 手动创建窗口，避免闪烁。

`tauri.conf.json` 中窗口配置添加 `"create": false`，阻止 Tauri 自动创建窗口。在 `setup` 闭包中手动构建窗口：

- **无 `--center` 参数**：使用 `WebviewWindowBuilder::from_config()` 创建窗口，调用 `.center()` 使其居中
- **有 `--center` 参数**：创建窗口时不调用 `.center()`，而是计算左上角坐标后用 `.position(x, y)` 设定初始位置

计算左上角坐标时需注意 DPI 缩放：

```rust
use tauri::{Manager, Emitter};
use tauri::LogicalPosition;

// 在 setup 中
let (text_value, center_value) = parse_args();

let mut builder = WebviewWindowBuilder::from_config(app, config)?;

if let Some((cx, cy)) = center_value {
    // 获取窗口逻辑尺寸（配置中的 1200×300 是逻辑像素）
    let logical_w = 1200.0;
    let logical_h = 300.0;
    let x = cx - logical_w / 2.0;
    let y = cy - logical_h / 2.0;
    builder = builder.position(x, y);
} else {
    builder = builder.center();
}

let window = builder.build()?;
```

这样窗口从第一帧就在正确位置，无闪烁。

### 3. 文字传递（Rust → 前端）

通过 Tauri 事件机制将 `--text` 值传给前端：

```rust
// 在 setup 中，窗口创建后
if let Some(text) = &text_value {
    window.emit("setup-args", serde_json::json!({ "text": text }))?;
}
```

### 4. 前端文字显示（`index.html`）

- 监听 `setup-args` 事件
- 收到 `text` 后在 `#clock` 上方插入 `#label` 元素
- `#label` 样式：
  - 字体：等宽字体 `Courier New, Consolas, monospace`
  - 字体大小：时钟字体大小的 60%
  - 颜色：`#000000`（和时钟一致）
  - 文字居中对齐
  - 带有 `data-tauri-drag-region`（可拖拽）
- 没有 `--text` 参数时不创建元素，布局不变

**label 空间竞争处理**：

`calcFontSize()` 当前使用 `h * 0.8` 控制时钟高度占比。当 label 存在时：

1. label 的实际像素高度 = 时钟字体大小 × 60% × 行高系数（约 1.2）
2. `calcFontSize()` 中扣减 label 高度：`byHeight = (h - labelHeight) * 0.8`
3. 由于 label 字体大小依赖时钟字体大小（60%），形成循环依赖，解决方案：
   - 先用当前 `calcFontSize()` 计算时钟字体大小
   - 用时钟字体大小 × 60% 设置 label 字体大小
   - 测量 label 实际高度（`label.offsetHeight`）
   - 扣减 label 高度后重新计算时钟字体大小
   - 实际上只需一次迭代即可收敛（因为 label 高度占比较小）

### 5. 配置变更（`tauri.conf.json`）

- 窗口配置添加 `"create": false`，阻止 Tauri 自动创建窗口
- 窗口创建完全由 Rust 端 `WebviewWindowBuilder` 控制

## 使用示例

```bash
# 默认启动（居中，无文字）
ms-clock.exe

# 显示"北京时间"
ms-clock.exe --text=北京时间

# 窗口中心在屏幕 (500, 300)
ms-clock.exe --center=500,300

# 同时使用两个参数
ms-clock.exe --text="北京时间" --center=500,300
```

## 错误处理

- `--center` 格式错误（非数字、缺少逗号等）：静默忽略，窗口默认居中
- `--text` 为空字符串：不显示文字
- 未知参数：忽略
