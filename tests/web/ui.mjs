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

const generated = await import(pathToFileURL(module).href);
const dom = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
const target = new dom.Element("div");
const instance = generated.mount(target);

const click = (node, type = "click") => node.dispatch(type, { target: node, type });
const buttonText = (text) => {
  const node = target.find((n) => n.tagName === "BUTTON" && n.textContent === text);
  assert(node != null, `no button with text ${JSON.stringify(text)}`);
  return node;
};

/** Advances every pending animation frame plus its microtasks. */
async function drainFrames() {
  for (let round = 0; round < 10; round += 1) {
    if (!globalThis.__rafQueue.length) break;
    while (globalThis.__rafQueue.length) {
      const callback = globalThis.__rafQueue.shift();
      callback();
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }
}

await check("the advanced fixture mounts", () => {
  assert(instance != null, "mount returned nothing");
  assert(target.find((n) => n.tagName === "CANVAS") != null, "no canvas");
  assert(target.find((n) => n.tagName === "TEXTAREA") != null, "no code editor");
  assert(target.byText("start") != null, "editor not seeded");
  assert(target.byText("beta") != null, "data grid did not sort onto page one");
  assert(target.byText("omega") == null, "data grid should have paged omega out");
});

await check("canvas drawing records operations under a resolved id", async () => {
  const canvas = target.find((n) => n.tagName === "CANVAS");
  assert(canvas != null, "no canvas");
  click(buttonText("draw"));
  await drainFrames();
  const ops = globalThis.__canvasOps;
  const rects = ops.filter((op) => op.op === "fillRect");
  // `tick` fills once per frame and stops when the frame counter reaches 3, so
  // three fills is the whole loop, not four.
  assert(rects.length === 3, `expected three fills, got ${rects.length}`);
  assert(ops[0].op === "clearRect", "a clear should come before the first fill");
  // The cleared area is the surface the element actually has, not a fixed guess:
  // a hardcoded 1000x1000 would wipe past the 320x200 canvas and waste a fill.
  assertEqual(
    JSON.stringify(ops[0].args),
    JSON.stringify([0, 0, 320, 200]),
    "a clear should cover exactly the canvas",
  );
  assert(rects.every((op) => op.color === "tomato"), "fills should use the requested color");
  assert(target.byText("3") != null, "the nextFrame loop should have run three times");
});

await check("a contenteditable editor binds text and reports the caret", () => {
  const editor = target.find((n) => n.getAttribute("contenteditable") === "true");
  assert(editor != null, "no contenteditable editor");
  assertEqual(editor.textContent, "start", "the binding should seed editor text");
  // An `on input` handler and a `bind` both want the same event, so they share
  // one DOM listener. Two listeners would mean a render between them, and a
  // render here writes the box from state -- throwing away what was just typed
  // and dropping the caret at the start.
  assertEqual(
    editor.listeners.get("input")?.size,
    1,
    "an on input handler and a bind should share one listener",
  );
  // Watch every write to the box, so a re-render that clobbers it is caught
  // rather than silently resetting the caret.
  const writes = [];
  let current = editor.textContent;
  Object.defineProperty(editor, "textContent", {
    configurable: true,
    get: () => current,
    set: (value) => {
      writes.push(value);
      current = value;
    },
  });
  editor.textContent = "chapter";
  globalThis.__selection = { rangeCount: 1, getRangeAt: () => ({ startOffset: 2, endOffset: 4 }) };
  // Only the dispatch is under observation; typing the text is the harness's
  // own write and would otherwise be counted as a clobbering render.
  writes.length = 0;
  editor.dispatch("input", { target: editor, type: "input" });
  assert(target.byText("chapter") != null, "state should follow the editor text");
  assert(target.byText("2") != null, "selection().start should reach state");
  assertEqual(
    writes.length,
    0,
    `a dispatch should render once, without writing the editor; got ${JSON.stringify(writes)}`,
  );
  // The text the user typed survives, and the reported caret is what they left.
  assertEqual(editor.textContent, "chapter", "the editor should keep what was typed");
});

await check("exec runs editing commands on the focused editor", () => {
  click(buttonText("bold"));
  click(buttonText("italic"));
  const ops = globalThis.__execs;
  assert(ops.includes("bold"), `missing bold in ${JSON.stringify(ops)}`);
  assert(ops.includes("italic"), `missing italic in ${JSON.stringify(ops)}`);
});

await check("a highlighted code editor keeps its token overlay in sync", async () => {
  const editor = target.find((n) => n.tagName === "TEXTAREA");
  // The overlay only reads as an overlay if it shares a box with the editor, so
  // the two live in a shell element the runtime builds rather than sitting side
  // by side. As siblings the same source is on screen twice.
  const shell = (() => {
    const found = target.find((n) => (n.attributes?.class ?? "").includes("plt-code-shell"));
    assert(found != null, "no shell around the code editor");
    return found;
  })();
  assert(shell.tagName === "DIV", `the shell should be a div, got ${shell.tagName}`);
  assert(
    shell.childNodes.some((n) => n === editor),
    "the editor should be inside the shell, not beside it",
  );
  const overlay = (() => {
    const pre = target.find((n) => n.tagName === "PRE");
    assert(pre != null, "no highlight overlay");
    return pre;
  })();
  assert(
    shell.childNodes.some((n) => n === overlay),
    "the overlay should be inside the same shell as the editor",
  );
  assert(overlay._html.includes("plt-tok"), "the overlay should tokenize keywords");
  assert(overlay._html.includes("fn"), "the overlay should contain the source");

  globalThis.__execs = [];
  globalThis.__active = editor;
  editor.selectionStart = 0;
  editor.selectionEnd = 0;
  click(buttonText("tab"));
  assertEqual(globalThis.__execs[0], "selectAll", "the tab keymap should run the command too");
  assert(editor.value.startsWith("  "), "indent should prefix the caret with two spaces");
  assert(target.byText("  fn hello() { return 1 }") != null, "indent should reach state");

  await new Promise((resolve) => setTimeout(resolve, 0));
  click(buttonText("grow"));
  assert(editor.value.includes("x"), "state growth should reach the editor");
  const synced = target.find((n) => n.tagName === "PRE");
  assert(synced != null && synced._html.includes("plt-tok"), "the overlay should stay tokenized");
});

await check("sortBy and page window the grid rows", () => {
  assert(target.byText("beta") != null, "sorted and paged rows should be rendered");
  assert(target.byText("alpha") == null, "page one should skip alpha");
  assert(target.byText("omega") == null, "page one should skip the omega head");
});

await check("a tree column expands and drops data onto a node", () => {
  assert(target.byText("a") != null, "static tree children missing");
  assert(target.byText("open") == null, "tree should start collapsed");
  click(buttonText("toggle"));
  assert(target.byText("open") != null, "toggle should expand the tree");
  const leaf = buttonText("leaf");
  leaf.dispatch("drop", { target: leaf, type: "drop", data: "item9" });
  assert(target.byText("item9") != null, "drop should deliver event.data");
});

await rm(module, { force: true });

console.log(`\n${passed} passed, ${failures.length} failed`);
// The code is set rather than `process.exit` called, because exiting outright
// drops whatever stdout has not been flushed yet when it is a pipe.
if (failures.length > 0) process.exitCode = 1;