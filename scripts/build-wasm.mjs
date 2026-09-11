import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// ツールのキャッシュもビルド生成物としてtarget内に置く。
const root = fileURLToPath(new URL('../', import.meta.url));
const cache = fileURLToPath(new URL('../target/wasm-pack-cache', import.meta.url));
const result = spawnSync('wasm-pack', [
  'build', 'crates/wasm', '--target', 'web',
  '--out-dir', '../../web/pkg', process.argv[2] || '--release', '--locked',
], {
  cwd: root,
  env: { ...process.env, WASM_PACK_CACHE: process.env.WASM_PACK_CACHE || cache },
  stdio: 'inherit',
});

if (result.error) console.error(`wasm-packを起動できませんでした: ${result.error.message}`);
process.exit(result.status ?? 1);
