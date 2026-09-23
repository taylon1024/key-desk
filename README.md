# key-desk

本地环境变量管理程序。一个 Rust 窗口，变量存在 SQLite 里，可以导出 `.env`。

变量值用 AES-256-GCM 加密后写入 SQLite。密钥存在系统钥匙串里（macOS 为「钥匙串访问」中的 `key-desk` / `sqlite-value-key`），数据库文件本身不含密钥。界面上的「敏感值」只控制是否遮罩显示。

第一次启动会在钥匙串里生成密钥；若系统弹出授权，需要允许本程序访问该钥匙串项。旧的明文记录会在打开数据库时自动改写成密文。

## 目录

- `src/main.rs`：打开窗口
- `src/ui/`：界面（`theme` 样式、`masthead` 页眉、`ledger` 列表、`form` 表单、`env_import` 环境导入）
- `src/db.rs`：SQLite 读写
- `src/models.rs`：变量校验和 `.env` 文本
- `data/variables.db`：运行后自动创建

## 开发

```bash
cargo run
```

`KEY_DESK_DB` 可覆盖数据库路径，默认是 `data/variables.db`。导出文件写到数据库所在目录。
