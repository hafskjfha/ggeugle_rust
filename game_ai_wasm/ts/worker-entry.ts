import { createEngineFunctions } from "./engine.js";
import type { WorkerEvent, WorkerRequest, WorkerResponse } from "./worker-runner.js";

const scope = self as unknown as {
  addEventListener(type: "message", listener: (event: MessageEvent<WorkerRequest>) => void, options: { once: boolean }): void;
  postMessage(response: WorkerResponse): void;
  close(): void;
};

scope.addEventListener("message", (event) => {
  void (async () => {
    try {
      const { method, args, wasmUrl } = event.data;
      const engine = await createEngineFunctions(wasmUrl);
      if (!Object.hasOwn(engine, method) || typeof engine[method] !== "function") {
        throw new Error(`Unsupported worker method: ${String(method)}`);
      }
      const emit = (event: WorkerEvent) => scope.postMessage({ type: "event", event });
      const callable = engine[method] as (...args: unknown[]) => unknown;
      const suppliedArgs = [...args];
      if (method === "startStreamingSingleThreadSearch" || method === "searchRootBranch") suppliedArgs[4] = emit;
      if (method === "chooseMove" || method === "startStreamingCriticalWordsInfo") suppliedArgs[3] = emit;
      const result = await callable(...suppliedArgs);
      scope.postMessage({ type: "result", result });
    } catch (cause) {
      const error = cause instanceof Error ? cause : new Error(String(cause));
      scope.postMessage({ type: "error", error: { name: error.name, message: error.message, stack: error.stack } });
    } finally {
      scope.close();
    }
  })();
}, { once: true });
