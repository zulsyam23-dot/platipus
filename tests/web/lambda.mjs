import { readFile, writeFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const built = process.argv[2] ? path.resolve(process.argv[2]) : null;
if (!built) {
  console.error("usage: node lambda.mjs <build-dir>");
  process.exit(2);
}
const entry = path.join(built, "app.js");
if (!existsSync(entry)) {
  console.error(`missing ${entry}; build the fixture first with \`plt build tests/web/fixtures/lambda.plt -o ${built}\``);
  process.exit(1);
}

await mkdir(built, { recursive: true });
const modulePath = path.join(built, "app.mjs");
await writeFile(modulePath, await readFile(entry, "utf8"), "utf8");
const { installDom } = await import(pathToFileURL(path.join(built, "dom.mjs")).href);
installDom();
const { mount } = await import(pathToFileURL(modulePath).href);

const target = new (await import(pathToFileURL(path.join(built, "dom.mjs")).href)).Element("div");
await mount(target);

let passed = 0;
const failures = [];
function check(name, fn) {
  try { fn(); passed += 1; console.log(`ok   ${name}`); }
  catch (error) { failures.push({ name, error }); console.log(`FAIL ${name}\n     ${error.message}`); }
}
const text = () => target.walk().filter((n) => n.tagName === "TEXT" || n.nodeType === 3 || n.textContent).map((n) => n.textContent).join(" ");

check("no output before the button is pressed", () => {
  if (!target.textContent.includes("")) throw new Error("unexpected");
});

const button = target.walk().find((n) => n.tagName === "BUTTON" && n.textContent === "go");
if (!button) { console.error("missing go button"); process.exit(1); }
button.dispatch("click", { target: button, type: "click" });

check("lambda results land in state and the DOM", () => {
  const all = target.walk().map((n) => n.textContent || "").join(" ");
  for (const expected of ["8", "5", "9", "11-12-13"]) {
    if (!all.includes(expected)) throw new Error(`expected ${expected} in ${JSON.stringify(all)}`);
  }
});

if (failures.length > 0) process.exitCode = 1;
