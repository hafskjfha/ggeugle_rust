import { parentPort } from 'node:worker_threads';

// Only the test harness contains Node-specific code. Production worker-entry is
// the exact same module loaded by browsers.
globalThis.self = globalThis;
globalThis.postMessage = (message) => parentPort.postMessage(message);
globalThis.close = () => parentPort.close();
globalThis.addEventListener = (type, listener) => {
  if (type === 'message') parentPort.on('message', (data) => listener({ data }));
};
await import('../../dist/worker-entry.js');
