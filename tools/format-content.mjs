// Repository-owned content only. Uses exactly the offline formatter shipped in the app.
import fs from "node:fs";
import prettier from "../assets/vendor/prettier/standalone.mjs";
import * as typescript from "../assets/vendor/prettier/typescript.mjs";
import * as estree from "../assets/vendor/prettier/estree.mjs";
const write = process.argv.includes("--write");
const fields = new Set([
  "starter",
  "solution",
  "alternative",
  "negative",
  "checks",
  "runtime",
  "example",
  "playground",
]);
const options = {
  parser: "typescript",
  plugins: [typescript, estree],
  printWidth: 80,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  trailingComma: "all",
  endOfLine: "lf",
};
let changed = 0,
  checked = 0;
async function visit(value, location) {
  if (!value || typeof value !== "object") return;
  for (const [key, item] of Object.entries(value)) {
    if (fields.has(key) && typeof item === "string" && item.trim()) {
      try {
        const formatted = await prettier.format(item, options);
        checked++;
        if (formatted !== item) {
          changed++;
          value[key] = formatted;
        }
      } catch (error) {
        throw new Error(location + "." + key + ": " + error.message);
      }
    } else await visit(item, location + "." + key);
  }
}
for (const path of ["content/course.json", "content/learning.json"]) {
  const data = JSON.parse(fs.readFileSync(path, "utf8"));
  await visit(data, path);
  if (write) fs.writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
}
console.log(
  `Prettier ${prettier.version}: ${checked} code fields checked; ${changed} ${write ? "formatted" : "need formatting"}.`,
);
if (changed && !write) process.exitCode = 1;
