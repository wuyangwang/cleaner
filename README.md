# Cleaner

跨平台 TUI 垃圾清理工具，支持 Windows/Linux/macOS。

## 功能特性

- 扫描并清理各包管理器缓存文件
- 树形目录展示，支持折叠展开
- 安全机制：拒绝扫描系统关键路径
- 跨平台支持：Windows、Linux、macOS

## 支持的包管理器

| 语言/工具 | 包管理器 | 清理目录 | 副作用 |
|------|----------|------|------|
| JavaScript | npm, pnpm, yarn, bun | ~/.npm, ~/.pnpm-store | 需重下 |
| Rust | cargo | ~/.cargo/registry/src, target | 需重编 |
| Go | go | ~/go/pkg/mod | 需重下 |
| Python | pip, poetry | ~/.cache/pip, pypoetry | 需重下 |
| Java | maven, gradle | ~/.m2, ~/.gradle | 需重下 |
| Deno | deno | ~/.deno | 需重下 |
| 浏览器 | Chrome | ~/.cache/google-chrome | 无 |
| 系统 | Logs, Trash | /var/log, ~/.Trash | 无 |
| 工具 | Homebrew, CocoaPods | ~/Library/Caches/... | 需重下 |
| 测试 | Playwright, Cypress | ~/.cache/... | 需重下 |

## 快捷键

| 按键 | 功能 |
|------|------|
| `↑↓` / `jk` | 移动光标 |
| `gg` | 跳转到第一条（快速连按两次 g） |
| `GG` | 跳转到最后一条（快速连按两次 G） |
| `Space` | 选择/取消选择（支持目录级递归） |
| `Enter` | 折叠/展开目录 |
| `D` | 进入删除确认界面（需先选择文件） |
| `A` | 全选/取消全选（切换） |
| `Q` | 退出 |
| `R` | 重新扫描 |
| `Esc` | 取消删除确认 / 关闭错误提示 |

**删除确认界面**：按 `Enter` 或 `D` 确认删除，按 `Esc` 取消

## 使用

```bash
# 构建
make build

# 运行
./target/release/cleaner

# 安装到本地
make install
```

## 命令

```bash
make build          # 当前平台构建
make build-all      # 所有平台构建
make check          # 代码检查 (clippy + fmt)
make test           # 运行测试
make dist           # 全平台打包
make release        # 发布版本（创建 tag 并推送，触发 GitHub Actions 构建）
```

## 项目结构

```
src/
├── main.rs      # 入口 + TUI 事件循环
├── app.rs       # 应用状态机
├── ui.rs        # ratatui 界面渲染
├── scanner.rs   # 文件扫描 + 安全检查
├── tree.rs      # 树结构（目录/文件）
├── targets.rs   # 清理目标目录配置
├── disk.rs      # 磁盘空间检测
└── error.rs     # 错误类型枚举
```

## 核心模块

### app.rs - 状态管理
- `AppState`: Scanning → Selecting → Confirming → Cleaning → Complete
- 管理文件列表、选择状态、清理进度

### scanner.rs - 文件扫描
- `scan_trash_dirs()`: 扫描各包管理器缓存目录
- `is_system_critical()`: 安全检查，禁止删除系统目录
- `TrashItem`: 文件条目（路径、大小、分类）

### tree.rs - 树结构
- `TreeNode`: 树节点（目录/文件），支持折叠展开
- `build_tree()`: 将扁平文件列表构建为目录树
- `flatten_tree()`: 将目录树扁平化为显示列表

### targets.rs - 清理目标配置
- `get_trash_directories()`: 获取各包管理器缓存目录列表
- `get_home_dir()`: 获取用户主目录

### disk.rs - 磁盘信息
- `get_disk_info()`: 获取指定路径的磁盘可用空间
- 使用 sysinfo 库跨平台获取

### error.rs - 错误处理
- `CleanError`: 枚举类型
  - `SystemPathForbidden`: 系统路径禁止删除
  - `NoFilesSelected`: 未选择文件
  - `FileNotFound`: 文件不存在
  - `PermissionDenied`: 权限不足

## 安全机制

1. **路径白名单**: 拒绝扫描 `/`, `/bin`, `/usr`, `C:\Windows` 等
2. **递归限制**: 最大扫描深度 5 层
3. **删除前验证**: 删除前再次检查路径安全性
4. **默认不选**: 文件默认不选中，需用户主动选择

## 依赖

- ratatui: TUI 框架
- crossterm: 终端控制
- walkdir: 目录遍历
- sysinfo: 系统信息
- anyhow: 错误处理
- dirs: 标准目录路径

## 构建要求

- Rust edition 2024
