// Usage: node crates/wasm-bench/bench.mjs   (after building wasm-bench for wasm32)
import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";

const path = new URL("../../target/wasm32-unknown-unknown/release/wasm_bench.wasm", import.meta.url);
const bytes = await readFile(path);
console.log(`module size              ${(bytes.length / 1024 / 1024).toFixed(1)} MiB`);
const { instance } = await WebAssembly.instantiate(bytes, {});
const w = instance.exports;

const time = (label, f) => {
  const t = performance.now();
  const r = f();
  console.log(`${label.padEnd(24)} ${(performance.now() - t).toFixed(1).padStart(8)} ms`);
  return r;
};

time("finalize (bootstrap)", () => w.finalize());
time("seed", () => w.seed(100n));
time("seed", () => w.seed(200n));
time("fuse", () => w.fuse());
const len = time("compress", () => w.compress());
const ok = time("verify_compressed", () => w.verify());
console.log(`compressed proof         ${String(len).padStart(8)} bytes`);
if (ok !== 1) throw new Error("verification failed");
console.log("ok");
