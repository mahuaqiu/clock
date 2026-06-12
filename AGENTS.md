# Repository Guidelines

## 项目概述

ms-clock 是一个基于 Tauri 2 + Rust 的毫秒时钟桌面应用，支持 macOS 和 Windows 双平台。窗口无边框、白色背景，时钟居中显示时:分:秒.毫秒，右上角有关闭按钮。

## 项目结构

```
ms-clock/
├── src/
│   └── index.html            # 前端界面（纯 HTML/CSS/JS，无构建工具）
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # 入口，调用 lib::run()
│   │   └── lib.rs            # Tauri 应用逻辑、命令定义
│   ├── capabilities/
│   │   └── default.json      # IPC 权限配置（Tauri 2 必须）
│   ├── icons/                # 各平台图标（icns/ico/png）
│   ├── Cargo.toml            # Rust 依赖
│   └── tauri.conf.json       # Tauri 核心配置
└── package.json              # npm 依赖（@tauri-apps/cli, @tauri-apps/api）
```

## 构建与开发命令

- `npx tauri dev` — 启动开发模式（热重载）
- `npx tauri build` — 构建生产包（macOS 生成 .app，Windows 生成 .exe/.msi）
- `cd src-tauri && cargo build --release` — 仅编译 Rust 二进制，不打包

## 编码规范

- Rust：遵循标准 `cargo fmt` 格式化，`cargo clippy` 检查
- 前端：纯 HTML/CSS/JS，无框架，无构建工具
- 注释使用中文
- `tauri.conf.json` 中 `withGlobalTauri` 必须放在 `app` 下面，不是 `build` 下面

## 关键注意事项

- **IPC 权限**：Tauri 2 默认拒绝所有前端调用，新增命令必须在 `capabilities/default.json` 中添加对应权限
- **全局 Tauri 对象**：`withGlobalTauri: true` 启用后，前端通过 `window.__TAURI__.core.invoke()` 调用 Rust 命令
- **Windows 子系统**：`main.rs` 中 `windows_subsystem = "windows"` 确保 Windows 上不弹出控制台窗口
- **图标生成**：使用 `npx tauri icon <源图>` 从一张 PNG 生成全部平台图标

## 提交规范

使用中文描述，格式：`<类型>: <描述>`，类型包括 `feat`、`fix`、`chore` 等。
