import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const local = path.join(root, '.tools', 'bin', process.platform === 'win32' ? 'wasm-bindgen.exe' : 'wasm-bindgen');
const bindgen = process.env.WASM_BINDGEN ?? (existsSync(local) ? local : 'wasm-bindgen');
function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit' });
  if (result.error) {
    throw new Error(`${command}: ${result.error.message}\nInstall the wasm32 target and wasm-bindgen-cli as described in README.md.`);
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run('cargo', ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown']);
mkdirSync(path.join(root, 'pkg'), { recursive: true });
run(bindgen, [path.join(root, 'target', 'wasm32-unknown-unknown', 'release', 'game_ai_wasm.wasm'),
  '--target', 'web', '--out-dir', path.join(root, 'pkg'), '--out-name', 'game_ai_wasm']);
