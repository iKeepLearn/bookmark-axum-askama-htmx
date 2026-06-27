# 书签库收藏助手（浏览器插件）

用 [WXT](https://wxt.dev) 写的 Manifest V3 浏览器插件，一键把当前网页收藏到你自建的书签库。

## 功能

- **Popup 弹窗**：点击图标打开，自动填好标题/网址/封面图（读取页面的 `og:title`/`og:image`），可以选分类、选/加标签、写描述，保存。
- **右键菜单**：在网页或链接上右键 →「收藏到书签库」，不弹窗，用默认信息直接快速收藏（标题、网址、封面图，不带分类/标签），完成后弹系统通知。
- **设置页**：填书签库地址 + API Token，保存时会请求一次跨域访问授权（只授权给你填的这个域名，不是 `<all_urls>`）。

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
  options/      设置页（服务器地址 + Token）
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

## 需要后端配合新增的部分

插件这边假设你的后端会提供下面这组 JSON API，用 `Authorization: Bearer <token>` 认证：

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/categories` | 返回 `[{ "id": number, "name": string }]` |
| GET | `/api/tags` | 返回 `[{ "id": number, "name": string }]` |
| POST | `/api/navs` | 创建一条书签 |

`POST /api/navs` 的请求体：

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

`category_id` 可以是 `null`（不选分类），`tag_ids` 可以是空数组。成功返回 `{ "id": number }`，失败返回 4xx + `{ "error": "string" }`（插件会把 `error` 文本直接展示给用户，所以最好是人能看懂的中文）。

另外还需要一个**登录后才能访问的网页**（比如 `/manage/tokens`），用来生成/查看/吊销 API Token——插件的设置页需要用户去这个页面复制一个 token 粘贴进来。这两块（JSON API + token 管理页的 Rust 代码）我还没写，需要的话告诉我一声，我接着补上。

## 关于权限

- `optional_host_permissions` declares 了 `http://*/*` 和 `https://*/*`，但**不会**在安装时就要求用户授权——只有在设置页保存服务器地址时，才会针对那一个域名弹一次授权请求。这是为了避免在应用商店审核时被要求解释为什么需要访问所有网站。
- `content_scripts` 的 `matches` 是 `<all_urls>`，这个没法收窄，因为读取 `og:` 元信息需要能在任意你想收藏的网站上跑。这只是读取，不会上传任何东西，除非你主动点了保存。

## 图标

`public/icon/` 下已经放了一套占位图标（书签丝带形状，配色和书签库主站一致）。想换成自己的设计，直接替换这几个尺寸的 PNG 文件即可，WXT 会自动读取并写进 manifest。
