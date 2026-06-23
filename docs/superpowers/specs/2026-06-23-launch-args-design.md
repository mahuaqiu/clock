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
- `--center` 的 x,y 为窗口中心点的屏幕逻辑坐标（像素）
- 参数格式错误时静默忽略，使用默认行为

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

### 2. 文字传递（Rust → 前端）

通过 Tauri 事件机制将 `--text` 值传给前端：

```rust
// 在 setup 中
if let Some(text) = &text_value {
    window.emit("setup-args", serde_json::json!({ "text": text })).unwrap();
}
```

### 3. 窗口位置设置（`lib.rs`）

在 `setup` 闭包中，如果 `--center` 有效：

1. 获取窗口尺寸
2. 计算左上角坐标：`x = center_x - width/2`，`y = center_y - height/2`
3. 调用 `window.set_position(LogicalPosition::new(x, y))`

当 `--center` 有值时，需将 `tauri.conf.json` 中 `center` 设为 `false`（通过 Rust 端动态设置），避免 Tauri 先居中再移动导致闪烁。

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
- 时钟字体大小计算需考虑 label 占用的空间

### 5. 配置变更（`tauri.conf.json`）

- 当传入了 `--center` 参数时，通过 Rust 代码在运行时禁用 `center` 行为
- 配置文件中 `center` 保持 `true` 不变（默认行为）

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
