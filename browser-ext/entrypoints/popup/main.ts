import "../../assets/tailwind.css";
import { getConfig, isConfigured } from "../../utils/storage";
import { fetchCategories, fetchTags, createNav } from "../../utils/api";
import type { Category, Tag } from "../../utils/api";
import { escapeHtml } from "../../utils/dom";

const app = document.getElementById("app")!;

interface PageMeta {
  title?: string;
  description?: string;
  image?: string;
}

async function getActiveTab() {
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
  return tab;
}

async function getPageMeta(tabId: number): Promise<PageMeta | null> {
  try {
    return await browser.tabs.sendMessage(tabId, { type: "get-page-meta" });
  } catch {
    // content script 在这个页面不可用（比如浏览器内置页面），返回 null 走兜底
    return null;
  }
}

function renderLoading() {
  app.innerHTML = `
    <div class="py-10 text-center text-ink-faint text-sm">加载中…</div>
  `;
}

function renderNeedsSetup(message?: string) {
  app.innerHTML = `
    <div class="text-center py-6">
      <div class="font-display font-semibold text-base text-ink mb-2">还没有配置服务器</div>
      <p class="text-ink-soft text-[13px] mb-4">${
        message ? escapeHtml(message) : "需要先填写书签库地址和 API Token"
      }</p>
      <button id="open-options" class="px-4 py-2 bg-accent text-white text-sm rounded-lg hover:bg-accent-ink transition-colors">
        去配置
      </button>
    </div>
  `;
  document.getElementById("open-options")!.addEventListener("click", () => {
    browser.runtime.openOptionsPage();
  });
}

function chipRadio(
  name: string,
  value: number,
  label: string,
  checked: boolean,
) {
  return `
    <label class="cursor-pointer text-[12.5px] rounded-md px-2.5 py-1 border border-line text-ink-soft bg-paper-deeper has-checked:bg-accent has-checked:text-white has-checked:border-accent transition-colors">
      <input type="radio" name="${name}" value="${value}" class="hidden" ${checked ? "checked" : ""} />
      ${escapeHtml(label)}
    </label>
  `;
}

function chipCheckbox(name: string, value: number, label: string) {
  return `
    <label class="cursor-pointer text-[12.5px] rounded-md px-2.5 py-1 border border-line text-ink-soft bg-paper-deeper has-checked:bg-accent has-checked:text-white has-checked:border-accent transition-colors">
      <input type="checkbox" name="tag_ids" value="${value}" class="hidden" />
      ${escapeHtml(label)}
    </label>
  `;
}

function renderForm(categories: Category[], tags: Tag[]) {
  app.innerHTML = `
    <div class="flex items-center justify-between mb-4">
      <div class="font-display font-semibold text-base text-ink">保存到书签库</div>
      <button id="open-options" type="button" class="text-ink-faint hover:text-ink-soft transition-colors" title="设置" aria-label="设置">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </button>
    </div>

    <form id="nav-form" class="flex flex-col gap-4">
      <div class="w-full">
        <input
          id="title" type="text" placeholder="title" required
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
      </div>

      <div class="w-full">
        <input
          id="url" type="text" placeholder="url" required
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
      </div>

      <div class="w-full">
        <input
          id="cover_image" type="text" placeholder="cover image"
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
        <img id="cover-preview" class="hidden mt-2 w-full aspect-video object-cover rounded-lg border border-line" alt="" />
      </div>

      ${
        categories.length
          ? `<div class="flex flex-wrap gap-1.5">${categories
              .map((c) => chipRadio("category_id", c.id, c.name, false))
              .join("")}</div>`
          : ""
      }

      <div class="w-full">
        <input
          id="new_tags" type="text" placeholder="新增标签（逗号分隔）"
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors mb-2"
        />
        ${
          tags.length
            ? `<div class="flex flex-wrap gap-1.5">${tags
                .map((t) => chipCheckbox("tag_ids", t.id, t.name))
                .join("")}</div>`
            : ""
        }
      </div>

      <textarea
        id="desc" rows="2" placeholder="desc"
        class="w-full border border-line rounded-lg px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors resize-none"
      ></textarea>

      <div id="status" class="text-sm min-h-4.5"></div>

      <button
        type="submit" id="submit-btn"
        class="px-4 py-2.5 inline-flex items-center justify-center bg-accent text-white font-medium text-sm rounded-lg shadow-sm hover:bg-accent-ink transition-colors disabled:opacity-50"
      >
        保存
      </button>
    </form>
  `;

  document.getElementById("open-options")!.addEventListener("click", () => {
    browser.runtime.openOptionsPage();
  });

  const coverInput = document.getElementById("cover_image") as HTMLInputElement;
  const coverPreview = document.getElementById(
    "cover-preview",
  ) as HTMLImageElement;
  coverInput.addEventListener("input", () => {
    const v = coverInput.value.trim();
    if (v) {
      coverPreview.src = v;
      coverPreview.classList.remove("hidden");
    } else {
      coverPreview.classList.add("hidden");
    }
  });
}

