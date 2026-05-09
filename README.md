# Cleaner

跨平台 TUI 垃圾清理工具。扫描并清理包管理器缓存文件，支持 Windows/Linux/macOS。

## 支持的包管理器

| 语言 | 包管理器 |
|------|----------|
| JavaScript | npm, pnpm, yarn, bun |
| Rust | cargo |
| Go | go |
| Python | pip, conda, poetry |
| Java | maven, gradle |
| .NET | nuget |
| Ruby | bundler |
| PHP | composer |
| Dart | pub |

## 快捷键

| 按键 | 功能 |
|------|------|
| `↑↓` / `jk` | 移动 |
| `Space` | 展开/折叠目录，选择/取消文件 |
| `A` | 全选 |
| `N` | 取消全选 |
| `Enter` | 确认删除 |
| `Esc` | 取消 |
| `Q` | 退出 |
| `R` | 重新扫描 |

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
make build          # 构建
make check          # 代码检查 (clippy + fmt)
make test           # 运行测试
make dist           # 全平台打包
```

## 安全机制

- 拒绝扫描系统关键路径（`/`, `/bin`, `/usr` 等）
- 最大扫描深度 5 层
- 文件默认不选中，需手动选择

## 构建要求

- Rust edition 2024
