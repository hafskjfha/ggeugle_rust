import { engineFunctions, GameWorkerRunner, isGameEnd } from "../dist/index.js";

const rule = {
  id: "example",
  metadata: { title: "예제 끝말잇기", updatedAt: 0, color: "" },
  content: {
    wordRule: {
      words: { type: "manual", option: { content: "사과 과자 자두 두부 부자" } },
      regexFilter: ".*",
      addedWords: "",
      removedWords: "",
    },
    wordConnectionRule: {
      changeFuncIdx: 0,
      rawHeadIdx: 1,
      headDir: 0,
      rawTailIdx: 1,
      tailDir: 1,
    },
    postprocessing: { manner: { type: 0 }, addedWords: "", removedWords: "" },
  },
};

const flow = 0;
const solver = await engineFunctions.getWcData(rule, flow);
const history = ["사과"];
const precedence = { rule: 0, maps: { edge: {}, node: {} } };

if (!isGameEnd(solver, history, false)) {
  const runner = new GameWorkerRunner(
    solver,
    2, // 난이도: 0 쉬움, 1 보통, 2 어려움
    1, // 후보 수 하나당 탐색 제한(초)
    false, // 첫 단어 뺏기 허용 여부
    history,
    precedence,
    (event) => {
      if (event.action === "move") console.log(`AI 선택: ${event.payload}`);
      if (event.action === "computerWin") console.log("AI 승리");
      if (event.action === "messageEnd") console.log("수 선택 완료");
    },
    "example",
  );
  try {
    await runner.run(flow);
  } finally {
    runner.terminate();
  }
}