function setStatus(text: string, kind: "ok" | "error" | "info") {
  const el = document.getElementById("status");
  if (!el) return;
  el.textContent = text;
  el.className =
    "text-sm min-h-[18px] " +
    (kind === "ok"
      ? "text-accent-ink"
      : kind === "error"
        ? "text-danger"
        : "text-ink-soft");
}

async function init() {
  renderLoading();

  const config = await getConfig();
  if (!isConfigured(config)) {
    renderNeedsSetup();
    return;
  }

  const tab = await getActiveTab();
  const meta = tab?.id ? await getPageMeta(tab.id) : null;

  let categories: Category[] = [];
  let tags: Tag[] = [];
  try {
    [categories, tags] = await Promise.all([
      fetchCategories(config),
      fetchTags(config),
    ]);
  } catch (err) {
    renderNeedsSetup(
      err instanceof Error
        ? `连接服务器失败：${err.message}`
        : "连接服务器失败，检查地址和 Token 是否正确",
    );
    return;
  }

  renderForm(categories, tags);

  const titleInput = document.getElementById("title") as HTMLInputElement;
  const urlInput = document.getElementById("url") as HTMLInputElement;
  const coverInput = document.getElementById("cover_image") as HTMLInputElement;
  const coverPreview = document.getElementById(
    "cover-preview",
  ) as HTMLImageElement;

  titleInput.value = meta?.title || tab?.title || "";
  urlInput.value = tab?.url || "";
  coverInput.value = meta?.image || "";
  if (coverInput.value) {
    coverPreview.src = coverInput.value;
    coverPreview.classList.remove("hidden");
  }

  const form = document.getElementById("nav-form") as HTMLFormElement;
  const submitBtn = document.getElementById("submit-btn") as HTMLButtonElement;

  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    submitBtn.disabled = true;
    setStatus("保存中…", "info");

    const categoryRaw = (
      form.elements.namedItem("category_id") as RadioNodeList | null
    )?.value;
    const tagCheckboxes = Array.from(
      form.querySelectorAll<HTMLInputElement>('input[name="tag_ids"]:checked'),
    );

    try {
      await createNav(config, {
        title: titleInput.value.trim(),
        url: urlInput.value.trim(),
        cover_image: coverInput.value.trim(),
        category_id: categoryRaw ? Number(categoryRaw) : null,
        tag_ids: tagCheckboxes.map((el) => Number(el.value)),
        new_tags: (
          document.getElementById("new_tags") as HTMLInputElement
        ).value.trim(),
        desc: (
          document.getElementById("desc") as HTMLTextAreaElement
        ).value.trim(),
      });
      setStatus("已保存 ✓", "ok");
      setTimeout(() => window.close(), 700);
    } catch (err) {
      submitBtn.disabled = false;
      setStatus(
        err instanceof Error ? err.message : "保存失败，未知错误",
        "error",
      );
    }
  });
}

init();
