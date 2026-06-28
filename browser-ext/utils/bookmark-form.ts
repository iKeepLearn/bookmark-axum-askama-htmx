import type { ExtensionConfig } from "./storage";
import type { Category, Tag } from "./api";
import { createBookmark, uploadCoverImage } from "./api";
import { escapeHtml } from "./dom";

export interface BookmarkFormInitial {
  title: string;
  url: string;
  coverImage: string;
}

export interface MountBookmarkFormOptions {
  container: HTMLElement;
  config: ExtensionConfig;
  categories: Category[];
  tags: Tag[];
  initial: BookmarkFormInitial;
  /** 点了右上角的设置图标 */
  onOpenOptions: () => void;
  /** 保存成功之后（比如可以用来关闭弹窗/窗口） */
  onSaved: () => void;
}

function chipRadio(value: number, label: string) {
  return `
    <label class="cursor-pointer text-[12.5px] rounded-md px-2.5 py-1 border border-line text-ink-soft bg-paper-deeper has-checked:bg-accent has-checked:text-white has-checked:border-accent transition-colors">
      <input type="radio" name="category_id" value="${value}" class="hidden" />
      ${escapeHtml(label)}
    </label>
  `;
}

function chipCheckbox(value: number, label: string) {
  return `
    <label class="cursor-pointer text-[12.5px] rounded-md px-2.5 py-1 border border-line text-ink-soft bg-paper-deeper has-checked:bg-accent has-checked:text-white has-checked:border-accent transition-colors">
      <input type="checkbox" name="tag_ids" value="${value}" class="hidden" />
      ${escapeHtml(label)}
    </label>
  `;
}

const SETTINGS_ICON = `
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
`;

export function renderLoading(container: HTMLElement) {
  container.innerHTML = `<div class="py-10 text-center text-ink-faint text-sm">加载中…</div>`;
}

export function renderConnectionError(
  container: HTMLElement,
  message: string,
  onOpenOptions: () => void,
) {
  container.innerHTML = `
    <div class="text-center py-6">
      <div class="font-display font-semibold text-base text-ink mb-2">连接失败</div>
      <p class="text-ink-soft text-[13px] mb-4">${escapeHtml(message)}</p>
      <button id="bf-open-options" class="px-4 py-2 cursor-pointer bg-accent text-white text-sm rounded-lg hover:bg-accent-ink transition-colors">
        检查设置
      </button>
    </div>
  `;
  document
    .getElementById("bf-open-options")!
    .addEventListener("click", onOpenOptions);
}

export function renderNeedsSetup(
  container: HTMLElement,
  onOpenOptions: () => void,
) {
  container.innerHTML = `
    <div class="text-center py-6">
      <div class="font-display font-semibold text-base text-ink mb-2">还没有配置服务器</div>
      <p class="text-ink-soft text-[13px] mb-4">需要先填写书签库地址和 API Token</p>
      <button id="bf-open-options" class="px-4 py-2 cursor-pointer bg-accent text-white text-sm rounded-lg hover:bg-accent-ink transition-colors">
        去配置
      </button>
    </div>
  `;
  document
    .getElementById("bf-open-options")!
    .addEventListener("click", onOpenOptions);
}

/**
 * 允许两种合法的图片/网址值：
 *  - 正常的 http(s) 链接
 *  - 后端上传接口返回的相对路径（例如 "images/xxx.jpg"）
 */
function isValidHttpUrl(value: string): boolean {
  if (value.startsWith("images/")) return true;
  try {
    const u = new URL(value);
    return u.protocol === "http:" || u.protocol === "https:";
  } catch {
    return false;
  }
}

function validateBookmarkForm(input: {
  title: string;
  url: string;
  cover_image: string;
}): string | null {
  if (!input.title) return "请填写标题";
  if (!input.url) return "请填写网址";
  if (!isValidHttpUrl(input.url)) return "网址格式不正确";
  if (input.cover_image && !isValidHttpUrl(input.cover_image)) {
    return "封面图片地址格式不正确";
  }
  return null;
}

