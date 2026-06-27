import { defineConfig } from "wxt";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  manifest: {
    name: "书签库收藏助手",
    description: "一键收藏当前网页到自建书签库",
    permissions: ["storage", "contextMenus", "activeTab", "notifications"],
    // 服务器地址是用户在 options 页里自己填的，构建时不知道具体域名，
    // 所以声明成 optional_host_permissions，等用户填好地址后再按需申请。
    optional_host_permissions: ["http://*/*", "https://*/*"],
  },
  vite: () => ({
    plugins: [tailwindcss()],
  }),
});
