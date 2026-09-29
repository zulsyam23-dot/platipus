use crate::codegen::components;
use crate::codegen::expression::{Scope, member_access, rewrite};
use crate::codegen::state::{
    is_global, is_persistent, is_reactive, scope_for, state_initializer, storage_scope,
};
use crate::codegen::statement::{declare_inputs, render_function};
use crate::ir::{IrComponent, IrModule, IrState};

pub fn render(module: &IrModule) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(PRELUDE);
    out.push('\n');
    out.push_str(&format!(
        "export const module_name = {:?};\n\n",
        module.name
    ));
    out.push_str("const $components = {};\n\n");
    for component in &module.components {
        out.push_str(&render_component(component));
        out.push('\n');
    }
    out.push_str(&render_api(module));
    out.push_str(&render_bootstrap(module));
    out
}

fn render_component(component: &IrComponent) -> String {
    let mut scope = scope_for(component);
    let channels = event_channels(&component.emits);
    for event in &component.emits {
        let key = event.strip_prefix("plt:").unwrap_or(event.as_str());
        scope.bind(event, member_access("events", key));
    }
    let mut out = String::new();
    out.push_str(&format!("function {}(inputs) {{\n", component.name));
    // The defaults are written into the caller's object so the closure over
    // `inputs` keeps seeing whatever the parent passes on the next render.
    out.push_str("  inputs = inputs ?? {};\n");
    out.push_str(&declare_inputs(&component.inputs));
    out.push('\n');
    out.push_str(&format!("  const events = {channels};\n"));
    out.push_str("  const s = {\n");
    for state in &component.states {
        out.push_str(&render_state(state, &scope));
    }
    out.push_str("  };\n");
    out.push_str("  const d = {};\n");
    for derived in &component.derived {
        out.push_str(&format!(
            "  d.{} = plt.computed(() => ({}), [{}], s, d);\n",
            derived.name,
            rewrite(&derived.value, &scope),
            derived
                .dependencies
                .iter()
                .map(|dependency| format!("{dependency:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for function in &component.functions {
        out.push_str("  ");
        out.push_str(&render_function(function, &scope));
        out.push('\n');
    }
    out.push_str("  const render = () => ");
    // A component body may hold several sibling roots, or none at all when it is
    // only a branch. Taking just the first element would drop everything after
    // it, and a body of only branches would render nothing, so every root is
    // emitted. More than one is wrapped in a tagless fragment, which the runtime
    // flattens: that keeps the roots as direct children of the parent instead
    // of inserting a wrapper element that would change the layout.
    let roots = components::render_items(&component.body, &scope);
    match roots.len() {
        0 => out.push_str("null"),
        1 => out.push_str(&roots[0]),
        _ => out.push_str(&format!("plt.el(null, {{}}, [{}])", roots.join(", "))),
    }
    out.push_str(";\n");
    out.push_str("  const vnode = render();\n");
    // A lifecycle handler declared on the component itself is called by the
    // runtime, not bound to a DOM event.
    let lifecycle = components::merge_handlers(&components::lifecycle_entries(
        &components::lifecycle(&component.handlers),
        &scope,
    ));
    match lifecycle {
        Some(lifecycle) => out.push_str(&format!("  const life = {lifecycle};\n")),
        None => out.push_str("  const life = {};\n"),
    }
    out.push_str("  return { s, d, events, render, vnode, life, inputs, inputDefaults };\n");
    out.push_str("}\n");
    out.push_str(&format!(
        "$components[{:?}] = {};\n",
        component.name, component.name
    ));
    out
}

fn render_state(state: &IrState, scope: &Scope) -> String {
    let value = rewrite(&state_initializer(state), scope);
    if is_global(&state.kind) {
        return format!(
            "    {}: plt.global({:?}, {}), // {}\n",
            state.name,
            state.name,
            value,
            storage_scope(&state.kind)
        );
    }
    if is_persistent(&state.kind) {
        return format!(
            "    {}: plt.persistent({:?}, {}), // {}\n",
            state.name,
            state.name,
            value,
            storage_scope(&state.kind)
        );
    }
    if is_reactive(state.kind) {
        return format!(
            "    {}: plt.signal({value}), // {}\n",
            state.name,
            storage_scope(&state.kind)
        );
    }
    format!("    {}: plt.ref({value}),\n", state.name)
}

fn event_channels(emits: &[String]) -> String {
    if emits.is_empty() {
        return "{}".to_string();
    }
    let entries = emits
        .iter()
        .map(|event| {
            let key = event.strip_prefix("plt:").unwrap_or(event.as_str());
            format!("{key:?}: plt.emitter({key:?})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{entries}}}")
}

fn render_api(module: &IrModule) -> String {
    let mut out = String::new();
    if module.apis.is_empty() {
        return out;
    }
    out.push_str("export const $api = {\n");
    for api in &module.apis {
        out.push_str(&format!("  {:?}: {{\n", api.name));
        for route in &api.routes {
            out.push_str(&format!(
                "    {:?}: plt.route({:?}, {:?}, {:?}),\n",
                route.name,
                route.method.as_str(),
                route.path,
                api.name
            ));
        }
        out.push_str("  },\n");
    }
    out.push_str("};\n\n");
    out
}

fn render_bootstrap(module: &IrModule) -> String {
    let app = match module.components.first() {
        Some(app) => app,
        None => return String::new(),
    };
    let mut out = String::new();
    out.push_str("export function mount(target) {\n");
    out.push_str("  if (target.__plt) plt.dispose(target.__plt);\n");
    out.push_str(&format!("  const instance = {}();\n", app.name));
    out.push_str("  instance.root = plt.host(instance);\n");
    out.push_str("  target.replaceChildren(instance.root);\n");
    out.push_str("  plt.bind(target, instance);\n");
    out.push_str("  return instance;\n");
    out.push_str("}\n\n");
    out.push_str("if (typeof document !== \"undefined\") {\n");
    out.push_str("  const root = document.getElementById(\"root\");\n");
    out.push_str("  if (root) mount(root);\n");
    out.push_str("}\n");
    out
}

const PRELUDE: &str = r#"const plt = (() => {
  const channels = new Map();

  // A state write must not re-render in the middle of a dispatch. A node can
  // carry more than one listener for one event -- an `on input` handler and the
  // listener a `bind` installs are the two that meet most often -- and a render
  // between them writes the node's value back from state, so the second
  // listener would read what the first one just undid. Writes made while a
  // dispatch is in flight therefore mark their instance, and the outermost
  // dispatch renders once when it ends.
  let batchDepth = 0;
  const dirtied = new Set();

  function beginBatch() {
    batchDepth += 1;
  }

  /**
   * Ends a transaction and renders whatever it dirtied. A render can dirty an
   * instance the batch had not queued -- a component's `on mount` writing state
   * while its parent is being patched is the ordinary case -- so the set is
   * drained until it is empty rather than flushed once and dropped.
   */
  function endBatch() {
    batchDepth -= 1;
    if (batchDepth > 0) return;
    while (dirtied.size > 0) {
      const queued = [...dirtied];
      dirtied.clear();
      for (const instance of queued) refresh(instance);
    }
  }

  /** Runs a listener as one transaction, so the writes it makes render once. */
  function transactional(handler) {
    return function (raw) {
      beginBatch();
      try {
        return handler.call(this, raw);
      } finally {
        endBatch();
      }
    };
  }

  function signal(initial) {
    let value = initial;
    const subscribers = new Set();
    const box = {
      get value() {
        return value;
      },
      set value(next) {
        if (Object.is(next, value)) return;
        value = next;
        for (const subscriber of [...subscribers]) subscriber(value);
      },
      subscribe(subscriber) {
        subscribers.add(subscriber);
        return () => subscribers.delete(subscriber);
      },
    };
    return box;
  }

  function computed(compute, dependencies, s, d) {
    const box = signal(undefined);
    box.stops = [];
    const recompute = () => {
      box.value = compute();
    };
    for (const name of dependencies) {
      const store = s[name] ?? d[name];
      if (store) box.stops.push(store.subscribe(recompute));
    }
    recompute();
    return box;
  }

  function ref(initial) {
    return { value: initial };
  }

  const globals = new Map();
  function global(name, initial) {
    if (!globals.has(name)) globals.set(name, signal(initial));
    return globals.get(name);
  }

  const persistence = new Map();
  function persistent(name, initial) {
    if (persistence.has(name)) return persistence.get(name);
    let restored = null;
    try {
      const raw = globalThis.localStorage?.getItem("plt:" + name);
      if (raw != null) restored = JSON.parse(raw);
    } catch (error) {
      restored = null;
    }
    const box = signal(restored ?? initial);
    box.subscribe((value) => {
      try {
        globalThis.localStorage?.setItem("plt:" + name, JSON.stringify(value));
      } catch (error) {
        void error;
      }
    });
    persistence.set(name, box);
    return box;
  }

function channelKey(name) {
    return name.startsWith("plt:") ? name.slice(4) : name;
  }

  // ---- browser API ----------------------------------------------------

  /**
   * `fetch(url)` resolves to a plain object rather than a raw `Response`, so a
   * handler can write it into a state and read `.status`, `.text`, or `.json`
   * without touching the browser API. Failures resolve too, with `ok: false`
   * and a message in `error`, so an uncaught rejection cannot take the program
   * down.
   */
  function fetch(url) {
    const fail = (message) => ({
      request: String(url),
      status: 0,
      ok: false,
      type: "error",
      text: "",
      json: null,
      error: String(message ?? "fetch failed"),
    });
    if (typeof globalThis.fetch !== "function") {
      return Promise.resolve(fail("fetch is unavailable"));
    }
    return globalThis
      .fetch(String(url))
      .then(async (response) => {
        let text = "";
        try {
          text = await response.text();
        } catch (error) {
          void error;
        }
        let json = null;
        try {
          json = JSON.parse(text);
        } catch (error) {
          void error;
        }
        return {
          request: String(url),
          status: response.status,
          ok: response.ok,
          type: response.type ?? "",
          text,
          json,
          error: "",
        };
      })
      .catch((error) => fail(error?.message ?? error));
  }

  function clipboardWrite(text) {
    if (globalThis.navigator?.clipboard?.writeText == null) {
      return Promise.resolve(false);
    }
    return globalThis.navigator.clipboard
      .writeText(String(text))
      .then(() => true)
      .catch(() => false);
  }

  function clipboardRead() {
    if (globalThis.navigator?.clipboard?.readText == null) {
      return Promise.resolve("");
    }
    return globalThis.navigator.clipboard.readText().catch(() => "");
  }

  /**
  * `openFile()` resolves to `{ name, size, type, text, cancelled }`. A
  * cancelled picker returns the empty result with `cancelled: true`. The File
  * System Access picker is preferred; otherwise a file input is created.
   */
  function openFile() {
    const blank = { name: "", size: 0, type: "", text: "", cancelled: true };
    const read = (file) =>
      file
        .text()
        .then((text) => ({
          name: file.name,
          size: file.size,
          type: file.type,
          text,
          cancelled: false,
        }))
        .catch(() => ({
          name: file.name,
          size: file.size,
          type: file.type,
          text: "",
          cancelled: false,
        }));
    const pick = async () => {
      if (typeof globalThis.showOpenFilePicker === "function") {
        const handles = await globalThis.showOpenFilePicker();
        const file = await handles[0]?.getFile();
        return file ? read(file) : null;
      }
      if (typeof document !== "undefined" && document.createElement) {
        const input = document.createElement("input");
        input.type = "file";
        return new Promise((resolve) => {
          let settled = false;
          const finish = (result) => {
            if (settled) return;
            settled = true;
            input.removeEventListener?.("change", onChange);
            input.removeEventListener?.("cancel", onCancel);
            resolve(result);
          };
          const onChange = () => {
            const file = input.files?.[0];
            if (file) void read(file).then(finish);
            else finish(blank);
          };
          const onCancel = () => finish(blank);
          input.addEventListener("change", onChange);
          input.addEventListener("cancel", onCancel);
          input.click();
        });
      }
      return null;
    };
    return pick().catch(() => blank);
  }

  const socketHandles = new Map();

  /**
   * `webSocket(url)` resolves after the connection opens or fails, returning a
   * handle `{ url, status, send(data), close() }`. Incoming messages are pulled
   * with `receive(socket)`, which resolves to the next message or waits for one.
   * A pending connection is shared per URL.
   */
  function webSocket(url) {
    const key = String(url);
    if (socketHandles.has(key)) return socketHandles.get(key);
    const queue = [];
    const waiters = [];
    const handle = {
      url: key,
      status: "connecting",
      send(data) {
        if (handle.status !== "open") return false;
        try {
          handle._ws?.send(String(data));
          return true;
        } catch (error) {
          void error;
          return false;
        }
      },
      close() {
        try {
          handle._ws?.close();
        } catch (error) {
          void error;
        }
      },
      _queue: queue,
      _waiters: waiters,
    };
    let settleReady;
    let readySettled = false;
    let timeoutId = null;
    let socket = null;
    let timedOut = false;
    const ready = new Promise((resolve) => {
      settleReady = resolve;
    });
    const settle = (status) => {
      if (readySettled) return;
      readySettled = true;
      handle.status = status;
      if (timeoutId != null) clearTimeout(timeoutId);
      if (status !== "open" && socketHandles.get(key) === ready) {
        socketHandles.delete(key);
      }
      settleReady(handle);
    };
    socketHandles.set(key, ready);
    const Socket = globalThis.WebSocket;
    if (typeof Socket !== "function") {
      settle("unavailable");
      return ready;
    }
    timeoutId = setTimeout(() => {
      timedOut = true;
      settle("timeout");
      if (socketHandles.get(key) === ready) socketHandles.delete(key);
      try {
        socket?.close();
      } catch (error) {
        void error;
      }
    }, 10_000);
    try {
      socket = new Socket(key);
      handle._ws = socket;
      socket.onopen = () => settle("open");
      socket.onmessage = (event) => {
        const text =
          typeof event?.data === "string" ? event.data : String(event?.data ?? "");
        if (waiters.length > 0) waiters.shift()(text);
        else queue.push(text);
      };
      socket.onerror = () => {
        if (readySettled) {
          handle.status = "error";
          if (socketHandles.get(key) === ready) socketHandles.delete(key);
        }
        else settle("error");
      };
      socket.onclose = () => {
        if (!readySettled) settle("closed");
        else if (!timedOut) handle.status = "closed";
        while (waiters.length > 0) waiters.shift()(null);
        if (socketHandles.get(key) === ready) socketHandles.delete(key);
      };
    } catch (error) {
      void error;
      settle("error");
    }
    return ready;
  }

  function receive(socket) {
    if (socket == null) return Promise.resolve(null);
    const queue = socket._queue ?? [];
    if (queue.length > 0) return Promise.resolve(queue.shift());
    if (socket.status !== "open") return Promise.resolve(null);
    return new Promise((resolve) => {
      (socket._waiters ??= []).push(resolve);
    });
  }

  /** The synchronous browser storage the `local` and `session` backends use. */
  function storageBackend(backend) {
    if (backend === "session") return globalThis.sessionStorage;
    if (backend === "local") return globalThis.localStorage;
    return null;
  }

  let idbPromise = null;
  function idb() {
    if (globalThis.indexedDB == null) return Promise.resolve(null);
    idbPromise ??= new Promise((resolve, reject) => {
      const request = globalThis.indexedDB.open("platipus", 1);
      request.onupgradeneeded = () => request.result.createObjectStore("kv");
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    return idbPromise;
  }

  function idbRequest(method, key, value) {
    return idb().then((db) => {
      if (db == null) return method === "get" ? null : false;
      return new Promise((resolve, reject) => {
        const tx = db.transaction("kv", method === "get" ? "readonly" : "readwrite");
        const objectStore = tx.objectStore("kv");
        const request =
          method === "get"
            ? objectStore.get(key)
            : method === "put"
              ? objectStore.put(value, key)
              : objectStore.delete(key);
        request.onsuccess = () => {
          if (method === "get") resolve(request.result ?? null);
          else resolve(true);
        };
        request.onerror = () => reject(request.error);
      });
    });
  }

  /** `store/load/drop` reach local, session, or indexed storage by backend. */
  function store(backend, key, value) {
    if (backend === "indexed") return idbRequest("put", key, value);
    const storage = storageBackend(backend);
    if (!storage) return false;
    try {
      storage.setItem(String(key), JSON.stringify(value));
      return true;
    } catch (error) {
      void error;
      return false;
    }
  }

  function load(backend, key) {
    if (backend === "indexed") return idbRequest("get", key);
    const storage = storageBackend(backend);
    if (!storage) return null;
    try {
      const raw = storage.getItem(String(key));
      return raw == null ? null : JSON.parse(raw);
    } catch (error) {
      void error;
      return null;
    }
  }

  function drop(backend, key) {
    if (backend === "indexed") return idbRequest("delete", key);
    const storage = storageBackend(backend);
    if (!storage) return;
    try {
      storage.removeItem(String(key));
    } catch (error) {
      void error;
    }
  }

  // ---- advanced UI ---------------------------------------------------

  /**
   * `canvas(id)` resolves the canvas carrying that `id` into an opaque handle
   * the drawing builtins take. The canvas has to be in the document for the
   * lookup to succeed, so it is typically called from a handler.
   */
  function canvas(id) {
    if (typeof document === "undefined" || typeof document.getElementById !== "function") {
      return null;
    }
    const node = document.getElementById(String(id));
    return node ? { _node: node } : null;
  }

  /** The 2D context behind a canvas handle, created lazily and shared. */
  function canvas2d(handle) {
    const node = handle?._node;
    if (!node || node.nodeType !== 1) return null;
    if (node.__ctx2d) return node.__ctx2d;
    const ctx = typeof node.getContext === "function" ? node.getContext("2d") : null;
    node.__ctx2d = ctx;
    return ctx;
  }

  function fill(handle, x, y, w, h, color) {
    const ctx = canvas2d(handle);
    if (!ctx) return false;
    try {
      if (color != null) ctx.fillStyle = String(color);
      ctx.fillRect(Number(x) || 0, Number(y) || 0, Number(w) || 0, Number(h) || 0);
      return true;
    } catch (error) {
      void error;
      return false;
    }
  }

  function clear(handle, color) {
    const node = handle?._node;
    const ctx = canvas2d(handle);
    if (!ctx) return false;
    // The drawing surface, not a fixed guess: a canvas is as large as its
    // `width`/`height` say, and a 1000x1000 clear either leaves a border behind
    // on a larger canvas or wastes a fill on a small one.
    const width = Number(node?.width) || 0;
    const height = Number(node?.height) || 0;
    try {
      // `clearRect` ignores `fillStyle`, so a requested colour means "repaint the
      // surface", which is the only way to give a cleared canvas a background.
      if (color != null) {
        ctx.fillStyle = String(color);
        ctx.fillRect(0, 0, width, height);
      } else {
        ctx.clearRect(0, 0, width, height);
      }
      return true;
    } catch (error) {
      void error;
      return false;
    }
  }

  function drawText(handle, text, x, y) {
    const ctx = canvas2d(handle);
    if (!ctx) return false;
    try {
      ctx.fillText(String(text), Number(x) || 0, Number(y) || 0);
      return true;
    } catch (error) {
      void error;
      return false;
    }
  }

  /**
   * `nextFrame()` resolves on the next animation frame. It uses
   * `requestAnimationFrame` when the platform offers one and falls back to a
   * macrotask, so an `async fn` can act as an animation loop by awaiting it.
   */
  function nextFrame() {
    const schedule =
      typeof globalThis.requestAnimationFrame === "function"
        ? globalThis.requestAnimationFrame
        : (callback) => setTimeout(callback, 0);
    return new Promise((resolve) => schedule(() => resolve()));
  }

  /** `wait(ms)` resolves after the delay, letting async composition pause. */
  function wait(ms) {
    return new Promise((resolve) =>
      setTimeout(resolve, Math.max(0, Number(ms) || 0)),
    );
  }

  /** `exec(cmd)` runs a browser editing command on the focused editor. */
  function exec(cmd) {
    if (typeof document !== "undefined" && typeof document.execCommand === "function") {
      try {
        return document.execCommand(String(cmd));
      } catch (error) {
        void error;
        return false;
      }
    }
    return false;
  }

  /** `selection()` reads the focused editor's caret offsets. */
  function selection() {
    const fallback = { start: -1, end: -1 };
    if (typeof document === "undefined" || typeof document.getSelection !== "function") {
      return fallback;
    }
    const current = document.getSelection();
    if (!current || current.rangeCount === 0) return fallback;
    try {
      const range = current.getRangeAt(0);
      return { start: range.startOffset, end: range.endOffset };
    } catch (error) {
      void error;
      return fallback;
    }
  }

  /**
   * `indent()` inserts an indentation at the caret of the focused textarea and
   * notifies its listeners, which is the "Tab inserts indentation" keymap of a
   * minimal code editor.
   */
  function indent() {
    const active = typeof document?.activeElement === "object" ? document.activeElement : null;
    if (!active || active.tagName !== "TEXTAREA") return false;
    try {
      const start = active.selectionStart ?? String(active.value).length;
      const end = active.selectionEnd ?? start;
      const before = String(active.value).slice(0, start);
      const after = String(active.value).slice(end);
      active.value = before + "  " + after;
      active.setSelectionRange?.(start + 2, start + 2);
      if (typeof active.dispatchEvent === "function") {
        active.dispatchEvent(new Event("input"));
      } else {
        active.dispatch?.("input", { type: "input", target: active });
      }
      return true;
    } catch (error) {
      void error;
      return false;
    }
  }

  /** `sortBy(list, key, dir)` returns a new array, sorted asc or desc. */
  function sortBy(list, key, direction) {
    const items = Array.from(list ?? []);
    const dir = direction === "desc" ? -1 : 1;
    const at = (item) => {
      const value = key == null || key === "" ? item : item?.[key];
      return typeof value === "string" ? value.toLocaleLowerCase() : value;
    };
    items.sort((a, b) => {
      const left = at(a);
      const right = at(b);
      if (left == null) return dir;
      if (right == null) return -dir;
      if (left < right) return -dir;
      if (left > right) return dir;
      return 0;
    });
    return items;
  }

  /** `page(items, index, size)` returns one window of the list. */
  function page(items, index, size) {
    const window = Math.max(1, Number(size) || 1);
    const pageIndex = Math.max(0, Number(index) || 0);
    return Array.from(items ?? []).slice(
      pageIndex * window,
      pageIndex * window + window,
    );
  }

  /** Keyword highlighting for the `CodeEditor` overlay. */
  function highlightHtml(text) {
    const escaped = String(text ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    return escaped.replace(
      /(\b(?:state|derived|input|fn|async|component|app|for|in|if|else|on|return|await|true|false|null)\b)/g,
      '<span class="plt-tok">$1</span>',
    );
  }

  /**
   * A `CodeEditor` with a `highlight` attribute gets a `<pre>` overlay that
   * mirrors the value as highlighted HTML.
   *
   * The two are put in a shell element rather than left as siblings. An overlay
   * only reads as an overlay if the two share one box: as siblings the
   * highlighted copy lands under the editable one, so the same source is on screen
   * twice and the control is twice as tall as it looks. The shell is what the
   * base stylesheet positions the `<pre>` against, and the editor itself stays a
   * textarea, so the vnode keeps pointing at the node the program bound to.
   */
  function ensureCodeOverlay(node) {
    if (node.tagName !== "TEXTAREA") return;
    if (node.getAttribute("highlight") == null) return;
    if (!node.__pltCode) {
      const shell = document.createElement("div");
      shell.className = "plt-code-shell";
      const overlay = document.createElement("pre");
      overlay.className = "plt-code";
      node.__pltShell = shell;
      node.__pltCode = overlay;
    }
    // Props are applied while the node is still detached, so the shell cannot be
    // placed until the node has a parent. This runs again on mount for that.
    //
    // The shell takes the editor's place first and the editor moves into it
    // after. Moving the editor into an unplaced shell would re-parent it to the
    // shell, and the shell would then be inserted into itself.
    const shell = node.__pltShell;
    if (!shell.parentNode && node.parentNode) {
      const parent = node.parentNode;
      parent.insertBefore(shell, node);
      shell.append(node, node.__pltCode);
    }
    syncCodeOverlay(node);
  }

  function syncCodeOverlay(node) {
    if (!node.__pltCode) return;
    node.__pltCode.innerHTML = highlightHtml(
      node.value != null ? node.value : node.textContent,
    );
  }

function emitter(name) {
    const key = channelKey(name);
    const listeners = new Set();
    if (!channels.has(key)) channels.set(key, []);
    const handle = {
      name: key,
      emit(payload) {
        for (const listener of [...listeners]) listener(payload);
      },
      on(listener) {
        listeners.add(listener);
        return () => listeners.delete(listener);
      },
    };
    channels.get(key).push(handle);
    return handle;
  }

function emit(target, payload) {
    if (target && typeof target.emit === "function") {
      target.emit(payload);
      return;
    }
    for (const entry of channels.get(channelKey(String(target))) ?? []) entry.emit(payload);
  }

  // ---- event objects --------------------------------------------------

  /**
   * The payload a drag carries. A browser hands it over on `dataTransfer`; a
   * synthetic event may carry it directly. Both spellings resolve to one value,
   * and a drag that has nothing to give resolves to `undefined` rather than
   * throwing, because `getData` is only readable inside a real drop handler.
   */
  function transferData(raw) {
    if (raw?.data != null) return raw.data;
    const transfer = raw?.dataTransfer;
    if (!transfer || typeof transfer.getData !== "function") return undefined;
    try {
      const text = transfer.getData("text/plain") || transfer.getData("text");
      return text === "" ? undefined : text;
    } catch (error) {
      void error;
      return undefined;
    }
  }

  /**
   * Normalizes a platform event into the object a handler reads. The `value`
   * field carries the current field value (checked for a checkbox or radio),
   * `key` the pressed key, and `position` the pointer coordinates, so a handler
   * never has to reach for a browser object. `data` carries what a drag dropped,
   * so `on drop` reads `event.data` instead of walking `dataTransfer`.
   * `stopPropagation` and `preventDefault` are forwarded to the raw event,
   * which is kept on `originalEvent` when it exists.
   */
  function event(raw) {
    if (raw && raw.__plt) return raw;
    const target = raw?.target ?? null;
    let value;
    if (target != null && target.type === "file") {
      // A file control's own `value` is a path the browser invents, which is
      // never what a program means, so the name of the file it was handed is
      // the value. Without a file list the attribute stands in for it, which is
      // what the shim and a hand-built event carry.
      value = target.files?.[0]?.name ?? target.value ?? "";
    } else if (target != null && (target.type === "checkbox" || target.type === "radio")) {
      value = target.checked;
    } else if (target != null && target.getAttribute?.("contenteditable") === "true") {
      value = target.textContent;
    } else if (target != null && "value" in target) {
      value = target.value;
    }
    const normalized = {
      value,
      key: raw?.key,
      data: transferData(raw),
      position:
        raw?.clientX != null
          ? { x: raw.clientX, y: raw.clientY }
          : { x: 0, y: 0 },
      target,
      type: raw?.type,
      originalEvent: raw,
      stopPropagation() {
        raw?.stopPropagation?.();
      },
      preventDefault() {
        raw?.preventDefault?.();
      },
    };
    Object.defineProperty(normalized, "__plt", { value: true });
    return normalized;
  }

  // ---- virtual nodes -------------------------------------------------

  function vnode(tag, props, children) {
    return { tag, props: props ?? {}, children: children ?? [], key: null, node: null };
  }

  function el(tag, props, children) {
    return vnode(tag, props, children);
  }

  function fragment(children) {
    return vnode(null, {}, children);
  }

  /** `if` in a template: only the taken branch is built. */
  function branch(condition, then, otherwise) {
    const chosen = condition ? then() : otherwise ? otherwise() : null;
    return chosen ?? fragment([]);
  }

  /** `for` in a template: each item becomes a keyed child. */
  function each(list, item) {
    const items = Array.from(list ?? []);
    const children = items.map((value, index) => {
      const node = item(value, index) ?? null;
      if (node) node.key = String(value?.key ?? value?.id ?? index);
      return node;
    });
    return fragment(children);
  }

  function twoWay(read, write) {
    return { read, write };
  }

  function route(method, path, api) {
    return { method, path, api };
  }

  // ---- reconciliation ------------------------------------------------

  function keyOf(vnode, index) {
    return vnode.key ?? `i${index}`;
  }

  function sameKind(a, b) {
    if (a.tag !== b.tag) return false;
    if (a.component || b.component) return a.component === b.component;
    return true;
  }

  /**
   * Reconciles two child lists. Iterating backwards keeps `anchor` pointing at
   * the node that must follow, so moved nodes are only touched when they are
   * actually out of order.
   */
  function patchList(parent, oldList, newList) {
    const pool = new Map();
    oldList.forEach((vnode, index) => pool.set(keyOf(vnode, index), vnode));
    let anchor = null;
    const placed = [];
    for (let index = newList.length - 1; index >= 0; index -= 1) {
      const vnode = newList[index];
      const key = keyOf(vnode, index);
      const old = pool.get(key) ?? null;
      pool.delete(key);
      const nodes = patch(parent, old, vnode);
      for (let position = nodes.length - 1; position >= 0; position -= 1) {
        const node = nodes[position];
        if (node.parentNode !== parent || node.nextSibling !== anchor) {
          parent.insertBefore(node, anchor);
        }
        anchor = node;
        placed.unshift(node);
      }
    }
    for (const leftover of pool.values()) remove(leftover);
    return placed;
  }

  function remove(vnode) {
    if (vnode == null) return;
    // The handlers to tear down live on the virtual node, not on the DOM node
    // it produced, and the node itself has to be torn down too, not only what
    // is below it.
    destroy(vnode);
    for (const node of rootsOf(vnode)) node.node?.remove();
  }

  /** The top-level virtual nodes a subtree occupies, in document order. */
  function rootsOf(vnode) {
    if (!vnode) return [];
    // A child that has not been instantiated yet is returned as the root itself,
    // so patching it is what builds the instance. Returning an empty list for it
    // would drop the whole subtree, which is how a component used as the app's
    // only root rendered nothing at all.
    if (vnode.component) return vnode.instance ? rootsOf(vnode.instance.vnode) : [vnode];
    if (vnode.tag == null) return vnode.children.flatMap(rootsOf);
    return [vnode];
  }

  /** Patches one virtual node and returns the DOM nodes it occupies. */
  function patch(parent, oldV, newV) {
    if (newV == null) {
      remove(oldV);
      return [];
    }
    if (newV.component) {
      // An instance belongs to the component that made it, so a node that
      // turned into a different component cannot keep the old instance.
      const reusable = oldV?.component === newV.component ? oldV.instance : null;
      if (oldV != null && reusable == null) remove(oldV);
      const instance = ensureInstance(newV, reusable, parent);
      if (reusable != null) {
        const next = instance.render();
        patchList(parent, rootsOf(instance.vnode), rootsOf(next));
        instance.vnode = next;
        instance.life?.update?.();
      }
      for (const node of rootsOf(instance.vnode)) enter(node);
      return rootsOf(instance.vnode).map((node) => node.node).filter(Boolean);
    }
    if (newV.tag == null) {
      // A fragment holds no node of its own, so its children are compared
      // against each other where the fragment stands. An empty fragment is how
      // a branch that was not taken is written, so it also has to take away
      // whatever the taken branch left behind. A component counts as
      // occupying nodes even though it has no tag of its own: it produces them
      // through its instance, and `remove` tears the instance down. Testing
      // only `tag` would leave the whole subtree of a section that swapped for
      // another one sitting in the document.
      const occupied = oldV != null && (oldV.tag != null || oldV.component != null);
      if ((newV.children ?? []).length === 0 && occupied) {
        remove(oldV);
        newV.node = null;
        return [];
      }
      const placed = patchList(parent, oldV?.children ?? [], newV.children ?? []);
      newV.node = oldV?.node ?? null;
      return placed;
    }
    if (oldV == null || !sameKind(oldV, newV)) {
      const node = document.createElement(newV.tag);
      createProps(node, newV.props);
      newV.node = node;
      // The node exists but is not in the document yet, so `create` runs before
      // the children are placed and before `mount`.
      fire(newV, "create");
      patchList(node, [], newV.children ?? []);
      if (oldV) remove(oldV);
      return [node];
    }
    const node = oldV.node;
    newV.node = node;
    // The node survives the re-render, so it is updated rather than created.
    fire(newV, "update");
    updateProps(node, oldV.props, newV.props);
    // Both sides are child lists, so the children are reconciled against the
    // children the reused node already has.
    patchList(node, oldV.children ?? [], newV.children ?? []);
    return [node];
  }

function createProps(node, props) {
    applyAttrs(node, {}, props.attrs ?? {});
    applyDom(node, props.dom ?? {});
    if (props.text != null) node.textContent = String(props.text);
    applyEvents(node, props.on ?? {});
    applyBinds(node, props.bind ?? {});
    ensureCodeOverlay(node);
  }

  function updateProps(node, oldProps, props) {
    applyAttrs(node, oldProps.attrs ?? {}, props.attrs ?? {});
    applyDom(node, props.dom ?? {});
    if (props.text != null && props.text !== oldProps.text) {
      node.textContent = String(props.text);
    }
    applyEvents(node, props.on ?? {});
    applyBinds(node, props.bind ?? {});
    syncCodeOverlay(node);
  }

  function applyAttrs(node, oldAttrs, attrs) {
    for (const name of Object.keys(oldAttrs)) {
      if (!(name in attrs)) node.removeAttribute(name);
    }
    for (const [name, value] of Object.entries(attrs)) {
      if (value == null || value === false) node.removeAttribute(name);
      else node.setAttribute(name, value === true ? "" : String(value));
    }
  }

function applyDom(node, dom) {
    for (const [name, value] of Object.entries(dom)) {
      const contenteditable = node.getAttribute?.("contenteditable") === "true";
      if (contenteditable && name === "value") {
        // A contenteditable box holds its text in `textContent`, and writing it
        // back throws the caret to the start. The value only comes from state
        // the node itself reported, so it is written when the two differ and
        // left alone when they already agree -- which is every re-render.
        if (node.textContent !== value) node.textContent = value;
      } else if (name in node) {
        node[name] = value;
      } else {
        node.setAttribute(name, value == null ? "" : String(value));
      }
    }
  }

  /**
   * Subscribes one callback to an event on a node.
   *
   * Every callback on the same event shares a single DOM listener, so a whole
   * dispatch is one transaction instead of one per callback. It has to be: an
   * `on input` handler and a `bind` listen for the same event, and if each
   * opened its own transaction then the render at the end of the first would
   * write the node's value back from state before the second had read what the
   * user typed. One listener per event also means a re-render cannot leave a
   * node carrying one listener per render.
   *
   * `name` is the subscription slot, not the event: the same event can be
   * subscribed under several names, and only one callback per name is live.
   */
  function subscribe(node, key, name, callback, capture) {
    const groups = (node.__pltGroups ??= new Map());
    // A capture and a bubble callback on the same event cannot share a
    // listener, so the phase is part of the group identity.
    const id = capture ? `${key}!` : key;
    let group = groups.get(id);
    if (!group) {
      const members = new Map();
      const listener = function (raw) {
        beginBatch();
        try {
          for (const [member, memberCallback] of [...members]) {
            // A callback that unsubscribes another one does not stop it running
            // in the dispatch already under way, but it is gone by the next.
            if (members.has(member)) memberCallback(raw);
          }
        } finally {
          endBatch();
        }
      };
      group = { key, capture, listener, members };
      groups.set(id, group);
      node.addEventListener(key, listener, capture);
    }
    group.members.set(name, callback);
    return group;
  }

  /** The subscription recorded under `name`, or null when there is none. */
  function subscriptionOf(node, name) {
    for (const group of (node.__pltGroups ?? new Map()).values()) {
      if (!group.members.has(name)) continue;
      return { group, callback: group.members.get(name) };
    }
    return null;
  }

  /** Every name a node currently holds a subscription under. */
  function subscriptionNames(node) {
    const names = [];
    for (const group of (node.__pltGroups ?? new Map()).values()) {
      for (const name of group.members.keys()) names.push(name);
    }
    for (const name of Object.keys(node.__pltSwipe ?? {})) names.push(`swipe:${name}`);
    return names;
  }

  /**
   * Drops one subscription, and the DOM listener with it once nothing else is
   * using that group. `swipe` keeps its own pair, because the event it stands
   * for has no DOM name of its own.
   */
  function unsubscribe(node, name) {
    const swipe = node.__pltSwipe?.[name];
    if (swipe) {
      node.removeEventListener("pointerdown", swipe.down, swipe.capture);
      node.removeEventListener("pointerup", swipe.up, swipe.capture);
      delete node.__pltSwipe[name];
      return;
    }
    for (const [id, group] of [...(node.__pltGroups ?? new Map())]) {
      if (!group.members.delete(name)) continue;
      if (group.members.size > 0) continue;
      node.removeEventListener(group.key, group.listener, group.capture);
      node.__pltGroups.delete(id);
      return;
    }
  }

  /** Drops every subscription a node holds, listeners included. */
  function unsubscribeAll(node) {
    for (const name of subscriptionNames(node)) unsubscribe(node, name);
    node.__pltGroups = null;
    node.__pltSwipe = null;
  }

  function applyEvents(node, handlers) {
    for (const [name, spec] of Object.entries(handlers)) {
      // A non-bubbling event is emitted as a [handler, true] descriptor.
      const capture = Array.isArray(spec);
      const handler = capture ? spec[0] : spec;
      const current = subscriptionOf(node, `on:${name}`);
      if (current?.callback === handler && current?.group.capture === capture) continue;
      // A re-render hands over a fresh closure, so the subscription is replaced
      // rather than added to.
      unsubscribe(node, `on:${name}`);
      if (channelKey(name) === "swipe") {
        // `swipe` is not a platform event; the runtime derives it from
        // pointer movement on the element itself.
        listenForSwipe(node, name, handler, capture);
        continue;
      }
      subscribe(node, channelKey(name), `on:${name}`, handler, capture);
    }
    // A handler that no longer appears is no longer subscribed.
    for (const name of subscriptionNames(node)) {
      if (!name.startsWith("on:") || name.slice(3) in handlers) continue;
      unsubscribe(node, name);
    }
  }

  /**
   * Turns the pointer down/up pair on an element into a `swipe` handler. A swipe
   * needs 24px of travel along one axis; the payload carries the travelled
   * vector and the dominant direction, in the same object a custom event
   * delivers.
   */
  function listenForSwipe(node, name, handler, capture) {
    const down = (event) => {
      node.__swipeStart = { x: event.clientX ?? 0, y: event.clientY ?? 0 };
    };
    const up = transactional((event) => {
      const start = node.__swipeStart;
      node.__swipeStart = null;
      if (!start) return;
      const dx = (event.clientX ?? 0) - start.x;
      const dy = (event.clientY ?? 0) - start.y;
      if (Math.max(Math.abs(dx), Math.abs(dy)) < 24) return;
      const dir =
        Math.abs(dx) >= Math.abs(dy)
          ? dx > 0
            ? "right"
            : "left"
          : dy > 0
              ? "down"
              : "up";
      handler({ dx, dy, dir });
    });
    (node.__pltSwipe ??= {})[name] = { down, up, capture };
    node.addEventListener("pointerdown", down, capture);
    node.addEventListener("pointerup", up, capture);
  }

  function applyBinds(node, binds) {
    for (const [name, link] of Object.entries(binds)) {
      // A scroll-position link (scrollTop/scrollLeft) follows the scroll event
      // rather than input, so its state tracks the element's own scrolling.
      // A contenteditable box has no `value`; its text lives in `textContent`.
      const contenteditable = node.getAttribute?.("contenteditable") === "true";
      const property = contenteditable ? "textContent" : name;
      const key = name.startsWith("scroll") ? "scroll" : "input";
      // The bind reads the node when the dispatch is over rather than when this
      // callback is reached, because an `on input` handler may write state in
      // between and the render that follows would otherwise put the node's
      // value back before the read.
      if (!subscriptionOf(node, `bind:${name}`)) {
        subscribe(node, key, `bind:${name}`, () => link.write(node[property]), false);
      }
      // Seed the field and follow outside changes, without fighting an
      // in-flight scroll: a field that already holds the linked value is left
      // alone.
      const target = link.read().value;
      if (node[property] !== target) node[property] = target;
    }
    for (const name of subscriptionNames(node)) {
      if (!name.startsWith("bind:") || name.slice(5) in binds) continue;
      unsubscribe(node, name);
    }
  }

  // ---- lifecycle ----------------------------------------------------

  /**
   * Calls a lifecycle handler if the virtual node declares one. `create` runs
   * when the node exists, `mount` once it has been placed, `update` when the
   * node survives a re-render, and `destroy` before it leaves the document.
   */
  function fire(vnode, name) {
    vnode?.props?.life?.[name]?.();
  }

  /** Mounts a node and everything below it, once. */
  function enter(vnode) {
    if (vnode == null) return;
    if (vnode.component) {
      enter(vnode.instance?.vnode);
      return;
    }
    if (vnode.tag != null) {
      if (vnode.node.__pltEntered) return;
      vnode.node.__pltEntered = true;
      fire(vnode, "mount");
      // The highlight overlay rides along next to the editor; the node is in
      // the document here, so the overlay can be placed beside it.
      ensureCodeOverlay(vnode.node);
    }
    for (const child of vnode.children) enter(child);
  }

  /**
   * Destroys a node and everything below it. Handlers run while the nodes are
   * still in the document, so they can read the DOM, and the listeners and
   * subscriptions each node holds are released with it.
   */
  function destroy(vnode) {
    if (vnode == null) return;
    if (vnode.component) {
      dispose(vnode.instance);
      return;
    }
    for (const child of vnode.children) destroy(child);
    fire(vnode, "destroy");
    const node = vnode.node;
    if (node == null) return;
    unsubscribeAll(node);
    const overlay = node.__pltCode;
    if (overlay) overlay.remove?.();
    node.__pltCode = null;
    node.__pltEntered = false;
  }

  /** Releases a component: its subscriptions, its subtree, and its handlers. */
  function dispose(instance) {
    if (instance == null || instance.disposed) return;
    instance.disposed = true;
    for (const stop of instance.stops ?? []) stop();
    instance.stops = [];
    instance.life?.destroy?.();
    destroy(instance.vnode);
  }

  // ---- components ----------------------------------------------------

  function child(name, inputs, children, listeners) {
    return { component: name, props: inputs, children, listeners, instance: null };
  }

  /**
   * Copies a parent's new props into the object the reused instance closes
   * over. The instance's `render` reads `inputs` directly, so writing into that
   * same object is what makes an input that the parent changed actually reach
   * the child; a fresh props object would leave the closure reading the values
   * the instance was built with. Declared defaults are re-applied afterwards,
   * because a parent only writes the inputs it actually passed.
   */
  function adoptProps(instance, props) {
    const target = instance.inputs;
    if (target == null) return;
    for (const name of Object.keys(target)) {
      if (!(name in props)) delete target[name];
    }
    Object.assign(target, props);
    for (const [name, fallback] of Object.entries(instance.inputDefaults ?? {})) {
      if (target[name] == null) target[name] = fallback;
    }
  }

  function ensureInstance(newV, reusable, parent) {
    if (reusable != null) {
      newV.instance = reusable;
      adoptProps(reusable, newV.props ?? {});
      return reusable;
    }
    const factory = $components[newV.component];
    if (!factory) throw new Error("unknown component: " + newV.component);
    const instance = factory(newV.props, newV.children);
    newV.instance = instance;
    instance.parent = parent;
    instance.stops = [];
    instance.disposed = false;
    instance.life?.create?.();
    watch(instance);
    // The factory already produced its first render, so place that tree rather
    // than diffing it against an empty one twice.
    mountTree(instance, parent);
    for (const [name, handler] of Object.entries(newV.listeners ?? {})) {
      const key = channelKey(name);
      const bus = instance.events?.[key];
      if (bus) bus.on(handler);
      else parent.addEventListener(key, handler);
    }
    return instance;
  }

  /**
   * Places a component's first tree and mounts it. A state write made by a
   * lifecycle handler while this runs cannot be applied by the render in
   * flight, so `busy` is held and the write is queued rather than re-entering
   * the patcher on a half-built tree.
   */
  function mountTree(instance, parent) {
    instance.busy = true;
    try {
      patchList(parent, [], rootsOf(instance.vnode));
      for (const node of rootsOf(instance.vnode)) enter(node);
      instance.life?.mount?.();
    } finally {
      instance.busy = false;
    }
    if (instance.pending) refresh(instance);
  }

  /** Re-renders the component whenever any of its reactive boxes changes. */
  function watch(instance) {
    const boxes = [
      ...Object.values(instance.s ?? {}),
      ...Object.values(instance.d ?? {}),
    ];
    for (const box of boxes) {
      instance.stops.push(box.subscribe(() => refresh(instance)));
      // A computed box also holds the subscriptions of its own dependencies.
      for (const stop of box.stops ?? []) instance.stops.push(stop);
    }
  }

  /**
   * Re-renders a component. A state write made while a render is in flight
   * cannot be applied by that render, so it is queued and picked up by a
   * follow-up pass rather than dropped. That is what lets a lifecycle handler
   * such as `on mount { count = 1 }` take effect.
   *
   * A write made inside a dispatch is queued behind the transaction instead,
   * so the whole dispatch costs one render rather than one per write.
   */
  function refresh(instance) {
    if (instance.disposed) return;
    if (batchDepth > 0) {
      instance.dirty = true;
      dirtied.add(instance);
      return;
    }
    if (instance.busy) {
      instance.pending = true;
      return;
    }
    instance.busy = true;
    try {
      do {
        instance.pending = false;
        instance.dirty = false;
        const next = instance.render();
        patchList(instance.parent, rootsOf(instance.vnode), rootsOf(next));
        instance.vnode = next;
        instance.life?.update?.();
      } while (instance.pending || instance.dirty);
    } finally {
      instance.busy = false;
    }
  }

  /** Wraps the application root so top-level `style { }` blocks have a target. */
  function host(instance) {
    const node = document.createElement("div");
    node.className = "plt-root";
    instance.parent = node;
    instance.stops = [];
    instance.disposed = false;
    instance.life?.create?.();
    watch(instance);
    mountTree(instance, node);
    return node;
  }

  function bind(target, instance) {
    target.__plt = instance;
    return instance;
  }

return {
    signal,
    computed,
    ref,
    global,
    persistent,
    emitter,
    emit,
    event,
    fetch,
    clipboardWrite,
    clipboardRead,
    openFile,
    webSocket,
    receive,
    store,
    load,
    drop,
    canvas,
    fill,
    clear,
    drawText,
    nextFrame,
    wait,
    exec,
    selection,
    indent,
    sortBy,
    page,
    el,
    fragment,
    branch,
    each,
    twoWay,
    child,
    route,
    host,
    bind,
    refresh,
    dispose,
  };
})();
"#;
