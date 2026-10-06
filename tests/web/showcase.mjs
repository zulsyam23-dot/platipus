import { readFile, writeFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const self = path.basename(fileURLToPath(import.meta.url));
const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error(`usage: node ${self} <build-dir>`);
  process.exit(2);
}
const entry = path.join(built, "app.js");
if (!existsSync(entry)) {
  console.error(`missing ${entry}; build the fixture first`);
  process.exit(1);
}

const module = path.join(built, "app.mjs");
await writeFile(module, await readFile(entry, "utf8"), "utf8");
const dom = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
dom.installDom();
const generated = await import(pathToFileURL(module).href);

let passed = 0;
const failures = [];
function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`ok   ${name}`);
  } catch (error) {
    failures.push(name);
    console.log(`FAIL ${name}: ${error.message}`);
  }
}
function assertEqual(actual, expected, label) {
  if (actual !== expected) throw new Error(`${label}: expected ${expected}, got ${actual}`);
}

const target = new dom.Element("div");
const instance = generated.mount(target);
const texts = () =>
  target.walk().filter((node) => node.nodeType === 1).map((node) => node.textContent);

check("mount renders", () => {
  if (instance == null) throw new Error("mount returned nothing");
});

check("heading renders", () => {
  if (!texts().some((text) => text.includes("Platipus Showcase"))) {
    throw new Error("heading missing");
  }
});

check("rust wasm computes fib(12) = 144", () => {
  if (!texts().some((text) => text === "144")) {
    throw new Error("fib(12) missing; rust bridge not running");
  }
});

check("counter state updates", () => {
  const plus = target.byTag("button")[0];
  plus.dispatch("click");
  plus.dispatch("click");
  if (!texts().includes("2")) throw new Error("counter did not reach 2");
});

if (failures.length > 0) {
  console.log(`${failures.length} failed`);
  process.exit(1);
}
console.log(`${passed} passed`);
