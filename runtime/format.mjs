// Fixed offline formatter: no user config, plugins or source execution.
import fs from "node:fs";
import prettier from "./standalone.mjs";
import * as typescript from "./typescript.mjs";
import * as estree from "./estree.mjs";
try {
  const source = fs.readFileSync("input.ts", "utf8");
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
  const formatted = await prettier.format(source, options);
  const points = JSON.parse(fs.readFileSync("positions.json", "utf8"));
  const lines = source.split("\n");
  const mapped = [];
  for (const line of points) {
    const row = Math.max(0, Math.min(line - 1, lines.length - 1));
    const prefix = lines.slice(0, row).join("\n");
    const offset =
      prefix.length +
      (row > 0 ? 1 : 0) +
      (lines[row].search(/\S/) < 0 ? 0 : lines[row].search(/\S/));
    const result = await prettier.formatWithCursor(source, {
      ...options,
      cursorOffset: offset,
    });
    mapped.push(
      result.formatted.slice(0, Math.max(0, result.cursorOffset)).split("\n")
        .length,
    );
  }
  fs.writeFileSync(
    "output.json",
    JSON.stringify({ source: formatted, breakpoints: mapped }),
  );
} catch (error) {
  process.stderr.write(String(error.message || error).slice(0, 4000));
  process.exitCode = 1;
}
