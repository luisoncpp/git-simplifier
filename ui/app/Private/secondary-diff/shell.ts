import { renderInto } from "../dom.ts";

const errorMessage = (error: unknown): string => {
  const message = (error as { message?: unknown } | null | undefined)?.message;
  return message == null ? String(error) : String(error);
};

export function renderApp(rootSelector: string, markup: string): void {
  const root = globalThis.document?.querySelector(rootSelector);
  if (root) renderInto(root, markup);
}

type ClickHandler<T> = (app: T, value: string, node?: HTMLElement) => unknown;

export function bindClickEvents<T extends { render(): void; state: { error: string } }>(
  app: T,
  handlers: Record<string, ClickHandler<T>>,
): void {
  const target = globalThis.document;
  if (!target) return;
  target.addEventListener("click", /*handleClick=*/ (event) => {
    const node = (event.target as HTMLElement | null)?.closest?.("[data-event]") as
      | (HTMLElement & { disabled?: boolean })
      | null;
    if (!node || node.disabled) return;
    const action = handlers[node.dataset.event ?? ""];
    if (!action) return;
    event.preventDefault();
    settle(app, action(app, node.dataset.value ?? "", node));
  });
}

export function listenForReload<T extends { render(): void; state: { error: string } }>(
  app: T,
  eventName: string,
  reload: () => unknown,
): void {
  const listen = globalThis.__TAURI__?.event?.listen;
  if (typeof listen !== "function") return;
  void listen(eventName, /*reloadSession=*/ () => {
    settle(app, reload());
  });
}

function settle<T extends { render(): void; state: { error: string } }>(
  app: T,
  result: unknown,
): void {
  if (!result || typeof (result as { catch?: unknown }).catch !== "function") return;
  (result as Promise<unknown>).catch((error: unknown) => {
    app.state.error = errorMessage(error);
    app.render();
  });
}

export async function runBusy<T extends { render(): void; state: { busy: boolean; error: string } }>(
  app: T,
  work: () => void | Promise<void>,
): Promise<void> {
  app.state.busy = true;
  app.state.error = "";
  app.render();
  try {
    await work();
  } catch (error) {
    app.state.error = errorMessage(error);
  } finally {
    app.state.busy = false;
    app.render();
  }
}
