.PHONY: build build-all clean install test release

# 当前平台构建
build:
	cargo build --release

# 所有平台交叉编译
build-all: build-linux build-windows build-macos

# Linux
build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM
build-linux-arm:
	cargo build --release --target aarch64-unknown-linux-gnu

# Windows
build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

# macOS
build-macos:
	cargo build --release --target x86_64-apple-darwin

# macOS ARM (Apple Silicon)
build-macos-arm:
	cargo build --release --target aarch64-apple-darwin

# 安装到本地
install:
	cargo install --path .

# 测试
test:
	cargo test

# 检查代码
check:
	cargo check
	cargo clippy -- -D warnings
	cargo fmt --check

# 格式化代码
fmt:
	cargo fmt

# 清理构建产物
clean:
	cargo clean
	rm -rf dist

# 打包分发
dist: build-all
	mkdir -p dist
	cp target/x86_64-unknown-linux-gnu/release/cleaner dist/cleaner-linux-amd64
	cp target/x86_64-pc-windows-gnu/release/cleaner.exe dist/cleaner-windows-amd64.exe
	cp target/x86_64-apple-darwin/release/cleaner dist/cleaner-macos-amd64
	@if [ -d "target/aarch64-apple-darwin/release" ]; then \
		cp target/aarch64-apple-darwin/release/cleaner dist/cleaner-macos-arm64; \
	fi
	@if [ -d "target/aarch64-unknown-linux-gnu/release" ]; then \
		cp target/aarch64-unknown-linux-gnu/release/cleaner dist/cleaner-linux-arm64; \
	fi
	@echo "打包完成，文件在 dist/ 目录"
	@ls -lh dist/

# 发布版本（创建 tag 并推送，触发 GitHub Actions 构建）
release:
	@ver=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/'); \
	echo "发布版本: v$$ver"; \
	git tag "v$$ver" && \
	git push origin "v$$ver" && \
	echo "已推送 tag v$$ver，GitHub Actions 将自动构建"

# 帮助
help:
	@echo "可用命令:"
	@echo "  make build          - 当前平台构建"
	@echo "  make build-all      - 所有平台构建"
	@echo "  make build-linux    - 构建 Linux 版本"
	@echo "  make build-windows  - 构建 Windows 版本"
	@echo "  make build-macos    - 构建 macOS 版本"
	@echo "  make install        - 安装到本地"
	@echo "  make test           - 运行测试"
	@echo "  make check          - 代码检查"
	@echo "  make fmt            - 格式化代码"
	@echo "  make clean          - 清理构建产物"
	@echo "  make dist           - 打包所有平台"
	@echo "  make release        - 发布版本（创建并推送 tag）"
