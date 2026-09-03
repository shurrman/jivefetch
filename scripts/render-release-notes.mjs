import { readFileSync } from "node:fs";
import { posix } from "node:path";

const [notesPath, repository, ref, mode] = process.argv.slice(2);

if (!notesPath || !repository || !ref) {
  console.error("Usage: render-release-notes.mjs <notes-path> <owner/repo> <git-ref>");
  process.exit(2);
}

const repositoryBase = `https://github.com/${repository}/blob/${encodeURIComponent(ref)}/`;
const releaseBase = `https://github.com/${repository}/releases/tag/${encodeURIComponent(ref)}`;
const notesStem = notesPath.endsWith(".md") ? notesPath.slice(0, -3) : null;

if (!notesStem) {
  console.error("Release notes path must end with .md");
  process.exit(2);
}

const languages = [
  { label: "English", heading: "English", anchor: "english", path: notesPath },
  { label: "Русский", heading: "Russian", anchor: "russian", path: `${notesStem}.ru.md` },
  {
    label: "简体中文",
    heading: "Simplified Chinese",
    anchor: "simplified-chinese",
    path: `${notesStem}.zh-CN.md`,
  },
];

const makeLinksAbsolute = (markdown, sourcePath) => {
  const sourceDirectory = posix.dirname(sourcePath.replaceAll("\\", "/"));
  return markdown.replace(
    /(\[[^\]]*\]\()([^)]+)(\))/g,
    (match, open, target, close) => {
      if (/^(?:[a-z][a-z\d+.-]*:|#)/i.test(target)) return match;

      const suffixAt = target.search(/[?#]/);
      const relativePath = suffixAt === -1 ? target : target.slice(0, suffixAt);
      const suffix = suffixAt === -1 ? "" : target.slice(suffixAt);
      const resolvedPath = posix.normalize(posix.join(sourceDirectory, relativePath));
      return `${open}${repositoryBase}${resolvedPath}${suffix}${close}`;
    },
  );
};

const sources = languages.map((language) => ({
  ...language,
  markdown: readFileSync(language.path, "utf8"),
}));
const titleMatch = sources[0].markdown.match(/^#\s+(.+)$/m);

if (!titleMatch) {
  console.error(`Release title is missing from ${notesPath}`);
  process.exit(1);
}

const sections = sources.map(({ heading, path, markdown }) => {
  const content = markdown
    .replace(/^\[English\][^\n]*\n+/, "")
    .replace(/^#\s+[^\n]+\n+/, "")
    .replace(/^(#{2,5})(\s+)/gm, (_match, hashes, spacing) => `${hashes}#${spacing}`)
    .trim();
  return `## ${heading}\n\n${makeLinksAbsolute(content, path)}`;
});
const languageNavigation = languages
  .map(({ label, anchor }) => `[${label}](${releaseBase}#${anchor})`)
  .join(" | ");
const rendered = `${languageNavigation}\n\n# ${titleMatch[1]}\n\n${sections.join("\n\n---\n\n")}\n`;

if (mode === "--check") {
  const unresolved = [...rendered.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)]
    .map((match) => match[1])
    .filter((target) => !/^(?:[a-z][a-z\d+.-]*:|#)/i.test(target));
  const missingSections = languages
    .filter(({ heading }) => !rendered.includes(`## ${heading}`))
    .map(({ heading }) => heading);
  const repositoryTranslationLinks = languages
    .slice(1)
    .filter(({ path }) => rendered.includes(`${repositoryBase}${path}`))
    .map(({ path }) => path);
  const missingNavigationTargets = languages
    .filter(({ anchor }) => !rendered.includes(`${releaseBase}#${anchor}`))
    .map(({ anchor }) => anchor);
  if (
    unresolved.length > 0 ||
    missingSections.length > 0 ||
    repositoryTranslationLinks.length > 0 ||
    missingNavigationTargets.length > 0
  ) {
    console.error(`Release-note URL rendering failed: ${unresolved.join(", ")}`);
    if (missingSections.length > 0) {
      console.error(`Missing in-page language sections: ${missingSections.join(", ")}`);
    }
    if (repositoryTranslationLinks.length > 0) {
      console.error(`Translations leave the release page: ${repositoryTranslationLinks.join(", ")}`);
    }
    if (missingNavigationTargets.length > 0) {
      console.error(`Missing tag-page language anchors: ${missingNavigationTargets.join(", ")}`);
    }
    process.exit(1);
  }
  console.log(
    `Checked release notes for ${ref}: three tag-page language anchors and no unresolved links.`,
  );
} else {
  process.stdout.write(rendered);
}
