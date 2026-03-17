import fs from "node:fs";
import path from "node:path";

const SKIP_COMPILE_DIRECTIVE = "skip-compile";
const CODE_FENCE_RE = /```([^\n]*)\n(.*?)\n```/gs;
const DIRECTIVE_RE = /<!--\s*ci\s*:\s*([^>]*)-->/gi;

type MarkdownBlock = {
  lang: string;
  content: string;
  start: number;
  end: number;
};

type CompilationRequest = {
  className: string;
  sourcePath: string;
  sourceCode: string;
};

type CompilerInputData = {
  requests: CompilationRequest[];
};

type CompilerOutputData = {
  results?: string[];
  error?: string | null;
};

function loadCases(root: string): string[] {
  if (!fs.existsSync(root)) {
    return [];
  }

  const out: string[] = [];

  function walk(dir: string): void {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(entryPath);
        continue;
      }
      if (!entry.isFile()) {
        continue;
      }
      if (!entry.name.endsWith(".md") || entry.name === "README.md") {
        continue;
      }
      out.push(entryPath);
    }
  }

  walk(root);
  out.sort();
  return out;
}

function parseCaseDirectives(text: string): Set<string> {
  const directives = new Set<string>();
  for (const match of text.matchAll(DIRECTIVE_RE)) {
    const raw = match[1] ?? "";
    for (const token of raw.trim().split(/[,\s]+/)) {
      if (token) {
        directives.add(token.toLowerCase());
      }
    }
  }
  return directives;
}

function parseMarkdownCases(text: string, casePath: string): MarkdownBlock[] {
  const blocks: MarkdownBlock[] = [];
  for (const match of text.matchAll(CODE_FENCE_RE)) {
    const full = match[0];
    const lang = (match[1] ?? "").trim();
    const content = match[2] ?? "";
    const start = match.index ?? 0;
    blocks.push({
      lang,
      content,
      start,
      end: start + full.length,
    });
  }

  if (blocks.length !== 1 && blocks.length !== 2) {
    throw new Error(
      `${casePath}: expected 1 or 2 code fences, found ${blocks.length}`,
    );
  }

  return blocks;
}

function ensureTrailingNewline(text: string): string {
  return text.endsWith("\n") ? text : `${text}\n`;
}

function parseArgs(argv: string[]): Map<string, string> {
  const args = new Map<string, string>();
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    if (!key.startsWith("--")) {
      continue;
    }
    const value = argv[i + 1];
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`missing value for ${key}`);
    }
    args.set(key, value);
    i += 1;
  }
  return args;
}

function buildInput(casesRoot: string, outputPath: string): void {
  const cases = loadCases(casesRoot);
  if (cases.length === 0) {
    throw new Error(`No case markdown files found under ${casesRoot}`);
  }

  const requests: CompilationRequest[] = [];
  for (const casePath of cases) {
    const text = fs.readFileSync(casePath, "utf8");
    const directives = parseCaseDirectives(text);
    if (directives.has(SKIP_COMPILE_DIRECTIVE)) {
      continue;
    }

    const blocks = parseMarkdownCases(text, casePath);
    requests.push({
      className: path.basename(casePath, ".md"),
      sourcePath: casePath,
      sourceCode: ensureTrailingNewline(blocks[0].content),
    });
  }

  const payload: CompilerInputData = { requests };
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}

function updateMarkdown(inputPath: string, outputPath: string): void {
  const inputData = JSON.parse(
    fs.readFileSync(inputPath, "utf8"),
  ) as CompilerInputData;
  const outputData = JSON.parse(
    fs.readFileSync(outputPath, "utf8"),
  ) as CompilerOutputData;

  if (outputData.error) {
    throw new Error(`Compilation error: ${outputData.error}`);
  }

  const requests = inputData.requests ?? [];
  const results = outputData.results ?? [];
  if (requests.length !== results.length) {
    throw new Error(
      `Request/result count mismatch: ${requests.length} requests vs ${results.length} results`,
    );
  }

  for (let i = 0; i < requests.length; i += 1) {
    const request = requests[i];
    const compiledHex = results[i];
    const casePath = request.sourcePath;
    const text = fs.readFileSync(casePath, "utf8");
    const directives = parseCaseDirectives(text);
    if (directives.has(SKIP_COMPILE_DIRECTIVE)) {
      continue;
    }

    const blocks = parseMarkdownCases(text, casePath);
    const normalizedHex = ensureTrailingNewline(compiledHex ?? "");
    const newBlock = `\`\`\`hex\n${normalizedHex}\`\`\``;

    let updated: string;
    if (blocks.length === 1) {
      updated = `${text.trimEnd()}\n\n${newBlock}\n`;
    } else {
      const target = blocks[1];
      updated = `${text.slice(0, target.start)}${newBlock}${text.slice(target.end)}`;
    }

    fs.writeFileSync(casePath, updated, "utf8");
  }
}

function main(): void {
  const [command, ...rest] = process.argv.slice(2);
  if (!command) {
    throw new Error("expected command: build-input | update-md");
  }

  const args = parseArgs(rest);
  if (command === "build-input") {
    buildInput(
      args.get("--cases") ?? "tests/cases",
      requiredArg(args, "--output"),
    );
    return;
  }
  if (command === "update-md") {
    updateMarkdown(requiredArg(args, "--input"), requiredArg(args, "--output"));
    return;
  }

  throw new Error(`unknown command: ${command}`);
}

function requiredArg(args: Map<string, string>, name: string): string {
  const value = args.get(name);
  if (!value) {
    throw new Error(`missing required argument: ${name}`);
  }
  return value;
}

main();
