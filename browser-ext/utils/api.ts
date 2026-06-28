import type { ExtensionConfig } from "./storage";

export interface Category {
  id: number;
  name: string;
}

export interface Tag {
  id: number;
  name: string;
}

export interface CreateBookmarkPayload {
  title: string;
  url: string;
  cover_image: string;
  category_id: number | null;
  tag_ids: number[];
  new_tags: string;
  desc: string;
}

class ApiError extends Error {}

function authHeaders(token: string): Record<string, string> {
  return { Authorization: `Bearer ${token}` };
}

async function parseErrorMessage(res: Response): Promise<string> {
  try {
    const body = await res.json();
    if (body && typeof body.message === "string") return body.message;
  } catch {
    // 响应体不是 JSON，忽略
  }
  return `请求失败 (HTTP ${res.status})`;
}

export async function fetchCategories(
  config: ExtensionConfig,
): Promise<Category[]> {
  const res = await fetch(`${config.serverUrl}/api/categories`, {
    headers: authHeaders(config.token),
  });
  if (!res.ok) throw new ApiError(await parseErrorMessage(res));
  return res.json();
}

export async function fetchTags(config: ExtensionConfig): Promise<Tag[]> {
  const res = await fetch(`${config.serverUrl}/api/tags`, {
    headers: authHeaders(config.token),
  });
  if (!res.ok) throw new ApiError(await parseErrorMessage(res));
  return res.json();
}

export async function createBookmark(
  config: ExtensionConfig,
  payload: CreateBookmarkPayload,
): Promise<{ id: number }> {
  const res = await fetch(`${config.serverUrl}/api/bookmark`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...authHeaders(config.token),
    },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new ApiError(await parseErrorMessage(res));
  return res.json();
}

export async function testConnection(
  config: ExtensionConfig,
): Promise<boolean> {
  try {
    const res = await fetch(`${config.serverUrl}/api/categories`, {
      headers: authHeaders(config.token),
    });
    return res.ok;
  } catch {
    return false;
  }
}

// utils/api.ts
export async function uploadCoverImage(
  config: ExtensionConfig,
  file: File,
): Promise<{ url: string; previewUrl: string }> {
  const formData = new FormData();
  formData.append("image", file);

  const res = await fetch(`${config.serverUrl}/api/upload`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${config.token}`, // 按你实际鉴权方式改
    },
    body: formData,
  });

  if (!res.ok) {
    let errorMessage = await parseErrorMessage(res);
    throw new Error(`上传失败 (${res.status}) (${errorMessage})`);
  }

  const data = await res.json();
  const url = data.image_url;
  const previewUrl = `${config.serverUrl}/${url}`;
  return { url, previewUrl }; // 按后端实际返回字段改
}

export async function generateToken({
  serverUrl,
  username,
  password,
}: {
  serverUrl: string;
  username: string;
  password: string;
}): Promise<{ ok: boolean; token?: string; message?: string }> {
  try {
    const res = await fetch(`${serverUrl}/api/token`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });

    if (!res.ok) {
      // 401/403 通常是账号密码错误
      const msg =
        res.status === 401 || res.status === 403
          ? "用户名或密码不对"
          : `请求失败（状态码 ${res.status}）`;
      return { ok: false, message: msg };
    }

    const data = await res.json();
    if (!data?.token) {
      return {
        ok: false,
        message: "服务器没有返回 token，检查接口格式是否一致",
      };
    }

    return { ok: true, token: data.token };
  } catch {
    return {
      ok: false,
      message: "网络请求失败，检查地址是否正确、服务是否可访问",
    };
  }
}
