import { readFileSync } from "node:fs";
import { posix } from "node:path";

const [notesPath, repository, ref, mode] = process.argv.slice(2);

if (!notesPath || !repository || !ref) {
  console.error("Usage: render-release-notes.mjs <notes-path> <owner/repo> <git-ref>");
  process.exit(2);
}

const sourceDirectory = posix.dirname(notesPath.replaceAll("\\", "/"));
const repositoryBase = `https://github.com/${repository}/blob/${encodeURIComponent(ref)}/`;
const markdown = readFileSync(notesPath, "utf8");

const rendered = markdown.replace(/(\[[^\]]*\]\()([^)]+)(\))/g, (match, open, target, close) => {
  if (/^(?:[a-z][a-z\d+.-]*:|#)/i.test(target)) return match;

  const suffixAt = target.search(/[?#]/);
  const relativePath = suffixAt === -1 ? target : target.slice(0, suffixAt);
  const suffix = suffixAt === -1 ? "" : target.slice(suffixAt);
  const resolvedPath = posix.normalize(posix.join(sourceDirectory, relativePath));
  return `${open}${repositoryBase}${resolvedPath}${suffix}${close}`;
});

if (mode === "--check") {
  const unresolved = [...rendered.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)]
    .map((match) => match[1])
    .filter((target) => !/^(?:[a-z][a-z\d+.-]*:|#)/i.test(target));
  if (unresolved.length > 0 || !rendered.includes(repositoryBase)) {
    console.error(`Release-note URL rendering failed: ${unresolved.join(", ")}`);
    process.exit(1);
  }
  console.log(`Checked release-note URLs for ${ref}: all links are absolute.`);
} else {
  process.stdout.write(rendered);
}
