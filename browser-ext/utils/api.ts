import type { ExtensionConfig } from "./storage";

export interface Category {
  id: number;
  name: string;
}

export interface Tag {
  id: number;
  name: string;
}

export interface CreateNavPayload {
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
    if (body && typeof body.error === "string") return body.error;
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

export async function createNav(
  config: ExtensionConfig,
  payload: CreateNavPayload,
): Promise<{ id: number }> {
  const res = await fetch(`${config.serverUrl}/api/navs`, {
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
