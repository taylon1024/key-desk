# 开发进度

最近更新：2026-09-24 ｜ 分支：`cursor/windows-support-on-dev-1716` ｜ 版本：`0.0.2`

## 当前状态

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| 本地存储 | 可用 | SQLite 按 scope 和 key 唯一索引；变量值以 AES-256-GCM 加密。 |
| 变量管理 | 可用 | 新增、编辑、删除、筛选、复制单条和复制 `.env`。 |
| Provider Key | 可用 | 模板提供变量名与 Base URL；示例密钥只作输入提示。 |
| 环境导入 | 可用 | macOS / Linux 用登录 shell；Windows 读用户 + 系统环境。 |
| 身份验证 | 待发布验证 | 发布模式：macOS Touch ID 或账户密码；Windows 用当前登录会话，不弹 Windows Hello。`dev` 默认跳过。 |
| 密钥存储 | 可用 | macOS 钥匙串与 Windows 凭据管理器；旧版 `*.db.key` 只读，不新建。 |
| 界面 | 持续优化 | 原生 egui，固定面板尺寸、统一列宽；像素风仅用现有字体、图标和绘制指令。 |

## 2026-09-24

- 在当前 `dev` 界面上补上 Windows 平台支持：凭据管理器保存 AES 密钥，`%SystemRoot%\Fonts` 发现 Consolas / 微软雅黑，环境导入读取用户和系统变量。
- macOS Touch ID 与粉色纸面主题保持不变。可切换的多主题仍留作后续，不并入这次改动。
- Linux 编译启用 eframe 的 X11 / Wayland，并把 Apple 框架依赖限制在 macOS，以便在 Linux 上 `cargo test` / `cargo check`。

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
3. 发布前逐一核对各 Provider 默认 URL，并在关闭 `dev` feature 后做 Touch ID 实机验收。
4. 对最小窗口、长 scope、导入弹窗和替换弹窗进行手动视觉验收。
5. 多主题切换（旧的 12 套主题草稿基于黑白界面）留到基于当前 `dev` 界面的后续改动，不在这次 Windows 支持里。
6. 在 Windows 实机上确认凭据管理器条目、Consolas / 微软雅黑，以及「user + machine」导入。

## 更新约定

每次改动功能或交互时同步更新本页的状态、日期、验证结果与待办；发布前清理已完成项。

## 最近验证

- `cargo fmt --check`：通过。
- `cargo test --quiet`：20 项测试通过（含主窗口、列表和表单布局稳定性、损坏密文处理、Windows 字体路径与 CRLF 环境行）。
- `cargo clippy --all-targets -- -D warnings`：通过。
- `cargo check --no-default-features`：通过（仅编译检查，尚未做发布模式实机验证）。
- `cargo check --target x86_64-pc-windows-gnu` 与 `--no-default-features`：通过（Linux 上的交叉检查，不是 Windows 实机）。
