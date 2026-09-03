import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const rootSets = [
  ["README.md", "README.ru.md", "README.zh-CN.md"],
  ["CHANGES.md", "CHANGES.ru.md", "CHANGES.zh-CN.md"],
  ["MEMORY.md", "MEMORY.ru.md", "MEMORY.zh-CN.md"],
  ["AGENTS.en.md", "AGENTS.md", "AGENTS.zh-CN.md"],
];

const collectMarkdown = (directory) =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return collectMarkdown(path);
    return entry.isFile() && entry.name.endsWith(".md") ? [path] : [];
  });

const docs = collectMarkdown("docs");
const englishDocs = docs.filter(
  (name) => !name.endsWith(".ru.md") && !name.endsWith(".zh-CN.md"),
);
const docSets = englishDocs.map((name) => {
  const stem = name.slice(0, -3);
  return [
    name,
    `${stem}.ru.md`,
    `${stem}.zh-CN.md`,
  ];
});

const sets = [...rootSets, ...docSets];
const errors = [];

for (const set of sets) {
  const readable = [];
  for (const file of set) {
    if (!existsSync(file)) {
      errors.push(`missing translation: ${file}`);
      continue;
    }
    const source = readFileSync(file, "utf8");
    readable.push({ file, source });
    const header = source.slice(0, 320);
    for (const label of ["English", "Русский", "简体中文"]) {
      if (!header.includes(label)) errors.push(`${file}: missing '${label}' link in header`);
    }
    for (const match of source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)) {
      const target = match[1].trim();
      if (/^(https?:\/\/|mailto:|#)/.test(target)) continue;
      const pathOnly = target.split("#")[0];
      if (pathOnly && !existsSync(resolve(dirname(file), pathOnly))) {
        errors.push(`${file}: broken relative link '${target}'`);
      }
    }
  }

  if (readable.length === 3) {
    const headingShape = ({ source }) =>
      [...source.matchAll(/^(#{1,6})\s+/gm)].map((match) => match[1].length).join(",");
    const [english, russian, chinese] = readable;
    const expectedShape = headingShape(english);

    for (const translation of [russian, chinese]) {
      if (headingShape(translation) !== expectedShape) {
        errors.push(`${translation.file}: heading structure differs from ${english.file}`);
      }
    }

    const compactLength = ({ source }) => source.replace(/\s+/g, "").length;
    const englishLength = compactLength(english);
    if (compactLength(russian) < englishLength * 0.4) {
      errors.push(`${russian.file}: translation is unexpectedly short`);
    }
    // Simplified Chinese conveys the same material with substantially fewer characters.
    if (compactLength(chinese) < englishLength * 0.23) {
      errors.push(`${chinese.file}: translation is unexpectedly short`);
    }
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(
  `Checked ${sets.length} translation sets (${sets.length * 3} files): complete, structurally aligned, and linked.`,
);
