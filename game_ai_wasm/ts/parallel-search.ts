import { DEFAULT_PRECEDENCE } from "./engine.js";
import type { Graph, GraphSolver, PrecInfo, RootBranchResult, RootSearchPlan, SearchEvent, SearchResult, SingleMove } from "./types.js";
import { abortError, timeoutError, validateTimeout, WorkerRunner, type WorkerRunnerOptions } from "./worker-runner.js";

export type ParallelSearchEvent = SearchEvent & { branchIndex: number | null };
export type ParallelSearchOptions = {
  workers?: number;
  timeoutMillis?: number;
  signal?: AbortSignal;
  onEvent?: (event: ParallelSearchEvent) => void;
};

type SearchSource =
  | { kind: "move"; graph: Graph; move: SingleMove }
  | { kind: "syllable"; solver: GraphSolver; syllable: string; changeFuncIdx: number };

/** Coordinates independent Rust response branches in bounded browser workers. */
export class ParallelSearchRunner {
  private readonly workerOptions: WorkerRunnerOptions;
  private cancelPending?: () => void;

  constructor(options: WorkerRunnerOptions = {}) {
    this.workerOptions = options;
  }

  terminate(): void {
    this.cancelPending?.();
  }

  search(
    graph: Graph,
    move: SingleMove,
    prec: PrecInfo = DEFAULT_PRECEDENCE,
    options: ParallelSearchOptions = {},
  ): Promise<SearchResult> {
    return this.runSearch({ kind: "move", graph, move }, prec, options);
  }

  /** Judge the player who must start with this syllable, including missing-tail nodes. */
  searchSyllable(
    solver: GraphSolver,
    syllable: string,
    changeFuncIdx = 0,
    prec: PrecInfo = DEFAULT_PRECEDENCE,
    options: ParallelSearchOptions = {},
  ): Promise<SearchResult> {
    return this.runSearch({ kind: "syllable", solver, syllable, changeFuncIdx }, prec, options);
  }

  private runSearch(source: SearchSource, prec: PrecInfo, options: ParallelSearchOptions): Promise<SearchResult> {
    this.terminate();
    return new Promise((resolve, reject) => {
      const defaultWorkers = typeof navigator === "undefined" ? 2 : Math.max(1, Math.min(4, navigator.hardwareConcurrency || 2));
      const workerCount = options.workers ?? defaultWorkers;
      if (!Number.isInteger(workerCount) || workerCount < 1 || workerCount > 64) {
        throw new RangeError("workers must be an integer between 1 and 64");
      }
      validateTimeout(options.timeoutMillis);
      if (options.signal?.aborted) throw abortError();

      const started = performance.now();
      const deadline = options.timeoutMillis === undefined ? undefined : started + options.timeoutMillis;
      const runners = new Set<WorkerRunner>();
      let settled = false;
      let finalizing = false;
      let timeout: ReturnType<typeof setTimeout> | undefined;
      const remaining = () => deadline === undefined ? undefined : Math.max(0, deadline - performance.now());

      const cleanup = () => {
        if (timeout !== undefined) clearTimeout(timeout);
        options.signal?.removeEventListener("abort", cancel);
        if (this.cancelPending === cancel) this.cancelPending = undefined;
        for (const runner of runners) runner.terminate();
        runners.clear();
      };
      const fail = (error: unknown) => {
        if (settled) return;
        settled = true;
        cleanup();
        reject(error);
      };
      const cancel = () => fail(abortError());
      const checkDeadline = () => {
        if (deadline !== undefined && performance.now() >= deadline) throw timeoutError();
      };
      const emit = (event: SearchEvent, branchIndex: number | null) => {
        if (!settled) {
          checkDeadline();
          if (source.kind === "syllable" && event.action === "stack") {
            event = { ...event, payload: event.payload.filter(([head]) => head !== "__none") };
          }
          options.onEvent?.({ ...event, branchIndex });
        }
      };
      const complete = (result: SearchResult) => {
        if (settled) return;
        try {
          checkDeadline();
        } catch (error) {
          fail(error);
          return;
        }
        // Release this request before callbacks can start a replacement search.
        settled = true;
        cleanup();
        if (source.kind === "syllable") {
          result = {
            ...result,
            isWin: !result.isWin,
            optimalPath: result.optimalPath.slice(1),
            visited: Math.max(0, result.visited - 1),
            duration: performance.now() - started,
          };
        }
        try {
          options.onEvent?.({ action: "done", payload: result, branchIndex: null });
          resolve(result);
        } catch (error) {
          reject(error);
        }
      };
      const newRunner = () => {
        const runner = new WorkerRunner(this.workerOptions);
        runners.add(runner);
        return runner;
      };

      this.cancelPending = cancel;
      options.signal?.addEventListener("abort", cancel, { once: true });
      if (options.timeoutMillis !== undefined) timeout = setTimeout(() => fail(timeoutError()), options.timeoutMillis);

      const runBranches = (plan: RootSearchPlan) => {
        if (settled) return;
        checkDeadline();
        if (plan.result !== null) {
          complete(plan.result);
          return;
        }
        const results: (RootBranchResult | null)[] = plan.moves.map(() => null);
        let nextIndex = 0;
        let prefix = 0;
        let winningIndex = Infinity;

        const finalize = () => {
          if (settled || finalizing) return;
          checkDeadline();
          finalizing = true;
          for (const runner of runners) runner.terminate();
          runners.clear();
          const finalizer = newRunner();
          finalizer.callAndTerminate("finishRootSearch", [plan, results, performance.now() - started], remaining()).then(complete, fail);
        };

        const dispatch = (runner: WorkerRunner) => {
          if (settled || finalizing || nextIndex >= plan.moves.length || nextIndex > winningIndex) return;
          checkDeadline();
          const index = nextIndex++;
          runner.callAndTerminate(
            "searchRootBranch",
            [plan.graph, plan.moves[index]!, prec, remaining()],
            remaining(),
            (event) => {
              if ("action" in event && (event.action === "stack" || event.action === "done")) emit(event, index);
            },
          ).then((result) => {
            if (settled || finalizing) return;
            results[index] = result;
            if (result.result.isWin) winningIndex = Math.min(winningIndex, index);
            while (prefix < results.length && results[prefix] !== null) {
              if (results[prefix]!.result.isWin) {
                // Only this ordered prefix participates in the Rust merge.
                for (let tail = prefix + 1; tail < results.length; tail++) results[tail] = null;
                finalize();
                return;
              }
              prefix++;
            }
            if (prefix === results.length) finalize();
            else dispatch(runner);
          }).catch((error) => {
            if (!settled && !finalizing) fail(error);
          });
        };

        if (plan.moves.length === 0) finalize();
        else for (let slot = 0; slot < Math.min(workerCount, plan.moves.length); slot++) dispatch(newRunner());
      };

      const planner = newRunner();
      if (source.kind === "syllable") {
        planner.callAndTerminate("prepareSyllableSearch",
          [source.solver, source.syllable, source.changeFuncIdx, prec, remaining()], remaining()).then(runBranches).catch(fail);
      } else {
        planner.callAndTerminate("prepareRootSearch", [source.graph, source.move, prec, remaining()], remaining()).then(runBranches).catch(fail);
      }
    });
  }
}
