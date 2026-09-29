import { readFile, writeFile, mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const self = path.basename(fileURLToPath(import.meta.url));
const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error(`usage: node ${self} <build-dir>`);
  process.exit(2);
}
const entry = path.join(built, "app.js");
if (!existsSync(entry)) {
  console.error(`missing ${entry}; build the showcase first`);
  process.exit(1);
}

await mkdir(built, { recursive: true });
const module = path.join(built, "app.mjs");
await writeFile(module, await readFile(entry, "utf8"), "utf8");
const dom = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
dom.installDom();
const generated = await import(pathToFileURL(module).href);
const target = new dom.Element("div");
const instance = generated.mount(target);

let passed = 0;
const failures = [];

async function check(name, run) {
  try {
    await run();
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

const hasText = (text) => target.walk().some((node) => node.textContent === text);
const hasPart = (text) => target.walk().some((node) => node.textContent.includes(text));
const button = (text) =>
  target.find((node) => node.tagName === "BUTTON" && node.textContent === text);
const click = (text) => {
  const node = button(text);
  assert(node != null, `no button labelled ${JSON.stringify(text)}`);
  node.dispatch("click", { target: node, type: "click" });
  return node;
};
const tab = (name) => click(name);

async function until(predicate, timeout = 2000) {
  const start = Date.now();
  while (!predicate()) {
    if (Date.now() - start > timeout) throw new Error("timed out waiting for showcase state");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

await check("the app mounts and reports lifecycle updates", () => {
  assert(instance != null, "mount returned nothing");
  assert(hasText("Showcase Studio"), "showcase heading is missing");
  assert(hasText("update hook emits ticked"), "lifecycle update is mislabeled");
  click("Reactive");
  assert(hasPart("update event"), "the update event did not reach the app");
});

await check("form values bind, persist, and submit", () => {
  tab("Forms");
  const name = target.find(
    (node) => node.tagName === "INPUT" && node.getAttribute("placeholder") === "your name",
  );
  assert(name != null, "name input is missing");
  name.value = "ada-lovelace";
  name.dispatch("input", { target: name, type: "input" });
  name.dispatch("focusout", { target: name, type: "focusout" });
  assertEqual(
    JSON.parse(globalThis.localStorage.getItem("showcase.name")),
    "ada-lovelace",
    "blur should persist the bound name",
  );

  const form = target.find((node) => node.tagName === "FORM");
  assert(form != null, "showcase inputs are not inside a form");
  let prevented = false;
  form.dispatch("submit", {
    target: form,
    type: "submit",
    preventDefault() {
      prevented = true;
    },
  });
  assert(prevented, "submit should prevent a page navigation");
  assert(hasText("ada-5"), "submit did not show its resulting name");
  assertEqual(
    JSON.parse(globalThis.localStorage.getItem("showcase.name")),
    "ada-5",
    "submit should persist the resulting name",
  );
});

await check("drag and drop carries a real text payload", () => {
  tab("Events");
  const source = target.find(
    (node) => node.tagName === "A" && node.textContent === "drag this text onto the target",
  );
  const drop = button("drop target");
  assert(source != null && drop != null, "drag source or drop target is missing");
  let payload = "";
  const dataTransfer = {
    setData(type, value) {
      if (type === "text/plain") payload = String(value);
    },
    getData(type) {
      return type === "text/plain" ? payload : "";
    },
  };
  source.dispatch("dragstart", { target: source, type: "dragstart", dataTransfer });
  drop.dispatch("drop", { target: drop, type: "drop", dataTransfer });
  assert(hasText("showcase payload"), "drop did not expose event.data");
});

await check("Tree expands through state and a conditional", () => {
  tab("Data");
  const tree = target.byTag("ul").find((node) =>
    node.textContent.includes("teams"),
  );
  assert(tree != null, "the showcase Tree is missing");
  assertEqual(
    tree.textContent,
    "teams",
    "tree children should start collapsed",
  );
  click("toggle tree");
  assert(
    tree.textContent.includes("core") && tree.textContent.includes("web") && tree.textContent.includes("docs"),
    "tree children did not expand",
  );
  click("toggle tree");
  assertEqual(tree.textContent, "teams", "tree children did not collapse");
});

await check("Tab indents the focused CodeEditor", () => {
  tab("Editors");
  const editor = target.byTag("textarea").at(-1);
  assert(editor != null, "CodeEditor textarea is missing");
  globalThis.__active = editor;
  const caret = editor.value.indexOf("return");
  editor.selectionStart = caret;
  editor.selectionEnd = caret;
  let prevented = false;
  editor.dispatch("keydown", {
    target: editor,
    type: "keydown",
    key: "Tab",
    preventDefault() {
      prevented = true;
    },
  });
  assert(prevented, "Tab should not move focus away from the editor");
  assert(editor.value.includes("    return 1"), "indent() did not insert two spaces");
  globalThis.__active = null;
});

await check("canvas runs six frames without overlapping loops", async () => {
  tab("Canvas");
  click("paint");
  for (let round = 0; round < 20; round += 1) {
    while (globalThis.__rafQueue.length > 0) {
      globalThis.__rafQueue.shift()();
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
    if (hasText("drew frame 6")) break;
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  const ops = globalThis.__canvasOps;
  assertEqual(ops.filter((op) => op.op === "fillRect").length, 30, "six frames of five bars");
  assertEqual(ops.filter((op) => op.op === "clearRect").length, 6, "one clear per frame");
  assert(hasText("drew frame 6"), "animation did not finish");
});

await check("browser API controls report their results", async () => {
  tab("Browser APIs");
  click("fetch");
  await until(() => hasText("200:platipus"));

  click("clipboard write");
  await until(() => hasText("clipboard write: true"));
  click("clipboard read");
  await until(() => hasText("clipboard read: copied at 200:platipus"));

  globalThis.__pick = { name: "showcase.txt", text: "real file payload" };
  click("open file");
  await until(() => hasText("file: showcase.txt"));

  const socketUrl = target.find(
    (node) => node.tagName === "INPUT" && node.getAttribute("placeholder") === "wss://your-echo-server",
  );
  assert(socketUrl != null, "WebSocket endpoint input is missing");
  socketUrl.value = "ws://echo";
  socketUrl.dispatch("input", { target: socketUrl, type: "input" });
  click("socket");
  await until(() => hasText("socket: ping"));

  click("storage");
  await until(() => hasText("local-value") && hasText("session-value") && hasText("indexed-value"));

  click("wait 30ms");
  await until(() => hasPart("waited, was"));
});

await check("sorting and pagination update the visible list", () => {
  tab("Collections");
  click("sort asc");
  assert(hasText("alpha,bravo"), "ascending page is incorrect");
  click("page 1");
  assert(hasText("charlie,delta"), "second sorted page is incorrect");
  click("sort desc");
  assert(hasText("bravo,alpha"), "descending page is incorrect");
});

await check("calculator evaluates chained operations and edge cases", () => {
  tab("Calculator");
  click("C");
  click("1");
  click("2");
  click("+");
  click("7");
  click("=");
  assert(hasText("19"), "12 + 7 should equal 19");

  click("C");
  click("1");
  click("2");
  click("+");
  click("7");
  click("×");
  click("2");
  click("=");
  assert(hasText("38"), "chained operations should evaluate left to right");

  click("C");
  click("5");
  click("0");
  click("%");
  assert(hasText("0.5"), "50 percent should equal 0.5");

  click("C");
  click("8");
  click("±");
  click("2");
  assert(hasText("-82"), "digit entry should preserve the negative sign");

  click("C");
  click("8");
  click("÷");
  click("0");
  click("=");
  assert(hasText("Cannot divide by zero"), "division by zero should be reported");
});

await rm(module, { force: true });
console.log(`\n${passed} passed, ${failures.length} failed`);
if (failures.length > 0) process.exitCode = 1;
