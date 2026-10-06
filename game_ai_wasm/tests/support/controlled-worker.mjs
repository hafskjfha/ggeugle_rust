export class ControlledWorker {
  listeners = new Map();
  request;
  terminated = false;

  constructor(onRequest) {
    this.onRequest = onRequest;
  }

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  postMessage(request) {
    this.request = structuredClone(request);
    this.onRequest?.(this, this.request);
  }

  terminate() {
    this.terminated = true;
  }

  dispatch(type, event) {
    for (const listener of [...(this.listeners.get(type) ?? [])]) listener(event);
  }

  result(result) {
    this.dispatch('message', { data: { type: 'result', result } });
  }

  event(event) {
    this.dispatch('message', { data: { type: 'event', event } });
  }

  error(name = 'RangeError', message = 'engine failed') {
    this.dispatch('message', { data: { type: 'error', error: { name, message, stack: 'worker stack' } } });
  }
}

export function controlledFactory(onRequest) {
  const workers = [];
  const factory = (url, options) => {
    const worker = new ControlledWorker(onRequest);
    worker.url = url;
    worker.options = options;
    workers.push(worker);
    return worker;
  };
  return { workers, factory };
}

export async function until(predicate) {
  for (let attempt = 0; attempt < 100; attempt++) {
    const value = predicate();
    if (value) return value;
    await new Promise((resolve) => setImmediate(resolve));
  }
  throw new Error('Expected worker operation did not start');
}

export function nextRequest(workers, method, seen = new Set()) {
  return until(() => {
    const worker = workers.find((worker) => worker.request?.method === method && !seen.has(worker));
    if (worker) seen.add(worker);
    return worker;
  });
}
