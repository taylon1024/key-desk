# Envman

本地环境变量管理程序。Rust 后端把变量存在 SQLite 里，TypeScript 前端提供增删改查和 `.env` 导出。

当前值以明文保存在本机数据库中，适合作为开发脚手架，还没有加密或权限控制。

## 目录

- `backend/`：Axum API，默认监听 `127.0.0.1:3001`
- `frontend/`：Vite + React + TypeScript，开发服务器把 `/api` 代理到后端
- `data/variables.db`：运行后自动创建的数据库

## 开发

开两个终端：

```bash
cd backend && cargo run
```

```bash
cd frontend && pnpm dev
```

浏览器打开 <http://localhost:5173>。

## 接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/health` | 健康检查 |
| `GET` | `/api/variables?scope=` | 列出变量，`scope` 可省略 |
| `POST` | `/api/variables` | 新增 |
| `PUT` | `/api/variables/{id}` | 更新 |
| `DELETE` | `/api/variables/{id}` | 删除 |
| `GET` | `/api/export?scope=` | 导出 dotenv 文本 |

## 配置

- `PORT`：后端端口，默认 `3001`
- `ENVMAN_DB`：数据库文件路径，默认 `data/variables.db`

构建前端后，后端会托管 `frontend/dist`：

```bash
cd frontend && pnpm build
cd ../backend && cargo run
```

然后打开 <http://127.0.0.1:3001>。
