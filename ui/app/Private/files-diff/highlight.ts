import type { DiffLine, FileDiff } from "./wire.ts";
import { esc } from "../dom.ts";

interface PrismToken {
  type: string;
  content: string | PrismToken | (string | PrismToken)[];
  alias?: string | string[];
}

interface PrismApi {
  languages: Record<string, unknown>;
  highlight(text: string, grammar: unknown, language: string): string;
  tokenize(text: string, grammar: unknown): (string | PrismToken)[];
}

interface Grammar {
  /// Grammars this one extends, which must run first.
  needs: string[];
  load: () => Promise<unknown>;
}

const EXTENSIONS: Record<string, string> = {
  ts: "typescript",
  mts: "typescript",
  cts: "typescript",
  tsx: "tsx",
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  jsx: "jsx",
  rs: "rust",
  css: "css",
  html: "markup",
  htm: "markup",
  svg: "markup",
  xml: "markup",
  json: "json",
  md: "markdown",
  toml: "toml",
  sh: "bash",
  bash: "bash",
  py: "python",
  yml: "yaml",
  yaml: "yaml",
};

/// Prism's core already carries markup and css; every other grammar
/// is a component file with an explicit `.js`, because Node does no extension
/// resolution for a deep CommonJS path.
const GRAMMARS: Record<string, Grammar> = {
  clike: { needs: [], load: () => import("prismjs/components/prism-clike.js") },
  javascript: { needs: ["clike"], load: () => import("prismjs/components/prism-javascript.js") },
  typescript: { needs: ["javascript"], load: () => import("prismjs/components/prism-typescript.js") },
  jsx: { needs: ["javascript"], load: () => import("prismjs/components/prism-jsx.js") },
  tsx: { needs: ["typescript", "jsx"], load: () => import("prismjs/components/prism-tsx.js") },
  rust: { needs: ["clike"], load: () => import("prismjs/components/prism-rust.js") },
  json: { needs: [], load: () => import("prismjs/components/prism-json.js") },
  markdown: { needs: [], load: () => import("prismjs/components/prism-markdown.js") },
  toml: { needs: [], load: () => import("prismjs/components/prism-toml.js") },
  bash: { needs: [], load: () => import("prismjs/components/prism-bash.js") },
  python: { needs: [], load: () => import("prismjs/components/prism-python.js") },
  yaml: { needs: ["clike"], load: () => import("prismjs/components/prism-yaml.js") },
};

/// A line longer than this is almost always minified or generated, where
/// tokenizing costs the most and reads the least.
const MAX_HIGHLIGHTED_LINE = 500;
const MAX_MEMO_ENTRIES = 20_000;

const memo = new Map<string, string>();
const loaded = new Set<string>();
let prism: PrismApi | null = null;

export function languageFor(path: string): string {
  const name = path.slice(path.lastIndexOf("/") + 1).toLowerCase();
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return "";
  return EXTENSIONS[name.slice(dot + 1)] ?? "";
}

/// Prism is browser-only and its component files bind a free `Prism` through the
/// global, so the core is published to `globalThis` before any grammar runs.
/// Without a document — which is how the test runner loads these sources, with no
/// bundler — nothing is imported at all and every line stays escaped plain text.
export async function ensureGrammars(languages: string[]): Promise<void> {
  if (!globalThis.document) return;
  try {
    prism ??= await loadCore();
    for (const language of new Set(languages)) await loadGrammar(language);
  } catch {
    // Highlighting is decoration: a missing grammar must not fail the diff.
  }
}

