import { readFile, writeFile, mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const self = path.basename(fileURLToPath(import.meta.url));
// The build directory is required; see the note in run.mjs.
const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error(`usage: node ${self} <build-dir>`);
  console.error(`example: plt build tests/web/fixtures/templates.plt -o out && node ${self} out`);
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

const texts = () =>
  target.walk().filter((node) => node.nodeType === 1).map((node) => node.textContent);
const buttons = () => target.byTag("button");
const input = () => target.byTag("input")[0];

await check("mount builds the whole tree", () => {
  assert(instance != null, "mount returned nothing");
  assert(target.find((node) => node.getAttribute("class") === "plt-page") != null, "no page");
  assert(target.find((node) => node.getAttribute("class") === "plt-column") != null, "no column");
});

await check("an `if` in a template renders the taken branch", () => {
  assert(
    target.byText("not positive") != null,
    `expected the else branch, got ${JSON.stringify(texts())}`,
  );
});

await check("a `for` in a template renders every item", () => {
  assertEqual(target.byText("a") != null, true, "item a");
  assertEqual(target.byText("b") != null, true, "item b");
});

await check("reactive state drives the template", () => {
  buttons()[1].dispatch("click");
  assert(target.byText("positive") != null, "the then branch should appear after +1");
  assert(target.byText("not positive") == null, "the else branch should be gone");
});

await check("the branch survives repeated updates", () => {
  buttons()[1].dispatch("click");
  buttons()[1].dispatch("click");
  assertEqual(target.byText("positive") != null, true, "still positive");
  assertEqual(target.byText("3") != null, true, "count text");
});

await check("a list branch keeps its nodes across updates", () => {
  assertEqual(target.byText("a") != null, true, "item a kept");
  assertEqual(target.byText("b") != null, true, "item b kept");
});

await check("`bind` seeds the field from state", () => {
  assertEqual(input().value, 3, "input value");
});

await check("editing a bound field writes back into state", () => {
  input().value = "7";
  input().dispatch("input");
  assertEqual(target.byText("7") != null, true, "count follows the field");
});

await check("a zero count flips the branch back", () => {
  for (let step = 0; step < 7; step += 1) buttons()[0].dispatch("click");
  assert(target.byText("not positive") != null, "else branch returns");
});

await rm(module, { force: true });

console.log(`\n${passed} passed, ${failures.length} failed`);
// The code is set rather than `process.exit` called, because exiting outright
// drops whatever stdout has not been flushed yet when it is a pipe.
if (failures.length > 0) process.exitCode = 1;
