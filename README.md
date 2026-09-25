# key-desk

本地保存环境变量和 LLM API Key 的 macOS 应用。变量放在本机 SQLite 里，值用 AES-256-GCM 加密。可以按 scope 筛选，把当前结果复制成 `.env`，也可以从菜单栏直接复制或编辑某一条。

![key-desk 主窗口](docs/images/app.png)

## 可以做什么

- 按 scope 保存变量，同一 scope 里名称不能重复。
- 列表默认遮住变量值，需要时再显示。复制时带上变量名，格式与 `.env` 一致。
- 从登录 shell 导入已有环境变量。
- 为常见模型服务商填入 API Key 和 Base URL 模板。
- 十三套配色，选择记在本机。
- 屏幕顶部的 **KD** 菜单列出当前变量，可以复制或打开窗口编辑。
- `main` 和发布包每次启动都要求 Touch ID 或 Mac 密码。

## 下载

macOS 12 及以上：[key-desk 0.0.2](https://github.com/taylon1024/key-desk/releases/tag/v0.0.2)

安装包未签名。第一次打开时，在 Key Desk 上右键并选择“打开”。

数据库、加密密钥文件和主题都在 `~/Library/Application Support/key-desk/`。密钥不写进数据库。界面上的 secret 只决定默认是否遮住显示。

## 开发

`main` 上直接运行会要求 Touch ID 或 Mac 密码：

```bash
cargo run
```

`dev` 分支默认跳过登录。

`KEY_DESK_DB` 可以指定数据库文件。`EXPORT` 只写入剪贴板，不生成文件。
