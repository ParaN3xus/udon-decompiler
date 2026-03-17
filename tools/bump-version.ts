import fs from "node:fs";
import path from "node:path";
import { edit, init } from "@rainbowatcher/toml-edit-js";

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

function requiredArg(args: Map<string, string>, name: string): string {
  const value = args.get(name);
  if (!value) {
    throw new Error(`missing required argument: ${name}`);
  }
  return value;
}

function validateVersion(version: string): void {
  if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`invalid version: ${version}`);
  }
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const version = requiredArg(args, "--version");
  const filePath = args.get("--file") ?? "Cargo.toml";

  validateVersion(version);

  const input = fs.readFileSync(filePath, "utf8");
  await init();
  const output = edit(input, "package.version", version);

  fs.writeFileSync(path.resolve(filePath), output, "utf8");
}

await main();
