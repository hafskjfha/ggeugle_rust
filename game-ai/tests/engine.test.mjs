import assert from "node:assert/strict";
import test from "node:test";

// A failed standalone import exposes any remaining frontend or Vite dependency.
const loaded = await import("../dist/index.js").catch((error) => ({ error }));

test("AI package loads in Node without a frontend", () => {
  assert.equal(loaded.error, undefined);
  assert.equal(typeof loaded.engineFunctions?.getWcData, "function");
});

if (!loaded.error) {
  const { engineFunctions, GameWorkerRunner, isGameEnd, sampleChangeFuncs, loadWords } = loaded;
  const prec = { rule: 0, maps: { edge: {}, node: {} } };
  const makeRule = (words, changeFuncIdx = 0) => ({
    id: "test",
    metadata: { title: "test", color: "", updatedAt: 0 },
    content: {
      wordRule: {
        words: { type: "manual", option: { content: words.join(" ") } },
        regexFilter: ".*",
        addedWords: "",
        removedWords: "",
      },
      wordConnectionRule: { changeFuncIdx, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
      postprocessing: { manner: { type: 0 }, addedWords: "", removedWords: "" },
    },
  });

  test("manual dictionary keeps filtered unique words and additions", async () => {
    const rule = makeRule(["사과", "사과", "과자", "a", ""]).content.wordRule;
    rule.regexFilter = "[가-힣]{2}";
    rule.removedWords = "과자";
    rule.addedWords = "자두 사과";
    assert.deepEqual(await loadWords(rule), ["사과", "자두"]);
  });

  test("standard initial-sound rule connects 량 to 양", () => {
    assert.ok(sampleChangeFuncs[1].forward("량").includes("양"));
    assert.ok(sampleChangeFuncs[1].backward("양").includes("량"));
  });

  test("consonant reversal connects 강 to 악 in Node ESM", async () => {
    assert.deepEqual(sampleChangeFuncs[8].forward("강"), ["강", "악"]);
    assert.deepEqual(sampleChangeFuncs[8].backward("강"), ["강", "악"]);
    const solver = await engineFunctions.getWcData(makeRule(["가강", "악어"], 8), 0);
    assert.deepEqual(solver.getNextWords(["가강"]), ["악어"]);
  });

  test("worker search preserves the mover's win and loss meaning", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["사과", "과자", "자두"]), 0);
    const graph = solver.graphSolver.graphs.union();
    assert.equal(engineFunctions.searchIsWin(graph, ["사", "과"], prec).isWin, true);
    assert.equal(engineFunctions.searchIsWin(graph, ["과", "자"], prec).isWin, false);
  });

  test("direct solver updates leave the original dictionary usable", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["사과", "과자", "자두"]), 0);
    const updated = engineFunctions.updateSolver(solver.graphSolver.graphs, [["사", "과", 1]], 0);
    assert.equal(updated.graphs.union().getEdgeNum("사", "과"), 0);
    assert.deepEqual(solver.getNextWords(["사과"]), ["과자"]);
    assert.deepEqual(solver.getNextWords([]).sort(), ["과자", "사과", "자두"]);
  });

  test("streamed stack events keep their search path after completion", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["사과", "과자", "자두"]), 0);
    const events = [];
    engineFunctions.startStreamingSingleThreadSearch(
      (event) => events.push(event), solver.graphSolver.graphs.union(), ["사", "과"], prec, 0,
    );
    assert.deepEqual(events.find((event) => event.action === "stack").payload, [["사", "과"]]);
    assert.equal(events.at(-1).action, "done");
    assert.equal(events.at(-1).payload.isWin, true);
  });

  test("history consumes words individually when syllable pairs match", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["가나", "가가나", "나가"]), 0);
    assert.equal(isGameEnd(solver, ["가나", "나가"], false), false);
    assert.equal(isGameEnd(solver, ["가나", "나가", "가가나"], false), true);
    assert.equal(isGameEnd(solver, ["가나", "나가", "가가나"], true), true);
  });

  for (const flow of [0, 1]) {
    test(`flow ${flow} classifies a forced three-move chain`, async () => {
      const solver = await engineFunctions.getWcData(makeRule(["사과", "과자", "자두"]), flow);
      // 사 only appears as a word head, so it belongs to graph position 1.
      assert.equal(solver.graphSolver.getNodeType("사", 1), "win");
      assert.equal(solver.graphSolver.getNodeType("과", 0), "lose");
      assert.equal(solver.graphSolver.getNodeType("자", 0), "win");
      assert.equal(solver.graphSolver.getNodeType("두", 0), "lose");
    });
  }

  test("empty dictionary is already terminal", async () => {
    const solver = await engineFunctions.getWcData(makeRule([]), 0);
    assert.equal(isGameEnd(solver, [], false), true);
  });

  test("hard AI selects a legal cyclic move when its search times out", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["가나", "나다", "다가"]), 0);
    assert.equal(solver.graphSolver.getNodeType("가", 0), "route");
    const events = [];
    const runner = new GameWorkerRunner(solver, 2, 0, false, [], prec, (event) => events.push(event), "timeout");
    try {
      await runner.run(0);
      const selected = events.find((event) => event.action === "move").payload;
      assert.ok(["가나", "나다", "다가"].includes(selected));
      assert.equal(events.at(-1).action, "messageEnd");
    } finally {
      runner.terminate();
    }
  });

  for (const difficulty of [0, 1, 2]) {
    test(`difficulty ${difficulty} chooses a legal remaining move through Node workers`, async () => {
      const solver = await engineFunctions.getWcData(makeRule(["사과", "과자", "자두"]), 0);
      const events = [];
      const runner = new GameWorkerRunner(solver, difficulty, 1, false, ["사과"], prec, (event) => events.push(event), "test");
      try {
        await runner.run(0);
        assert.deepEqual(events.filter((event) => event.action === "move"), [{ action: "move", payload: "과자" }]);
        assert.equal(events.at(-1).action, "messageEnd");
      } finally {
        runner.terminate();
      }
    });
  }

  test("stealing the first word and terminal win events remain available", async () => {
    const solver = await engineFunctions.getWcData(makeRule(["사과"]), 0);
    assert.equal(isGameEnd(solver, ["사과"], false), true);
    assert.equal(isGameEnd(solver, ["사과"], true), false);
    const events = [];
    const runner = new GameWorkerRunner(solver, 1, 1, true, ["사과"], prec, (event) => events.push(event), "steal");
    try {
      await runner.run(0);
      assert.deepEqual(events.filter((event) => event.action !== "debug").map((event) => event.action), ["move", "computerWin", "messageEnd"]);
      assert.equal(events.find((event) => event.action === "move").payload, "사과");
    } finally {
      runner.terminate();
    }
  });
}
