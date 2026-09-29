import { readFile, writeFile, mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const self = path.basename(fileURLToPath(import.meta.url));
// The build directory is required; see the note in run.mjs.
const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error(`usage: node ${self} <build-dir>`);
  process.exit(2);
}
const entry = path.join(built, "app.js");

if (!existsSync(entry)) {
  console.error(`missing ${entry}; build the fixture first with \`plt build <fixture> -o ${built}\``);
  process.exit(1);
}

await mkdir(built, { recursive: true });
const module = path.join(built, "app.mjs");
await writeFile(module, await readFile(entry, "utf8"), "utf8");

// The shim the compiler emits is the one under test, so the harness has no copy.
const { installDom } = await import(pathToFileURL(path.join(built, "dom.mjs")).href);

installDom();

// The file picker is a browser capability the shim does not fake, so the
// harness stands in for it the way a browser would.
globalThis.showOpenFilePicker = async () => [
  {
    getFile: async () => ({
      name: "notes.txt",
      size: 5,
      type: "text/plain",
      text: async () => "hello",
    }),
  },
];

let passed = 0;
const failures = [];

async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`ok   ${name}`);
  } catch (error) {
    failures.push({ name, error });
    console.log(`FAIL ${name}`);
    console.log(`     ${error.message}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

async function until(fn, timeout = 2000) {
  const start = Date.now();
  for (;;) {
    if (fn()) return;
    if (Date.now() - start > timeout) throw new Error("timed out waiting for the condition");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

const generated = await import(pathToFileURL(module).href);
const dom = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
const target = new dom.Element("div");
const instance = generated.mount(target);

const byText = (text) => target.byText(text) != null;

await check("mount runs the async browser work", () => {
  assert(instance != null, "mount returned nothing");
  assert(target.byTag("div").length > 0, "no root element");
});

await check("fetch resolves into a state as a plain object", async () => {
  await until(() => byText("200:hi"));
});

await check("clipboard writes and reads", async () => {
  await until(() => byText("draft"));
});

await check("openFile delivers the picked file name", async () => {
  await until(() => byText("notes.txt"));
});

await check("received socket messages flow through receive", async () => {
  // The shim echoes what is sent, so the socket's own `send` already carries a
  // message back through `receive`. Injecting a second one by hand would only
  // prove the handler is still attached after it has already been consumed.
  const sockets = globalThis.__sockets ?? [];
  assert(sockets.length > 0, "no test socket instances");
  assert(sockets[0].sent.includes("ping"), `nothing was sent: ${sockets[0].sent}`);
  await until(() => byText("ping"));
});

await check("storage persists locally, per session, and in indexed", async () => {
  await until(() => byText("dark") && byText("1") && byText("x"));
});

await check("cancelling the fallback file picker resolves as cancelled", async () => {
  const originalCreateElement = globalThis.document.createElement;
  globalThis.showOpenFilePicker = undefined;
  let picker;
  globalThis.document.createElement = (tag) => {
    const node = originalCreateElement(tag);
    if (String(tag).toLowerCase() === "input") {
      picker = node;
      node.click = () => node.dispatch("cancel", { type: "cancel" });
    }
    return node;
  };

  try {
    const button = target.byText("cancel file picker");
    assert(button != null, "no fallback picker test button");
    button.dispatch("click", { target: button, type: "click" });
    await until(() => byText("true"));
    assert(picker != null, "the fallback input was not created");
    assertEqual(picker.type, "file", "fallback input type");
    assertEqual(picker.listenerCount("cancel"), 0, "cancel listener was not cleaned up");
  } finally {
    globalThis.document.createElement = originalCreateElement;
  }
});

await check("the fallback file picker reads a selected file", async () => {
  const originalCreateElement = globalThis.document.createElement;
  globalThis.showOpenFilePicker = undefined;
  let picker;
  globalThis.document.createElement = (tag) => {
    const node = originalCreateElement(tag);
    if (String(tag).toLowerCase() === "input") {
      picker = node;
      node.files = [
        {
          name: "fallback.txt",
          size: 8,
          type: "text/plain",
          text: async () => "fallback",
        },
      ];
      node.click = () => node.dispatch("change", { type: "change" });
    }
    return node;
  };

  try {
    const button = target.byText("choose fallback file");
    assert(button != null, "no fallback selection test button");
    button.dispatch("click", { target: button, type: "click" });
    await until(() => byText("fallback.txt"));
    assertEqual(picker?.listenerCount("change"), 0, "change listener was not cleaned up");
    assertEqual(picker?.listenerCount("cancel"), 0, "cancel listener was not cleaned up");
  } finally {
    globalThis.document.createElement = originalCreateElement;
  }
});

await rm(module, { force: true });

console.log(`\n${passed} passed, ${failures.length} failed`);
// The code is set rather than `process.exit` called, because exiting outright
// drops whatever stdout has not been flushed yet when it is a pipe.
if (failures.length > 0) process.exitCode = 1;