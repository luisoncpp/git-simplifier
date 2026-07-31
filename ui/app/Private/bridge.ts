import type { Bridge } from "./types.ts";

const NO_DESKTOP = "Desktop repository access unavailable. Run Git Simplifier as a Tauri app.";

export class TauriBridge implements Bridge {
  async invoke<T = unknown>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    const invoke = globalThis.__TAURI__?.core?.invoke;
    if (typeof invoke !== "function") throw new Error(NO_DESKTOP);
    return invoke(command, args) as Promise<T>;
  }

  async pickRepository(): Promise<string | null> {
    const open = globalThis.__TAURI__?.dialog?.open;
    if (typeof open !== "function") throw new Error("Native folder picker unavailable in browser mode.");
    return open({ directory: true, multiple: false, title: "Open Git repository" }) as Promise<string | null>;
  }
}

export class FixtureBridge implements Bridge {
  private readonly data: Record<string, unknown>;

  constructor(data: Record<string, unknown> = {}) {
    this.data = data;
  }

  async invoke<T = unknown>(command: string): Promise<T> {
    if (!(command in this.data)) throw new Error(`Fixture has no ${command}`);
    return this.data[command] as T;
  }

  async pickRepository(): Promise<string | null> {
    return null;
  }
}
