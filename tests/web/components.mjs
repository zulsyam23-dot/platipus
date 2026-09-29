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

const { mount } = await import(pathToFileURL(module).href);

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

const target = new (await import(pathToFileURL(path.join(built, "dom.mjs")).href)).Element("div");
mount(target);

/** Every element's own text, so a label on a button counts as well as on a span. */
const has = (text) => target.walk().some((n) => n.textContent === text);
const spans = () => target.walk().filter((n) => n.tagName === "SPAN").map((n) => n.textContent);
const button = (text) => target.find((n) => n.tagName === "BUTTON" && n.textContent === text);
/** The app's own column, told apart by the buttons it holds further down. */
const appColumn = () =>
  target.walk().find(
    (n) =>
      n.tagName === "DIV" &&
      (n.attributes?.class ?? "").includes("plt-column") &&
      // The buttons sit in a row of their own, so this has to look at the whole
      // subtree rather than the column's immediate children.
      n.walk().some((c) => c.tagName === "BUTTON" && c.textContent === "bump"),
  );
const press = (text) => {
  const node = button(text);
  assert(node != null, `no button labelled ${JSON.stringify(text)}`);
  node.dispatch("click", { target: node, type: "click" });
};

await check("a component that is the app's only root still renders", () => {
  // Nothing outside the component builds a tree here, so an uninstantiated child
  // has to be handed to the runtime rather than treated as an empty result.
  assert(has("outer"), "the component's button should be on screen");
  assert(has("0"), "the input the app passed should seed the counter");
});

await check("an input the app passed beats the component's own default", () => {
  // `start` defaults to 0 in the component, and the app also passes 0, so the
  // label is what shows the input arrived: the default is "count".
  assertEqual(button("outer").textContent, "outer", "the input should override the declared default");
});

await check("a component with two roots puts both in the document", () => {
  press("pair");
  assert(has("left-slot"), "the first root should be mounted");
  assert(has("right-slot"), "the second root should be mounted");
  assert(has("pair-footer"), "the sibling root should be mounted");
  // Both roots are direct children of the app's column: the runtime places a
  // fragment's children rather than inventing a wrapper to hold them.
  const column = appColumn();
  assert(column != null, "no app column");
  const roots = column.childNodes.filter((n) => n.textContent === "left-slotright-slot" || n.textContent === "pair-footer");
  assertEqual(roots.length, 2, "both roots should sit side by side in the parent");
  assertEqual(roots[0].tagName, "DIV", "the first root keeps its own tag");
  assertEqual(roots[1].tagName, "SPAN", "the second root keeps its own tag");
});

await check("swapping one section for another takes the old one out of the document", () => {
  press("counter");
  assert(has("outer"), "the counter should be back");
  // A component occupies nodes through its instance rather than through a tag of
  // its own, so removing it needs the instance, not the vnode's own node.
  assert(has("left-slot") == false, "the previous section's nodes should be gone");
  assert(has("right-slot") == false, "the previous section's nodes should be gone");
  assert(has("pair-footer") == false, "the previous section's nodes should be gone");
  assertEqual(
    target.walk().filter((n) => n.tagName === "BUTTON" && n.textContent === "outer").length,
    1,
    "the returning section should be mounted exactly once",
  );
});

await check("a branch that is not taken leaves nothing behind", () => {
  press("none");
  assert(has("nothing here"), "the taken branch should render");
  assert(has("outer") == false, "the branch that was replaced should be gone");
  assertEqual(button("outer"), null, "no stale button should remain to take a click");
});

await check("an input the parent changed reaches the child it is already holding", () => {
  press("counter");
  // A fresh mount seeds from the input, so the number on screen is still 0 here.
  assert(has("0"), "the mounted component should have seeded from the input");
  press("retag");
  // The instance is reused, so an input read once at mount would still read
  // `echo-0` on screen and the parent's change would go nowhere.
  assert(has("echo-1"), "the reused instance should show the parent's new input");
  assert(has("echo-0") == false, "the old input should not still be on screen");
  press("bump");
  // `n` belongs to the child, and the input only seeds it on mount, so a parent
  // that increments its own counter does not push its number into the child.
  assert(has("1") == false, "the child's own state should not follow the parent's input");
});

await check("a fresh mount seeds from the input the parent holds now", () => {
  // The app's counter is at 1, so a component mounted now has to come up at 1
  // rather than at the value the child last saw.
  press("pair");
  press("counter");
  assert(has("1"), "the new instance should seed from the current input");
  assert(has("echo-1"), "and take the input the parent holds now");
});

await check("the component's own state keeps counting across a swap", () => {
  // Mounting reseeded from the input, so the number is known before the click and
  // the button inside the component is the one that moves it.
  const inner = button("outer");
  assert(inner != null, "the component's button should be on screen");
  inner.dispatch("click", { target: inner, type: "click" });
  assert(has("2"), "the component's own click should add one to its own state");
});

await check("a removed section no longer responds", () => {
  // The node the component built is still reachable from here, so this asks the
  // runtime to act on something it has already torn down. A handler that outlived
  // its vnode would still write to state and the count would move.
  const stale = button("outer");
  assert(stale != null, "the component's button should be on screen");
  press("pair");
  // Taken after the swap, so the comparison is against what is on screen now and
  // not against the section that just left.
  const before = spans().join(",");
  assert(before.includes("pair-footer"), `the pair section should be up, got ${before}`);
  stale.dispatch("click", { target: stale, type: "click" });
  assertEqual(spans().join(","), before, "a detached component's handler should not run");
  assert(has("outer") == false, "and its nodes should stay out of the document");
});

await rm(built, { recursive: true });

console.log(`\n${passed} passed, ${failures.length} failed`);
// The code is set rather than `process.exit` called, because exiting outright
// drops whatever stdout has not been flushed yet when it is a pipe.
if (failures.length > 0) process.exitCode = 1;
