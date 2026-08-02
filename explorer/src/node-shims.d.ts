/**
 * Minimal ambient declarations for the tiny slice of the Node.js API the
 * explorer CLI uses. This lets the project type-check and compile fully
 * offline with a stock `tsc`, without downloading `@types/node`.
 *
 * These declarations are intentionally narrow — only what `cli.ts` touches.
 */

declare module "node:fs" {
  export function readFileSync(path: string, encoding: "utf8"): string;
}

declare module "node:process" {
  const proc: {
    readonly argv: string[];
    exit(code?: number): never;
    readonly stdout: { write(s: string): boolean };
    readonly stderr: { write(s: string): boolean };
  };
  export default proc;
}

declare const console: {
  log(...args: unknown[]): void;
  error(...args: unknown[]): void;
};

declare module "node:test" {
  export function test(name: string, fn: () => void | Promise<void>): void;
}

declare module "node:assert/strict" {
  const assert: {
    (value: unknown, message?: string): asserts value;
    equal(actual: unknown, expected: unknown, message?: string): void;
    deepEqual(actual: unknown, expected: unknown, message?: string): void;
    ok(value: unknown, message?: string): asserts value;
    throws(fn: () => unknown, message?: string): void;
  };
  export default assert;
}