/// Pre-highlights file streams so multi-line constructs (block comments, template
/// strings) are tokenized across lines rather than broken into single-line tokens.
export function highlightsFor(
  file: FileDiff,
  full: FileDiff | null,
  language: string,
): Map<DiffLine, string> {
  const map = new Map<DiffLine, string>();
  if (!language || !prism?.languages[language]) return map;

  if (full?.hunks.length) {
    const fullLines = full.hunks.flatMap((hunk) => hunk.lines);
    const rendered = highlightLines(fullLines.map((l) => l.text), language);
    fullLines.forEach((line, index) => map.set(line, rendered[index]));
  }

  for (const hunk of file.hunks) {
    mapStream(hunk.lines.filter((l) => l.kind !== "add"), language, map);
    mapStream(hunk.lines.filter((l) => l.kind !== "del"), language, map);
  }
  return map;
}

export function highlightLines(lines: string[], language: string): string[] {
  const grammar = language && prism ? prism.languages[language] : undefined;
  if (!grammar || lines.some((line) => line.length > MAX_HIGHLIGHTED_LINE)) {
    return lines.map(esc);
  }
  try {
    const tokens = prism!.tokenize(lines.join("\n"), grammar);
    return splitTokensIntoLines(tokens, lines.length);
  } catch {
    return lines.map(esc);
  }
}

/// Always safe HTML. Prism's output is already escaped and every other branch
/// escapes here, so callers concatenate the result raw and must never re-escape.
export function highlightCode(text: string, language: string): string {
  const grammar = language && prism ? prism.languages[language] : undefined;
  if (!grammar || text.length > MAX_HIGHLIGHTED_LINE) return esc(text);
  const key = `${language} ${text}`;
  const cached = memo.get(key);
  if (cached != null) return cached;
  const marked = highlightLines([text], language)[0] ?? esc(text);
  if (memo.size >= MAX_MEMO_ENTRIES) memo.clear();
  memo.set(key, marked);
  return marked;
}

function mapStream(lines: DiffLine[], language: string, map: Map<DiffLine, string>): void {
  if (!lines.length) return;
  const unmapped = lines.filter((line) => !map.has(line));
  if (!unmapped.length) return;
  const rendered = highlightLines(lines.map((l) => l.text), language);
  lines.forEach((line, index) => {
    if (!map.has(line)) map.set(line, rendered[index]);
  });
}

function splitTokensIntoLines(tokens: (string | PrismToken)[], count: number): string[] {
  const lines: string[] = Array.from({ length: count }, () => "");
  let lineIndex = 0;

  function walk(token: string | PrismToken, activeClasses: string[]): void {
    if (typeof token === "string") {
      const parts = token.split("\n");
      for (let i = 0; i < parts.length; i += 1) {
        if (i > 0) lineIndex += 1;
        if (lineIndex >= lines.length) break;
        if (!parts[i]) continue;
        const escaped = esc(parts[i]);
        lines[lineIndex] += activeClasses.length
          ? `<span class="${activeClasses.join(" ")}">${escaped}</span>`
          : escaped;
      }
      return;
    }
    const tokenClasses = ["token", token.type];
    if (token.alias) {
      if (Array.isArray(token.alias)) tokenClasses.push(...token.alias);
      else tokenClasses.push(token.alias);
    }
    const combined = [...activeClasses, ...tokenClasses];
    if (Array.isArray(token.content)) {
      for (const child of token.content) walk(child, combined);
    } else if (typeof token.content === "string") {
      walk(token.content, combined);
    } else if (token.content) {
      walk(token.content as PrismToken, combined);
    }
  }

  for (const token of tokens) walk(token, []);
  return lines;
}

async function loadCore(): Promise<PrismApi> {
  const host = globalThis as { Prism?: unknown };
  // Read by the core as it runs; without it Prism scans the whole document on
  // load looking for `language-*` elements it will never find here.
  host.Prism = { manual: true };
  const module = (await import("prismjs")) as { default?: PrismApi };
  const core = module.default ?? (module as unknown as PrismApi);
  host.Prism = core;
  return core;
}

async function loadGrammar(language: string): Promise<void> {
  const grammar = GRAMMARS[language];
  if (!grammar || loaded.has(language)) return;
  for (const need of grammar.needs) await loadGrammar(need);
  await grammar.load();
  loaded.add(language);
}

