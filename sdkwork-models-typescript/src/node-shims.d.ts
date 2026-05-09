declare module "node:fs/promises" {
  export function readFile(path: string, encoding: "utf8"): Promise<string>;
  export function readdir(path: string): Promise<string[]>;
}

declare module "node:path" {
  export function join(...paths: string[]): string;
}