export function mountBookmarkForm(opts: MountBookmarkFormOptions) {
  const {
    container,
    config,
    categories,
    tags,
    initial,
    onOpenOptions,
    onSaved,
  } = opts;

  container.innerHTML = `
    <div class="flex items-center justify-between mb-4">
      <div class="font-display font-semibold text-base text-ink">保存到书签库</div>
      <button id="bf-open-options" type="button" class="text-ink-faint cursor-pointer hover:text-ink-soft transition-colors" title="设置" aria-label="设置">
        ${SETTINGS_ICON}
      </button>
    </div>

    <form id="bf-form" class="flex flex-col gap-4">
      <div class="w-full">
        <input id="bf-title" type="text" placeholder="title" required
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors" />
      </div>

      <div class="w-full">
        <input id="bf-url" type="text" placeholder="url" required
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors" />
      </div>

      <div class="w-full">
        <div class="flex gap-2">
          <input id="bf-cover" type="text" placeholder="cover image"
            class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors" />
          <button type="button" id="bf-upload-btn"
            class="shrink-0 px-3.5 py-2 cursor-pointer bg-paper-deeper hover:bg-paper-deep text-ink-soft hover:text-ink rounded-lg text-sm transition-colors disabled:opacity-50">
            Upload
          </button>
          <input id="bf-cover-file" type="file" accept="image/*" class="hidden" />
        </div>
      </div>

      ${
        categories.length
          ? `<div class="flex flex-wrap gap-1.5">${categories.map((c) => chipRadio(c.id, c.name)).join("")}</div>`
          : ""
      }

      <div class="w-full">
        <input id="bf-new-tags" type="text" placeholder="新增标签（逗号分隔）"
          class="px-3 py-2 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors mb-2" />
        ${
          tags.length
            ? `<div class="flex flex-wrap gap-1.5">${tags.map((t) => chipCheckbox(t.id, t.name)).join("")}</div>`
            : ""
        }
      </div>

      <textarea id="bf-desc" rows="2" placeholder="desc"
        class="w-full border border-line rounded-lg px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors resize-none"></textarea>

      <div id="bf-status" class="text-sm min-h-4.5"></div>

      <button type="submit" id="bf-submit"
        class="px-4 py-2.5 inline-flex items-center justify-center cursor-pointer bg-accent text-white font-medium text-sm rounded-lg shadow-sm hover:bg-accent-ink transition-colors disabled:opacity-50">
        保存
      </button>
    </form>
  `;

  document
    .getElementById("bf-open-options")!
    .addEventListener("click", onOpenOptions);

  const titleInput = document.getElementById("bf-title") as HTMLInputElement;
  const urlInput = document.getElementById("bf-url") as HTMLInputElement;
  const coverInput = document.getElementById("bf-cover") as HTMLInputElement;
  const uploadBtn = document.getElementById(
    "bf-upload-btn",
  ) as HTMLButtonElement;
  const fileInput = document.getElementById(
    "bf-cover-file",
  ) as HTMLInputElement;
  const newTagsInput = document.getElementById(
    "bf-new-tags",
  ) as HTMLInputElement;
  const descInput = document.getElementById("bf-desc") as HTMLTextAreaElement;
  const form = document.getElementById("bf-form") as HTMLFormElement;
  const submitBtn = document.getElementById("bf-submit") as HTMLButtonElement;
  const statusEl = document.getElementById("bf-status") as HTMLDivElement;

  titleInput.value = initial.title;
  urlInput.value = initial.url;
  coverInput.value = initial.coverImage;

  function setStatus(text: string, kind: "ok" | "error" | "info") {
    statusEl.textContent = text;
    statusEl.className =
      "text-sm min-h-[18px] " +
      (kind === "ok"
        ? "text-accent-ink"
        : kind === "error"
          ? "text-danger"
          : "text-ink-soft");
  }

  uploadBtn.addEventListener("click", () => {
    fileInput.click();
  });

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    if (!file.type.startsWith("image/")) {
      setStatus("请选择图片文件", "error");
      fileInput.value = "";
      return;
    }

    uploadBtn.disabled = true;
    uploadBtn.textContent = "上传中…";
    setStatus("正在上传图片…", "info");

    try {
      // 上传成功后 cover_image 的值固定为后端返回的 url；
      // 用户之后手动改动输入框，就又变回"用户自己填的图片 url"
      const { url } = await uploadCoverImage(config, file);
      coverInput.value = url;
      setStatus("图片上传成功", "ok");
    } catch (err) {
      setStatus(
        err instanceof Error ? `上传失败：${err.message}` : "图片上传失败",
        "error",
      );
    } finally {
      uploadBtn.disabled = false;
      uploadBtn.textContent = "Upload";
      fileInput.value = "";
    }
  });

  form.addEventListener("submit", async (e) => {
    e.preventDefault();

    const categoryRaw = (
      form.elements.namedItem("category_id") as RadioNodeList | null
    )?.value;
    const checkedTags = Array.from(
      form.querySelectorAll<HTMLInputElement>('input[name="tag_ids"]:checked'),
    );

    const payload = {
      title: titleInput.value.trim(),
      url: urlInput.value.trim(),
      cover_image: coverInput.value.trim(),
      category_id: categoryRaw ? Number(categoryRaw) : null,
      tag_ids: checkedTags.map((el) => Number(el.value)),
      new_tags: newTagsInput.value.trim(),
      desc: descInput.value.trim(),
    };

    const validationError = validateBookmarkForm(payload);
    if (validationError) {
      setStatus(validationError, "error");
      return;
    }

    submitBtn.disabled = true;
    setStatus("保存中…", "info");

    try {
      await createBookmark(config, payload);
      setStatus("已保存 ✓", "ok");
      onSaved();
    } catch (err) {
      submitBtn.disabled = false;
      setStatus(
        err instanceof Error ? err.message : "保存失败，未知错误",
        "error",
      );
    }
  });
}
