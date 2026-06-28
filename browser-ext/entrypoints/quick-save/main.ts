import "../../assets/tailwind.css";
import { getConfig, isConfigured } from "../../utils/storage";
import { fetchCategories, fetchTags } from "../../utils/api";
import {
  renderLoading,
  renderNeedsSetup,
  renderConnectionError,
  mountBookmarkForm,
} from "../../utils/bookmark-form";

const app = document.getElementById("app")!;

function readInitialFromQuery() {
  const params = new URLSearchParams(window.location.search);
  return {
    title: params.get("title") || "",
    url: params.get("url") || "",
    coverImage: params.get("image") || "",
  };
}

async function init() {
  renderLoading(app);

  const config = await getConfig();
  if (!isConfigured(config)) {
    renderNeedsSetup(app, () => browser.runtime.openOptionsPage());
    return;
  }

  const initial = readInitialFromQuery();

  try {
    const [categories, tags] = await Promise.all([
      fetchCategories(config),
      fetchTags(config),
    ]);

    mountBookmarkForm({
      container: app,
      config,
      categories,
      tags,
      initial,
      onOpenOptions: () => browser.runtime.openOptionsPage(),
      onSaved: () => setTimeout(() => window.close(), 700),
    });
  } catch (err) {
    renderConnectionError(
      app,
      err instanceof Error
        ? err.message
        : "连接服务器失败，检查地址和 Token 是否正确",
      () => browser.runtime.openOptionsPage(),
    );
  }
}

init();
