# 开发进度

最近更新：2026-09-25 ｜ 分支：`main` ｜ 版本：`0.0.2`

## 当前状态

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| 本地存储 | 可用 | SQLite 在 `~/Library/Application Support/key-desk/`；变量值以 AES-256-GCM 加密。仓库里的旧库只复制一次，原文件保留。 |
| 变量管理 | 可用 | 新增、编辑、删除、筛选、复制单条和复制 `.env`。 |
| Provider Key | 可用 | 模板提供变量名与 Base URL；示例密钥只作输入提示。 |
| 环境导入 | 可用 | 支持登录 shell 与当前进程，筛选、选择和填入表单。 |
| 身份验证 | 可用 | `main` 与发布包要求 Touch ID 或 Mac 密码；`dev` 默认跳过。 |
| 界面 | 持续优化 | 原生 egui，固定面板尺寸、统一列宽；像素风仅用现有字体、图标和绘制指令。 |

## 2026-09-25

- 增加 GitHub Actions 的 macOS Rust 检查与 Pull Request 模板；首次远端运行结果待验证。
- 合并 `dev` 的新版 README 和标题图；修正 `main` 的启动说明，并清理 4 处 Clippy 告警。
- macOS 菜单栏 KD 快捷项列出当前库中的变量。每一项可以复制 `KEY=value`，或打开主窗口编辑该条。值不会出现在菜单文字里。
- 数据库、同名 `.key` 和 `theme_id` 改到用户的 Application Support 目录。`KEY_DESK_DB` 仍可覆盖路径。发布包由打包脚本去掉 `dev`，启动要求 Touch ID 或 Mac 密码。签名和公证需要本机已有的 Developer ID 证书。

## 2026-09-23

- 稳定列表、表单和状态栏布局；统一 scope 控件宽度，长名称在菜单中截断并可悬停查看全文。
- 调整方角黑色标题条、淡粉底色和点阵装饰，参考 [TypeSafe AI](https://typesafe.ai/) 的窗口视觉；没有新增 UI 依赖或网络资源。
- 修复 Provider 模板把示例密钥当作真实值、环境导入窗口无法关闭、非 ASCII 预览截断可能崩溃的问题。
- 系统环境变量重名时明确提示保存的是独立本地条目；损坏密文会报错，不再伪装成普通值。
- 新数据库只在钥匙串存密钥，继续读取旧版 `.db.key` 文件；清理未使用的 Provider 结构体，使模板列表不再逐帧分配。
- 根据 [Google 官方文档](https://ai.google.dev/gemini-api/docs/openai) 修正 Gemini 的 OpenAI 兼容 Base URL。

## 待办

1. 制定旧版 `.db.key` 文件迁移到钥匙串的可恢复流程；在迁移前不能自动删除旧文件。
2. Provider Key 与 Base URL 目前分别写入；若第二条冲突，界面会报告部分成功。后续设计成可确认的原子写入。
3. 用去掉 `dev` 的发布包在本机走一遍 Touch ID 或 Mac 密码。签名和公证要等 Developer ID 证书。
4. 对最小窗口、长 scope、导入弹窗和替换弹窗进行手动视觉验收。

## 更新约定

每次改动功能或交互时同步更新本页的状态、日期、验证结果与待办；发布前清理已完成项。

## 最近验证

- `cargo fmt --check`：通过。
- `cargo test`：25 项测试通过（含主窗口、列表和表单布局稳定性、损坏密文处理）。
- `cargo clippy --all-targets -- -D warnings`：首次运行因 4 处旧告警未通过；修正后重跑通过。
- `cargo check --no-default-features`：通过（仅编译检查，尚未做发布模式实机验证）。
