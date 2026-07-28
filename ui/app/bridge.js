export class TauriBridge {
  async invoke(command, args = {}) {
    const invoke = globalThis.__TAURI__?.core?.invoke;
    if (typeof invoke !== "function") throw new Error("Desktop repository access unavailable. Run Git Helper as a Tauri app.");
    return invoke(command, args);
  }

  async pickRepository() {
    const open = globalThis.__TAURI__?.dialog?.open;
    if (typeof open !== "function") throw new Error("Native folder picker unavailable in browser mode.");
    return open({ directory: true, multiple: false, title: "Open Git repository" });
  }
}

export class FixtureBridge {
  constructor(data = {}) { this.data = data; }
  async invoke(command) { if (!(command in this.data)) throw new Error(`Fixture has no ${command}`); return this.data[command]; }
  async pickRepository() { return null; }
}
