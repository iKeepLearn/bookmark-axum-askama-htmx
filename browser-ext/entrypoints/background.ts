import { getConfig, isConfigured } from "../utils/storage";
import { createBookmark } from "../utils/api";

const MENU_ID = "save-to-bookmark-library";
const QUICK_SAVE_WINDOW_WIDTH = 400;
const QUICK_SAVE_WINDOW_HEIGHT = 640;

interface PageMeta {
  title?: string;
  image?: string;
}

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

    let meta: PageMeta = {};
    try {
      meta = await browser.tabs.sendMessage(tab.id, { type: "get-page-meta" });
    } catch {
      // content script 在这个页面不可用（比如浏览器内置页面），用 tab 信息兜底
    }

    const params = new URLSearchParams({
      title: meta.title || tab.title || "",
      url: targetUrl,
      image: meta.image || "",
    });

    await browser.windows.create({
      url: browser.runtime.getURL(`/quick-save.html?${params.toString()}`),
      type: "popup",
      width: QUICK_SAVE_WINDOW_WIDTH,
      height: QUICK_SAVE_WINDOW_HEIGHT,
      focused: true,
    });
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
