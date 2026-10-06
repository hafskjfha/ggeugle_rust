import { Worker } from 'node:worker_threads';

/** Adapt real Node worker threads to the browser Worker boundary under test. */
export function nodeWorkerFactory(_url, _options) {
  const worker = new Worker(new URL('./node-worker-entry.mjs', import.meta.url));
  const listeners = new Map();
  return {
    postMessage: (message) => worker.postMessage(message),
    terminate: () => { void worker.terminate(); },
    addEventListener(type, listener) {
      const nodeType = type === 'messageerror' ? 'messageerror' : type;
      const wrapped = type === 'message' ? (data) => listener({ data }) :
        (error) => listener({ error, message: error.message });
      listeners.set(listener, { nodeType, wrapped });
      worker.on(nodeType, wrapped);
    },
    removeEventListener(_type, listener) {
      const registration = listeners.get(listener);
      if (registration) worker.off(registration.nodeType, registration.wrapped);
      listeners.delete(listener);
    },
  };
}
