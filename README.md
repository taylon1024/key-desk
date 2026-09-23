# key-desk

本地环境变量管理程序。一个 Rust 窗口，变量存在 SQLite 里，可以导出 `.env`。

当前值以明文保存在本机数据库中，这是脚手架，还没有加密。

## 目录

- `src/main.rs`：打开窗口
- `src/app.rs`：界面
- `src/db.rs`：SQLite 读写
- `src/models.rs`：变量校验和 `.env` 文本
- `data/variables.db`：运行后自动创建

## 开发

```bash
cargo run
```

`KEY_DESK_DB` 可覆盖数据库路径，默认是 `data/variables.db`。导出文件写到数据库所在目录。
