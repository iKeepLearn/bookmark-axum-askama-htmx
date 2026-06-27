import { storage } from "#imports";

export interface ExtensionConfig {
  serverUrl: string;
  token: string;
}

export const serverUrlItem = storage.defineItem<string>("local:serverUrl", {
  fallback: "",
});

export const tokenItem = storage.defineItem<string>("local:apiToken", {
  fallback: "",
});

function stripTrailingSlash(url: string): string {
  return url.replace(/\/+$/, "");
}

export async function getConfig(): Promise<ExtensionConfig> {
  const [serverUrl, token] = await Promise.all([
    serverUrlItem.getValue(),
    tokenItem.getValue(),
  ]);
  return { serverUrl: stripTrailingSlash(serverUrl), token };
}

export async function setConfig(
  serverUrl: string,
  token: string,
): Promise<void> {
  await Promise.all([
    serverUrlItem.setValue(stripTrailingSlash(serverUrl)),
    tokenItem.setValue(token),
  ]);
}

export function isConfigured(config: ExtensionConfig): boolean {
  return Boolean(config.serverUrl && config.token);
}
