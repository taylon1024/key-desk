---
name: package-dmg
description: 把 key-desk 打成 macOS .app 并封装为 DMG 安装包。在用户要求封装、打包、导出 dmg、制作安装包，或提到 hdiutil / Key Desk.app 时使用。
---

# 封装 key-desk DMG

在仓库根目录执行 [scripts/package-dmg.sh](scripts/package-dmg.sh)。不要手写另一套打包命令。

## 打包前

1. 确认在 macOS 上，且工作区是 key-desk 仓库根目录。
2. 读 `src/db.rs` 的 `db_path`。若仍使用 `CARGO_MANIFEST_DIR`，先告诉用户：安装包在别的电脑上没有可写的数据库目录，正式分发前应改为 `~/Library/Application Support/key-desk/`。用户明确说只在本机试用时，可以继续打包，并在结果里写明数据库实际路径。
3. 发布包必须去掉 `dev` 特性，否则启动不要求 Touch ID 或 Mac 密码。脚本已使用 `--no-default-features`，不要改成普通 `cargo build --release`。

## 执行

```bash
.cursor/skills/package-dmg/scripts/package-dmg.sh
```

脚本会：

- `cargo build --release --no-default-features`
- 组装 `dist/Key Desk.app`
- 生成 `dist/key-desk-<版本>.dmg`（版本取自 `Cargo.toml`）
- DMG 内含应用，以及指向 `/Applications` 的替身

不要把 `dist/`、`.app`、`.dmg` 提交进 git。

## 完成后告知用户

- DMG 的路径
- 未签名：其他 Mac 需在应用上右键“打开”
- 去掉该提示需要 Apple Developer 证书 `codesign`，再用 `notarytool` 公证
- 数据库路径是否仍指向编译期的项目目录
