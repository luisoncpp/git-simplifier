import type { Bridge, FetchProgressEvent } from "./types.ts";

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

  listen(event: string, handler: (payload: FetchProgressEvent) => void): void {
    const listen = globalThis.__TAURI__?.event?.listen;
    if (typeof listen !== "function") return;
    const subscribing = listen(event, /*forwardPayload=*/ (raw: unknown) => {
      handler((raw as { payload: FetchProgressEvent }).payload);
    });
    Promise.resolve(subscribing).catch(/*listenFailureIsNonFatal=*/ () => {});
  }
}

/** @public Test-only bridge; fixtures supply command responses. */
export class FixtureBridge implements Bridge {
  private readonly data: Record<string, unknown>;
  private readonly listeners = new Map<string, (payload: FetchProgressEvent) => void>();

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

  listen(event: string, handler: (payload: FetchProgressEvent) => void): void {
    this.listeners.set(event, handler);
  }

  /** @public Test hook: deliver a backend event to a registered listener. */
  emitEvent(event: string, payload: FetchProgressEvent): void {
    this.listeners.get(event)?.(payload);
  }
}
