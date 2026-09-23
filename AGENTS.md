# 项目协作约定

- 修改功能、交互或安全行为时，同步更新 `docs/DEVELOPMENT_PROGRESS.md` 的状态和日期。
- 提交前运行 `cargo fmt --check`、`cargo test` 和 `cargo clippy --all-targets -- -D warnings`，记录未通过项。
- 不提交数据库、密钥文件、真实环境变量或凭据。
- 在聊天中展示本地图片时使用绝对路径的 Markdown 图片语法：`![说明](/absolute/path/image.png)`。
