// A repeatable performance harness for the generated runtime.
//
// The compiler turns a program into JavaScript, so the only honest way to talk
// about how fast it is to run it. This harness builds the fixture through the
// compiler, mounts it against the emitted DOM shim, times each workload
// separately, and prints a table that can be diffed between runs on the same
// machine.
//
// It deliberately asserts almost nothing about time. A wall-clock bound inside
// `cargo test` fails on a slow or busy machine and then gets deleted, which is
// worse than having no bound at all. What it does assert is that each workload
// produced the right answer: a change that makes the runtime faster by computing
// the wrong thing should fail here rather than pass quietly.
//
// The one bound it keeps is a ceiling far above anything a working build
// reaches, so a regression that turns a linear loop into something quadratic
// fails loudly instead of merely getting slow.

import { readFile, writeFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error("usage: node benchmark.mjs <build-dir>");
  process.exit(2);
}
const entry = path.join(built, "app.js");
if (!existsSync(entry)) {
  console.error(`missing ${entry}; build the fixture first`);
  process.exit(1);
}

await mkdir(built, { recursive: true });
const modulePath = path.join(built, "app.mjs");
await writeFile(modulePath, await readFile(entry, "utf8"), "utf8");
const dom = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
dom.installDom();
const { mount } = await import(pathToFileURL(modulePath).href);

const target = new dom.Element("div");
const instance = mount(target);

// Reading state back is how the harness knows a workload finished; the render is
// synchronous, so the value is there the moment `dispatch` returns.
const read = (name) => instance.s[name].value;
const button = (label) =>
  target
    .walk()
    .filter((node) => node.tagName === "BUTTON" && node.textContent === label)
    .pop();

const WORKLOADS = [
  { button: "range", key: "rangeTotal", expected: 499999500000, note: "for i in 0..1_000_000" },
  { button: "while", key: "whileTotal", expected: 499999500000, note: "while i < 1_000_000" },
  { button: "list", key: "listTotal", expected: 9999900000, note: "100k list, map, fold" },
  { button: "calls", key: "fibTotal", expected: 229988, note: "1000 recursive calls" },
];

for (const workload of WORKLOADS) {
  if (!button(workload.button)) {
    console.error(`the fixture has no ${workload.button} button`);
    process.exit(1);
  }
}

// A warm-up pass over everything first. The first run pays for module evaluation
// and for V8's initial optimisation of the generated functions, which would
// otherwise be charged to whichever workload happened to run first.
for (const workload of WORKLOADS) {
  button(workload.button).dispatch("click");
  if (read(workload.key) !== workload.expected) {
    console.error(`${workload.button} did not produce the right answer on the warm-up pass`);
    process.exit(1);
  }
}

let passed = 0;
const failures = [];
function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`ok   ${name}`);
  } catch (error) {
    failures.push({ name, error });
    console.log(`FAIL ${name}\n     ${error.message}`);
  }
}

console.log("");
console.log("workload                       result          time");
console.log("---------------------------  --------------  ---------");

let wrong = [];
for (const workload of WORKLOADS) {
  const target = button(workload.button);
  const started = process.hrtime.bigint();
  target.dispatch("click");
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
  const value = read(workload.key);
  const right = value === workload.expected;
  if (!right) wrong.push(workload.note);
  console.log(
    `${workload.note.padEnd(27)}  ${String(value).padStart(14)}  ${elapsedMs.toFixed(1).padStart(7)} ms${right ? "" : "   <- wrong"}`,
  );
}

// The ceiling is deliberately far above a healthy build. A million counted
// iterations cost single-digit milliseconds on any current machine, so thirty
// seconds is not a performance target; it is the point at which something has
// stopped being linear.
const started = process.hrtime.bigint();
button("all").dispatch("click");
const wholeMs = Number(process.hrtime.bigint() - started) / 1e6;
check("the whole run stays far below the ceiling", () => {
  if (wholeMs > 30_000) {
    throw new Error(`a full run took ${wholeMs.toFixed(0)} ms; a loop has probably stopped being linear`);
  }
});

check("every workload produced its exact value", () => {
  if (wrong.length > 0) {
    throw new Error(`these returned the wrong number: ${wrong.join(", ")}`);
  }
});

console.log("");
console.log(`one full run of all four: ${wholeMs.toFixed(1)} ms`);

if (failures.length > 0) {
  process.exitCode = 1;
}
