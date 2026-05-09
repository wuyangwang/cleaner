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

- **app.rs**: 状态管理（Scanning → Selecting → Confirming → Cleaning → Complete）
- **scanner.rs**: 文件扫描 + 安全检查
- **tree.rs**: 树结构（目录/文件），支持折叠展开
- **targets.rs**: 清理目标目录配置
- **disk.rs**: 磁盘空间检测
- **error.rs**: 错误类型枚举

## 代码规范

### Git 提交规范

遵循 Conventional Commits：

```bash
git commit -m "feat: 添加新功能"
git commit -m "fix: 修复问题"
git commit -m "refactor: 重构代码"
git commit -m "docs: 更新文档"
git commit -m "style: 格式化代码"
git commit -m "test: 测试"
git commit -m "chore: 构建/工具"
```

### 代码质量

```bash
cargo fmt              # 格式化
cargo check            # 检查
cargo clippy -- -D warnings  # Clippy 静态分析
cargo test             # 运行测试
```

## 工作流提醒

1. 每完成一个功能步骤后及时 `git add . && git commit`，不要攒多个功能一起提交
2. 提交信息遵循 Conventional Commits: `feat:`, `fix:`, `refactor:`, `docs:`, `style:`, `chore:`
3. 先格式化再提交: `cargo fmt && cargo check && git add . && git commit`
4. 完成 UI/交互改动后，运行一次确认能编译通过
5. 发布版本: 手动运行 `make release`（创建 tag 并推送，触发 GitHub Actions 构建）

## 快捷键

| 按键 | 功能 |
|------|------|
| ↑↓/jk | 移动光标 |
| gg | 跳转到第一条（快速连按两次 g） |
| GG | 跳转到最后一条（快速连按两次 G） |
| Space | 选择/取消选择（支持目录级递归） |
| Enter | 折叠/展开目录 |
| D | 进入删除确认界面（需先选择文件） |
| A | 全选/取消全选（切换） |
| Q | 退出 |
| R | 重新扫描 |
| Esc | 取消删除确认 / 关闭错误提示 |

**删除确认界面**：按 `Enter` 或 `D` 确认删除，按 `Esc` 取消

## 依赖

- ratatui: TUI 框架
- crossterm: 终端控制
- walkdir: 目录遍历
- sysinfo: 系统信息
- anyhow: 错误处理
- dirs: 标准目录路径

## 构建

```bash
make build          # 当前平台
make build-all      # 所有平台
make dist           # 打包到 dist/
```
