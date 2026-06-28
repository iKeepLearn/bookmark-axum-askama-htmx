import { getConfig, isConfigured } from "../utils/storage";
import { createBookmark } from "../utils/api";

const MENU_ID = "save-to-bookmark-library";

export default defineBackground(() => {
  browser.runtime.onInstalled.addListener(() => {
    browser.contextMenus.create({
      id: MENU_ID,
      title: "收藏到书签库",
      contexts: ["page", "link"],
    });
  });

  browser.contextMenus.onClicked.addListener(async (info, tab) => {
    if (info.menuItemId !== MENU_ID || !tab?.id) return;

    const config = await getConfig();
    if (!isConfigured(config)) {
      notify("还没有配置", "请先在插件设置里填写服务器地址和 Token");
      browser.runtime.openOptionsPage();
      return;
    }

    const targetUrl = info.linkUrl || tab.url || "";
    if (!targetUrl) {
      notify("收藏失败", "没能拿到这个页面的网址");
      return;
    }

    let meta: { title?: string; image?: string } = {};
    try {
      meta = await browser.tabs.sendMessage(tab.id, { type: "get-page-meta" });
    } catch {
      // content script 在这个页面不可用（比如浏览器内置页面），用 tab 信息兜底
    }

    const title = meta.title || tab.title || targetUrl;

    try {
      await createBookmark(config, {
        title,
        url: targetUrl,
        cover_image: meta.image || "",
        category_id: null,
        tag_ids: [],
        new_tags: "",
        desc: "",
      });
      notify("已收藏", title);
    } catch (err) {
      notify("收藏失败", err instanceof Error ? err.message : "未知错误");
    }
  });
});

function notify(title: string, message: string) {
  browser.notifications?.create({
    type: "basic",
    iconUrl: browser.runtime.getURL("/icon/128.png"),
    title,
    message,
  });
}
