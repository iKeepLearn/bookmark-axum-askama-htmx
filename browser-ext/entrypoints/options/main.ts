import "../../assets/tailwind.css";
import { getConfig, setConfig } from "../../utils/storage";
import { testConnection, generateToken } from "../../utils/api";

const app = document.getElementById("app")!;

app.innerHTML = `

  <div class="bg-white border border-line rounded-xl shadow-sm p-6">
    <form id="config-form" class="flex flex-col gap-5">
      <div class="w-full">
        <label class="block font-mono text-[10.5px] uppercase tracking-[0.08em] text-ink-faint mb-1.5">
          书签库地址
        </label>
        <input
          id="server-url"
          type="text"
          placeholder="https://your-domain.com"
          class="px-3.5 py-2.5 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
        <p class="text-ink-faint text-xs mt-1.5">不用加结尾的斜杠，比如 https://bm.example.com</p>
      </div>

      <div class="w-full">
        <label class="block font-mono text-[10.5px] uppercase tracking-[0.08em] text-ink-faint mb-1.5">
          用户名
        </label>
        <input
          id="username"
          type="text"
          autocomplete="username"
          placeholder="登录用户名"
          class="px-3.5 py-2.5 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
      </div>

      <div class="w-full">
        <label class="block font-mono text-[10.5px] uppercase tracking-[0.08em] text-ink-faint mb-1.5">
          密码
        </label>
        <input
          id="password"
          type="password"
          autocomplete="current-password"
          placeholder="登录密码"
          class="px-3.5 py-2.5 border border-line rounded-lg w-full text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
        />
      </div>

      <div class="w-full">
        <label class="block font-mono text-[10.5px] uppercase tracking-[0.08em] text-ink-faint mb-1.5">
          Token
        </label>
        <input
          id="token"
          type="text"
          readonly
          placeholder="点击下方「生成密钥」自动获取"
          class="px-3.5 py-2.5 border border-line rounded-lg w-full text-sm text-ink-soft bg-paper-deeper placeholder:text-ink-faint focus:outline-none transition-colors"
        />
        <p class="text-ink-faint text-xs mt-1.5">没有服务？访问 <a class="text-accent" href="https://github.com/ikeeplearn/bookmark-axum-askama-htmx" target="_blank">项目仓库</a> 按 README 进行部署</p>
      </div>

      <div id="status" class="text-sm min-h-4.5"></div>

      <div class="flex gap-2">
        <button
          type="submit"
          id="generate-btn"
          class="flex-1 px-4 py-2.5 cursor-pointer inline-flex items-center justify-center bg-accent text-white font-medium text-sm rounded-lg shadow-sm hover:bg-accent-ink transition-colors disabled:opacity-60"
        >
          生成密钥
        </button>
        <button
          type="button"
          id="test-btn"
          class="px-4 py-2.5 cursor-pointer text-ink-soft border border-line rounded-lg text-sm hover:bg-paper-deeper transition-colors"
        >
          测试连接
        </button>
      </div>
    </form>
  </div>
`;

const form = document.getElementById("config-form") as HTMLFormElement;
const serverUrlInput = document.getElementById(
  "server-url",
) as HTMLInputElement;
const usernameInput = document.getElementById("username") as HTMLInputElement;
const passwordInput = document.getElementById("password") as HTMLInputElement;
const tokenInput = document.getElementById("token") as HTMLInputElement;
const statusEl = document.getElementById("status") as HTMLDivElement;
const testBtn = document.getElementById("test-btn") as HTMLButtonElement;
const generateBtn = document.getElementById(
  "generate-btn",
) as HTMLButtonElement;

function setStatus(text: string, kind: "ok" | "error" | "info") {
  statusEl.textContent = text;
  statusEl.className =
    "text-sm min-h-4.5 " +
    (kind === "ok"
      ? "text-accent-ink"
      : kind === "error"
        ? "text-danger"
        : "text-ink-soft");
}

function normalizeOrigin(rawUrl: string): string | null {
  try {
    return new URL(rawUrl).origin + "/*";
  } catch {
    return null;
  }
}

function updateGenerateBtnLabel() {
  generateBtn.textContent = tokenInput.value.trim() ? "重新生成" : "生成密钥";
}

async function init() {
  const config = await getConfig();
  serverUrlInput.value = config.serverUrl;
  tokenInput.value = config.token;
  updateGenerateBtnLabel();
}

form.addEventListener("submit", async (e) => {
  e.preventDefault();

  const serverUrl = serverUrlInput.value.trim().replace(/\/+$/, "");
  const username = usernameInput.value.trim();
  const password = passwordInput.value;

  if (!serverUrl || !username || !password) {
    setStatus("服务器地址、用户名和密码都要填", "error");
    return;
  }

  const origin = normalizeOrigin(serverUrl);
  if (!origin) {
    setStatus(
      "服务器地址看起来不对，检查一下格式（比如 https://your-domain.com）",
      "error",
    );
    return;
  }

  // 必须在用户点击按钮触发的事件里直接申请权限，浏览器才会认为这是一次有效的用户操作
  const granted = await browser.permissions.request({ origins: [origin] });
  if (!granted) {
    setStatus("没有授权访问这个域名，插件没法连接到服务器", "error");
    return;
  }

  generateBtn.disabled = true;
  setStatus("正在获取密钥…", "info");

  const result = await generateToken({ serverUrl, username, password });

  if (!result.ok || !result.token) {
    generateBtn.disabled = false;
    setStatus(result.message ?? "获取密钥失败", "error");
    return;
  }

  tokenInput.value = result.token;
  await setConfig(serverUrl, result.token);
  passwordInput.value = "";
  setStatus("正在测试连接…", "info");
  const ok = await testConnection({ serverUrl, token: result.token });
  setStatus(
    ok
      ? "保存成功，连接正常"
      : "已保存，但连接测试没通过，检查地址和 Token 是否正确",
    ok ? "ok" : "error",
  );

  setStatus("密钥已获取并保存", "ok");
  generateBtn.disabled = false;
});

testBtn.addEventListener("click", async () => {
  const serverUrl = serverUrlInput.value.trim().replace(/\/+$/, "");
  const token = tokenInput.value.trim();
  if (!serverUrl || !token) {
    setStatus("服务器地址和 Token 都要填", "error");
    return;
  }
  setStatus("正在测试连接…", "info");
  const ok = await testConnection({ serverUrl, token });
  setStatus(
    ok ? "连接正常" : "连接失败，检查地址、Token，以及是否已授权访问该域名",
    ok ? "ok" : "error",
  );
});

init();
