# key-desk

本地环境变量与 Provider API Key 管理程序。原生 Rust 窗口，变量存在 SQLite 中；`EXPORT` 会把当前筛选结果以 `.env` 格式复制到剪贴板。

变量值用 AES-256-GCM 加密后写入 SQLite。新数据库的密钥存在系统钥匙串里（macOS 为「钥匙串访问」中的 `key-desk` / `sqlite-value-key`）；旧版本生成的同名 `.db.key` 文件仍可读取，**迁移完成前请勿删除旧密钥文件**。界面上的 `secret` 只控制是否遮罩显示。

第一次写入时会在钥匙串里生成密钥；若系统弹出授权，需要允许本程序访问该钥匙串项。旧的明文记录会在打开数据库时自动改写成密文。开发分支默认启用 `dev` feature，跳过 Touch ID；验证发布模式请运行 `cargo run --no-default-features`。

## 目录

- `src/main.rs`：打开窗口
- `src/ui/`：界面（样式、列表、表单、环境导入、重名确认）
- `src/db.rs`：SQLite 读写
- `src/models.rs`：变量校验和 `.env` 文本
- `data/variables.db`：运行后自动创建
- `docs/DEVELOPMENT_PROGRESS.md`：开发进度、验证结果和待办

## 开发

```bash
cargo run
```

`KEY_DESK_DB` 可覆盖数据库路径，默认是 `data/variables.db`。导出结果写入剪贴板，不会自动生成文件。
