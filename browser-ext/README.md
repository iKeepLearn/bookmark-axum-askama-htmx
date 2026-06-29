# 书签库收藏助手（浏览器插件）

用 [WXT](https://wxt.dev) 写的 Manifest V3 浏览器插件，一键把当前网页收藏到你自建的书签库。

## 功能

- **Popup 弹窗**：点击图标打开，自动填好标题/网址/封面图（读取页面的 `og:title`/`og:image`），可以选分类、选/加标签、写描述，保存。
- **右键菜单**：在网页或链接上右键 →「收藏到书签库」，不弹窗，用默认信息直接快速收藏（标题、网址、封面图，不带分类/标签），完成后弹系统通知。
- **设置页**：填写书签库地址 + 用户名 + 密码，点击「生成密钥」自动获取 API Token（会请求该域名的访问权限），也可以测试连接。

## 配置说明

1. 点击插件图标，再点击右上角的设置图标进入设置页面
2. 填写：
   - **书签库地址**：你的书签库 URL（如 `https://bks.artista.cc`，不要加结尾的斜杠）
   - **用户名**：你的登录用户名
   - **密码**：你的登录密码
3. 点击「生成密钥」：插件会自动用用户名密码获取 API Token 并保存
4. 可选：点击「测试连接」验证配置是否正确

配置完成后就可以开始收藏网页了！

## 本地开发

```bash
bun install      # 会自动跑一次 wxt prepare
bun run dev      # Chrome，热重载
bun run dev:firefox
```

`bun run dev` 之后，按照终端提示在 `chrome://extensions` 里加载 `.output/chrome-mv3-dev` 目录（开发者模式 → 加载已解压的扩展程序）。

## 打包

```bash
bun run build           # 产物在 .output/chrome-mv3
bun run build:firefox
bun run zip              # 直接打成 .zip，方便上传应用商店或自己分发
```

## 目录结构

```
entrypoints/
  popup/        弹窗 UI（点图标）
  options/      设置页（服务器地址 + 用户名密码 → 自动获取 Token）
  quick-save/   右键快速保存的侧边栏
  background.ts 右键菜单 + 通知
  content.ts    读取当前页面的 og 元信息
utils/
  storage.ts    本地存储封装
  api.ts        和后端 JSON API 通信
  dom.ts        转义网页来源文本，防止拼进 innerHTML 时出问题
assets/
  tailwind.css  Tailwind v4 入口，沿用书签库主站同一套设计 token
public/icon/    插件图标（16/32/48/96/128，已生成好占位图，可以替换成自己的设计）
```

## 后端 API

插件需要后端提供以下 API：

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/token` | 用用户名密码生成 API Token |
| GET | `/api/categories` | 返回 `[{ "id": number, "name": string }]` |
| GET | `/api/tags` | 返回 `[{ "id": number, "name": string }]` |
| POST | `/api/bookmark` | 创建一条书签 |

### POST /api/tokens

请求体：
```json
{
  "username": "string",
  "password": "string"
}
```

成功返回：
```json
{ "token": "string" }
```

失败返回 4xx + `{ "error": "string" }`。

### POST /api/bookmark

请求体：
```json
{
  "title": "string",
  "url": "string",
  "cover_image": "string",
  "category_id": 1,
  "tag_ids": [1, 2],
  "new_tags": "笔记,摄影",
  "desc": "string"
}
```

`tag_ids` 可以是空数组。成功返回 `{ "id": number }`，失败返回 4xx + `{ "error": "string" }`（插件会把 `error` 文本直接展示给用户，所以最好是人能看懂的中文）。

其他 API 请求需要带 Header：`Authorization: Bearer <token>`。

## 关于权限

- `optional_host_permissions` declares 了 `http://*/*` 和 `https://*/*`，但**不会**在安装时就要求用户授权——只有在设置页保存服务器地址时，才会针对那一个域名弹一次授权请求。这是为了避免在应用商店审核时被要求解释为什么需要访问所有网站。
- `content_scripts` 的 `matches` 是 `<all_urls>`，这个没法收窄，因为读取 `og:` 元信息需要能在任意你想收藏的网站上跑。这只是读取，不会上传任何东西，除非你主动点了保存。

## 图标

`public/icon/` 下已经放了一套占位图标（书签丝带形状，配色和书签库主站一致）。想换成自己的设计，直接替换这几个尺寸的 PNG 文件即可，WXT 会自动读取并写进 manifest。
