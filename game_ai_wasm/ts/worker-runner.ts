import type { AiOptions, CriticalWordsInfo, EngineFunctions, GameEvent, Graph, NodePos, PrecInfo, SearchEvent, SingleMove, WordSolver } from "./types.js";

export type WorkerMethod = keyof EngineFunctions;
export type WorkerEvent = SearchEvent | GameEvent | CriticalWordsInfo;
/** Callbacks are transported as event messages instead of cloned functions. */
export type WorkerArgs<K extends WorkerMethod> =
  K extends "chooseMove" ? [solver: WordSolver, history: string[], options?: AiOptions] :
  K extends "startStreamingSingleThreadSearch" ? [graph: Graph, move: SingleMove, prec: PrecInfo | undefined, timeoutMillis: number | undefined] :
  K extends "startStreamingCriticalWordsInfo" ? [graph: Graph, view: NodePos, flow: NodePos] :
  K extends "searchRootBranch" ? [graph: Graph, move: SingleMove, prec?: PrecInfo, timeoutMillis?: number] :
  Parameters<EngineFunctions[K]>;
export type WorkerEventLike = {
  data?: unknown;
  error?: unknown;
  message?: string;
  preventDefault?: () => void;
};
export type WorkerListener = (event: WorkerEventLike) => void;

/** The browser Worker boundary; factories may also wrap Node workers. */
export interface WorkerLike {
  addEventListener(type: "message" | "error" | "messageerror", listener: WorkerListener): void;
  removeEventListener(type: "message" | "error" | "messageerror", listener: WorkerListener): void;
  postMessage(message: unknown): void;
  terminate(): void | Promise<unknown>;
}

export type WorkerFactory = (url: URL, options: WorkerOptions) => WorkerLike;
export type WorkerRunnerOptions = { workerFactory?: WorkerFactory; wasmUrl?: string | URL };
export type WorkerRequest = { method: WorkerMethod; args: unknown[]; wasmUrl?: string };
export type WorkerResponse =
  | { type: "event"; event: WorkerEvent }
  | { type: "result"; result: unknown }
  | { type: "error"; error: { name: string; message: string; stack?: string } };

export function abortError(): Error {
  const error = new Error("Worker call cancelled");
  error.name = "AbortError";
  return error;
}

export function timeoutError(): Error {
  const error = new Error("Timeout exceeded");
  error.name = "TimeoutError";
  return error;
}

export function validateTimeout(timeoutMillis?: number): void {
  if (timeoutMillis !== undefined && (!Number.isFinite(timeoutMillis) || timeoutMillis < 0 || timeoutMillis > 2_147_483_647)) {
    throw new RangeError("timeoutMillis must be a finite number between 0 and 2147483647");
  }
}

/** Runs one engine call in a fresh module Worker and cancels any older call. */
export class WorkerRunner {
  private readonly workerFactory: WorkerFactory;
  private readonly wasmUrl?: string;
  private cancelPending?: () => void;

  constructor(options: WorkerRunnerOptions = {}) {
    this.workerFactory = options.workerFactory ?? ((url, workerOptions) => new Worker(url, workerOptions) as unknown as WorkerLike);
    if (options.wasmUrl !== undefined) {
      const base = typeof location === "undefined" ? import.meta.url : location.href;
      this.wasmUrl = new URL(options.wasmUrl, base).href;
    }
  }

  terminate(): void {
    this.cancelPending?.();
  }

  callAndTerminate<K extends WorkerMethod>(
    method: K,
    args: WorkerArgs<K>,
    timeoutMillis?: number,
    onEvent?: (event: WorkerEvent) => void,
  ): Promise<Awaited<ReturnType<EngineFunctions[K]>>> {
    this.terminate();
    return new Promise((resolve, reject) => {
      validateTimeout(timeoutMillis);
      const deadline = timeoutMillis === undefined ? undefined : performance.now() + timeoutMillis;
      const worker = this.workerFactory(new URL("./worker-entry.js", import.meta.url), { type: "module" });
      let settled = false;
      let timeout: ReturnType<typeof setTimeout> | undefined;

      const cleanup = () => {
        if (timeout !== undefined) clearTimeout(timeout);
        worker.removeEventListener("message", onMessage);
        worker.removeEventListener("error", onError);
        worker.removeEventListener("messageerror", onMessageError);
        if (this.cancelPending === cancel) this.cancelPending = undefined;
        try {
          const termination = worker.terminate();
          if (termination !== undefined) void Promise.resolve(termination).catch(() => {});
        } catch {
          // Cleanup must not replace the engine result or rejection.
        }
      };

      const fail = (error: unknown) => {
        if (settled) return;
        settled = true;
        cleanup();
        reject(error);
      };
      const cancel = () => fail(abortError());

      const onMessage: WorkerListener = ({ data }) => {
        if (settled) return;
        if (deadline !== undefined && performance.now() >= deadline) {
          fail(timeoutError());
          return;
        }
        if (data === null || typeof data !== "object" || !("type" in data)) {
          fail(new Error("Invalid worker response"));
          return;
        }
        const response = data as WorkerResponse;
        if (response.type === "event") {
          try {
            onEvent?.(response.event);
            if (settled) return;
            if (deadline !== undefined && performance.now() >= deadline) fail(timeoutError());
          } catch (error) {
            fail(error);
          }
        } else if (response.type === "error") {
          if (!response.error || typeof response.error.message !== "string") {
            fail(new Error("Invalid worker error response"));
            return;
          }
          const error = new Error(response.error.message);
          error.name = response.error.name;
          if (response.error.stack !== undefined) error.stack = response.error.stack;
          fail(error);
        } else if (response.type === "result") {
          settled = true;
          cleanup();
          resolve(response.result as Awaited<ReturnType<EngineFunctions[K]>>);
        } else {
          fail(new Error("Invalid worker response"));
        }
      };
      const onError: WorkerListener = (event) => {
        event.preventDefault?.();
        fail(event.error ?? new Error(event.message ?? "Worker failed"));
      };
      const onMessageError: WorkerListener = () => fail(new Error("Worker message could not be deserialized"));

      this.cancelPending = cancel;
      try {
        worker.addEventListener("message", onMessage);
        worker.addEventListener("error", onError);
        worker.addEventListener("messageerror", onMessageError);
        if (deadline !== undefined) timeout = setTimeout(() => fail(timeoutError()), Math.max(0, deadline - performance.now()));
        const request: WorkerRequest = { method, args };
        if (this.wasmUrl !== undefined) request.wasmUrl = this.wasmUrl;
        worker.postMessage(request);
      } catch (error) {
        fail(error);
      }
    });
  }
}
