import { parentPort } from "node:worker_threads";
import { engineFunctions } from "./engine-functions.js";
import type { WorkerRequest, WorkerResponse } from "./worker-runner.js";

const port = parentPort;
if (port === null) throw new Error("The engine worker requires a parent port");

port.once("message", async ({ method, args }: WorkerRequest) => {
  try {
    let result: unknown;
    switch (method) {
      case "getWcData":
        result = await engineFunctions.getWcData(
          ...(args as Parameters<typeof engineFunctions.getWcData>),
        );
        break;
      case "updateSolver":
        result = engineFunctions.updateSolver(
          ...(args as Parameters<typeof engineFunctions.updateSolver>),
        );
        break;
      case "searchIsWin":
        result = engineFunctions.searchIsWin(
          ...(args as Parameters<typeof engineFunctions.searchIsWin>),
        );
        break;
      default:
        throw new Error(`Unsupported worker method: ${String(method)}`);
    }
    port.postMessage({ type: "result", result } satisfies WorkerResponse);
  } catch (cause) {
    const error = cause instanceof Error ? cause : new Error(String(cause));
    port.postMessage({
      type: "error",
      error: { name: error.name, message: error.message, stack: error.stack },
    } satisfies WorkerResponse);
  } finally {
    port.close();
  }
});
