import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [directory, outputName] = process.argv.slice(2);

if (!directory || !outputName) {
  console.error("Usage: node scripts/checksum-assets.mjs <directory> <output-file>");
  process.exit(2);
}

const files = readdirSync(directory)
  .filter((name) => name !== outputName && statSync(join(directory, name)).isFile())
  .sort();

if (files.length === 0) {
  console.error(`No release assets found in ${directory}`);
  process.exit(1);
}

const lines = files.map((name) => {
  const digest = createHash("sha256")
    .update(readFileSync(join(directory, name)))
    .digest("hex");
  return `${digest}  ${name}`;
});

writeFileSync(join(directory, outputName), `${lines.join("\n")}\n`);
console.log(`Wrote ${outputName} for ${files.length} asset(s).`);
