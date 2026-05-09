# Cleaner - 垃圾清理工具

跨平台 TUI 垃圾清理工具，支持 Windows/Linux/macOS。

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

## 支持清理的包管理器

| 语言/工具 | 包管理器 | 目录 | 副作用 |
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

## 构建

```bash
make build          # 当前平台
make build-all      # 所有平台
make dist           # 打包到 dist/
```

## 代码质量

```bash
# 格式化
cargo fmt

# 检查
cargo check

# Clippy 静态分析
cargo clippy -- -D warnings

# 运行测试
cargo test
```

## Git 提交规范

每完成一个功能步骤后进行提交：

```bash
# 1. 检查状态
git status

# 2. 添加变更
git add .

# 3. 提交（遵循 Conventional Commits）
git commit -m "feat: 添加新功能"
git commit -m "fix: 修复问题"
git commit -m "refactor: 重构代码"
git commit -m "docs: 更新文档"
git commit -m "style: 格式化代码"
```

**提交类型**：
- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `style`: 格式化
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

## 依赖

- ratatui: TUI 框架
- crossterm: 终端控制
- walkdir: 目录遍历
- sysinfo: 系统信息
- anyhow: 错误处理
- dirs: 标准目录路径

## 工作流提醒

1. 每完成一个功能步骤后及时 `git add . && git commit`，不要攒多个功能一起提交
2. 提交信息遵循 Conventional Commits: `feat:`, `fix:`, `refactor:`, `docs:`, `style:`, `chore:`
3. 先格式化再提交: `cargo fmt && cargo check && git add . && git commit`
4. 完成 UI/交互改动后，运行一次确认能编译通过

## 快捷键

| 按键 | 功能 |
|------|------|
| ↑↓/jk | 移动光标 |
| gg | 跳转到第一条（快速连按两次 g） |
| G | 跳转到最后一条（快速连按两次 Shift+G） |
| Space | 选择/取消选择（支持目录级递归） |
| Enter | 折叠/展开目录 |
| D | 进入删除确认界面（需先选择文件） |
| A | 全选/取消全选（切换） |
| Q | 退出 |
| R | 重新扫描 |
| Esc | 取消删除确认 / 关闭错误提示 |

**删除确认界面**：按 `Enter` 或 `D` 确认删除，按 `Esc` 取消

