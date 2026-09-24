# key-desk

本地环境变量与 Provider API Key 管理程序。原生 Rust 窗口，变量存在 SQLite 中；`EXPORT` 会把当前筛选结果以 `.env` 格式复制到剪贴板。

变量值用 AES-256-GCM 加密后写入 SQLite。数据库里的格式是 `kd1:` + base64(nonce || ciphertext)。新密钥不放在数据库里，也不再新建 `*.db.key`：没有旧密钥文件时，向系统存储取或新建一把。旧版本生成的同名 `.db.key` 仍可读取，**迁移完成前请勿删除旧密钥文件**。界面上的 `secret` 只控制是否遮罩显示。

| 系统 | 存储 | 解锁 |
| --- | --- | --- |
| macOS | 登录钥匙串（钥匙串访问里的通用密码）。服务名 `key-desk`，账户 `sqlite-value-key` | 发布模式每次打开用 Touch ID；没有指纹或指纹失败时回退到 Mac 登录密码。第一次写入钥匙串时系统可能弹出授权 |
| Windows | 凭据管理器里的通用凭据。目标名是 `sqlite-value-key.key-desk`（`keyring` 把「账户.服务」拼成 target）。可在「控制面板 → 凭据管理器 → Windows 凭据」里看到 | 不弹 Windows Hello。当前 Windows 登录用户就是边界。凭据按该用户保护，读取时不再额外弹窗 |

两边的密文格式相同。只拷数据库、不拷旧的 `.db.key` 时，会用那台机器自己的钥匙串或凭据管理器，解不开原来的密文。

旧的明文记录会在打开数据库时自动改写成密文。开发分支默认启用 `dev` feature，各平台都跳过打开时的确认（包括 macOS Touch ID）；验证发布模式请运行 `cargo run --no-default-features`。

## 目录

- `src/main.rs`：打开窗口
- `src/ui/`：界面（样式、列表、表单、环境导入、重名确认）
- `src/db.rs`：SQLite 读写
- `src/crypto.rs`：AES-256-GCM 与旧版 `*.db.key` 读取
- `src/os_key.rs`：钥匙串 / 凭据管理器
- `src/auth/`：macOS Touch ID；Windows 与其他系统直接解锁
- `src/models.rs`：变量校验和 `.env` 文本
- `data/variables.db`：运行后自动创建
- `docs/DEVELOPMENT_PROGRESS.md`：开发进度、验证结果和待办

## 开发

需要 Rust 1.95 或更新（`edition = "2024"`，并且 `eframe` 0.36 要求 1.95）。Linux 上窗口后端启用了 X11 和 Wayland；macOS 与 Windows 仍用 glow。

```bash
cargo run
```

`KEY_DESK_DB` 可覆盖数据库路径，默认是 `data/variables.db`。导出结果写入剪贴板，不会自动生成文件。

### Windows

在 Windows 上装好 Rust（MSVC 工具链，或 GNU 的 `x86_64-pc-windows-gnu`）后：

```bash
cargo run
```

在别的系统上可以先做类型检查。`rusqlite` 会编译 SQLite 的 C 代码，所以 GNU 目标需要 `x86_64-w64-mingw32-gcc`：

```bash
rustup target add x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu
```

MSVC 目标在 Windows 上检查：

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --target x86_64-pc-windows-msvc
```

字体：等宽优先 `%SystemRoot%\Fonts\consola.ttf`（Consolas），没有再试 Cascadia Mono、Courier New、Lucida Console。中文优先 `msyh.ttc`（微软雅黑，字体集的第 0 个面），没有再试 `msyh.ttf`、宋体 `simsun.ttc`、微软正黑 `msjh.ttc`。系统盘不是 `C:` 时走 `%SystemRoot%\Fonts`，并仍会尝试 `C:\Windows\Fonts`。

导入环境变量时，「user + machine」读的是用户和系统环境，不是当前进程继承到的那一份。同名变量用户优先；`Path` 按「系统;用户」拼接。
