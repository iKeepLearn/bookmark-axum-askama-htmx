export default defineContentScript({
  matches: ["<all_urls>"],
  main() {
    browser.runtime.onMessage.addListener((message) => {
      if (
        typeof message === "object" &&
        message !== null &&
        "type" in message &&
        message.type === "get-page-meta"
      ) {
        return Promise.resolve(getPageMeta());
      }
    });
  },
});

function readMeta(key: string): string {
  const byProperty = document.querySelector(`meta[property="${key}"]`);
  const byName = document.querySelector(`meta[name="${key}"]`);
  return (
    byProperty?.getAttribute("content") || byName?.getAttribute("content") || ""
  );
}

function getPageMeta() {
  return {
    title: readMeta("og:title") || document.title,
    description: readMeta("og:description") || readMeta("description"),
    image: readMeta("og:image") || readMeta("twitter:image"),
  };
}
