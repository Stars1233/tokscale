#!/usr/bin/env bun
/**
 * Native Runner - Subprocess for non-blocking native Rust calls
 *
 * This script runs in a separate process to keep the main event loop free
 * for UI rendering (e.g., spinner animation).
 *
 * Communication: tmpfile (JSON input) -> tmpfile (JSON output)
 */

import { readFileSync, writeFileSync } from "node:fs";

interface NativeRunnerRequest {
  method: string;
  args: unknown[];
}

async function main() {
  const inputFile = process.argv[2];
  const outputFile = process.argv[3];

  if (!inputFile || !outputFile) {
    process.stderr.write(JSON.stringify({ error: "Usage: native-runner <inputFile> <outputFile>" }));
    process.exit(1);
  }

  // Dynamic import so that load failures are caught and reported as JSON to stderr
  // rather than causing an unhandled crash with no diagnostic output (e.g. on NixOS
  // where the .node binary's dynamic linker path differs from standard Linux paths).
  let nativeCore: typeof import("@tokscale/core");
  try {
    const mod = await import("@tokscale/core");
    // CJS modules wrapped by Bun expose exports as .default; fall back to the namespace.
    nativeCore = ((mod as unknown as { default: typeof mod }).default ?? mod) as typeof import("@tokscale/core");
  } catch (e) {
    const err = e as Error;
    process.stderr.write(JSON.stringify({
      error: `Failed to load native module (@tokscale/core): ${err.message}`,
      stack: err.stack,
    }));
    process.exit(1);
  }

  const input = readFileSync(inputFile, "utf-8");

  let request: NativeRunnerRequest;
  try {
    request = JSON.parse(input) as NativeRunnerRequest;
  } catch (e) {
    throw new Error(`Malformed JSON input: ${(e as Error).message}`);
  }

  const { method, args } = request;

  if (!Array.isArray(args) || args.length === 0) {
    throw new Error(`Invalid args for method '${method}': expected at least 1 argument`);
  }

  let result: unknown;

  switch (method) {
    case "parseLocalClients":
    case "parseLocalSources":
      result = nativeCore.parseLocalClients(args[0] as Parameters<typeof nativeCore.parseLocalClients>[0]);
      break;
    case "finalizeReport":
      result = await nativeCore.finalizeReport(args[0] as Parameters<typeof nativeCore.finalizeReport>[0]);
      break;
    case "finalizeMonthlyReport":
      result = await nativeCore.finalizeMonthlyReport(args[0] as Parameters<typeof nativeCore.finalizeMonthlyReport>[0]);
      break;
    case "finalizeGraph":
      result = await nativeCore.finalizeGraph(args[0] as Parameters<typeof nativeCore.finalizeGraph>[0]);
      break;
    case "finalizeReportAndGraph":
      result = await nativeCore.finalizeReportAndGraph(args[0] as Parameters<typeof nativeCore.finalizeReportAndGraph>[0]);
      break;
    default:
      throw new Error(`Unknown method: ${method}`);
  }

  writeFileSync(outputFile, JSON.stringify(result), "utf-8");
}

main().catch((e) => {
  const error = e as Error;
  process.stderr.write(JSON.stringify({
    error: error.message,
    stack: error.stack,
  }));
  process.exit(1);
});
