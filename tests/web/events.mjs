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

const input = () => target.byTag("input")[0];

await check("mount builds the handler fixture", () => {
  assert(instance != null, "mount returned nothing");
  assert(input() != null, "no input");
  assert(target.byText("start") != null, "initial field text");
});

await check("a field event exposes its value as event.value", () => {
  const field = input();
  field.value = "hello";
  field.dispatch("input", { target: field, type: "input" });
  assert(target.byText("hello") != null, "the state should follow event.value");
});

await check("a keyboard event exposes its key as event.key", () => {
  const field = input();
  field.dispatch("keydown", { target: field, key: "Enter", type: "keydown" });
  assert(target.byText("Enter") != null, "the state should follow event.key");
});

await check("a pointer event exposes its position as event.position", () => {
  const scroller = target.find((node) => node.getAttribute("class") === "plt-scroll");
  assert(scroller != null, "no scroll element");
  scroller.dispatch("scroll", { target: scroller, clientX: 5, clientY: 40, type: "scroll" });
  assert(target.byText("40") != null, "the state should follow event.position.y");
});

await check("a scrollTop bind keeps state in step with the scroll", () => {
  const scrollers = target.walk().filter(
    (node) => node.getAttribute("class") === "plt-scroll",
  );
  assert(scrollers.length >= 2, "expected two scroll elements");
  const bound = scrollers[1];
  bound.scrollTop = 60;
  bound.dispatch("scroll", { target: bound, type: "scroll" });
  assert(target.byText("60") != null, "depth should follow the bound scrollTop");
});

await rm(module, { force: true });

console.log(`\n${passed} passed, ${failures.length} failed`);
// The code is set rather than `process.exit` called, because exiting outright
// drops whatever stdout has not been flushed yet when it is a pipe.
if (failures.length > 0) process.exitCode = 1;