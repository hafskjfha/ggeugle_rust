import { Worker } from "node:worker_threads";
import type { EngineFunctions } from "./engine-functions.js";

export type WorkerMethod = "getWcData" | "updateSolver" | "searchIsWin";
type WorkerApi = Pick<EngineFunctions, WorkerMethod>;

export type WorkerRequest = {
  method: WorkerMethod;
  args: unknown[];
};

export type WorkerResponse =
  | { type: "result"; result: unknown }
  | { type: "error"; error: { name: string; message: string; stack?: string } };

/** Runs one engine call per worker, cancelling any previous pending call. */
export class WorkerRunner {
  private cancelPending?: () => void;

  terminate(): void {
    this.cancelPending?.();
  }

  callAndTerminate<K extends WorkerMethod>(
    method: K,
    args: Parameters<WorkerApi[K]>,
    timeoutMillis?: number,
  ): Promise<Awaited<ReturnType<WorkerApi[K]>>> {
    this.terminate();

    return new Promise((resolve, reject) => {
      const worker = new Worker(new URL("./worker-entry.js", import.meta.url));
      let settled = false;
      let timeout: ReturnType<typeof setTimeout> | undefined;

      const cleanup = () => {
        if (timeout !== undefined) clearTimeout(timeout);
        worker.off("message", onMessage);
        worker.off("error", onError);
        worker.off("exit", onExit);
        if (this.cancelPending === cancel) this.cancelPending = undefined;
        void worker.terminate().catch(() => {});
      };

      const fail = (error: unknown) => {
        if (settled) return;
        settled = true;
        cleanup();
        reject(error);
      };

      const cancel = () => {
        const error = new Error("Worker call cancelled");
        error.name = "AbortError";
        fail(error);
      };

      const onMessage = (response: WorkerResponse) => {
        if (settled) return;
        if (response.type === "error") {
          const error = new Error(response.error.message);
          error.name = response.error.name;
          if (response.error.stack !== undefined) error.stack = response.error.stack;
          fail(error);
          return;
        }
        settled = true;
        cleanup();
        resolve(response.result as Awaited<ReturnType<WorkerApi[K]>>);
      };

      const onError = (error: Error) => fail(error);
      const onExit = (code: number) =>
        fail(new Error(`Worker exited before returning a result (code ${code})`));

      this.cancelPending = cancel;
      worker.once("message", onMessage);
      worker.once("error", onError);
      worker.once("exit", onExit);

      if (timeoutMillis !== undefined) {
        timeout = setTimeout(() => fail(new Error("Timeout exceeded")), timeoutMillis);
      }

      try {
        worker.postMessage({ method, args } satisfies WorkerRequest);
      } catch (error) {
        fail(error);
      }
    });
  }
}
