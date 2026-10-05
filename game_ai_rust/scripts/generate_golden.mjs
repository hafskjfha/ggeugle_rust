// Build game-ai first: cd game-ai && npm run build
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, resolve } from "node:path";
import { createHash } from "node:crypto";

const crateRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixtureRoot = resolve(process.argv[2] ?? resolve(crateRoot, "tests/fixtures"));
const original = await import(pathToFileURL(resolve(crateRoot, "../game-ai/dist/index.js")));
const { engineFunctions, sampleChangeFuncs, isGameEnd } = original;
const fixtureBytes = readFileSync(resolve(fixtureRoot, "words.txt"));
const words = fixtureBytes.toString("utf8").trim().split(/\s+/);
const provenance = JSON.parse(readFileSync(resolve(fixtureRoot, "provenance.json"), "utf8"));
const ordinal = (left, right) => left < right ? -1 : left > right ? 1 : 0;
const sortedMap = map => [...map].sort(([a], [b]) => ordinal(a, b));
const sortedEdges = edges => edges.sort((a, b) => ordinal(a[0], b[0]) || ordinal(a[1], b[1]));
const precedence = { rule: 0, maps: { edge: {}, node: {} } };
const makeRule = (inputWords, changeFuncIdx = 0) => ({
  id: "parity",
  metadata: { title: "TypeScript parity fixture", updatedAt: 0, color: "" },
  content: {
    wordRule: { words: { type: "manual", option: { content: inputWords.join(" ") } }, regexFilter: ".*", removedWords: "", addedWords: "" },
    wordConnectionRule: { changeFuncIdx, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
    postprocessing: { manner: { type: 0 }, removedWords: "", addedWords: "" },
  },
});

const cases = [];
async function addCase(name, rule, flow, histories = [], search = false) {
  const solver = await engineFunctions.getWcData(rule, flow);
  const graphSolver = solver.graphSolver;
  const entry = {
    name, rule, flow,
    words: solver.wordMap.getAllWords(),
    types: graphSolver.typeMap.map(sortedMap),
    depths: graphSolver.depthMap.map(sortedMap),
    partitions: Object.fromEntries(["removed", "winlose", "route"].map(name => [name, sortedEdges(graphSolver.graphs.getGraph(name).edges(1))])),
    wordTypes: graphSolver.getWordTypeNum(),
    routeNodes: [graphSolver.getRouteNodes(0), graphSolver.getRouteNodes(1)],
    maxRoute: [graphSolver.getMaxRouteInfo(0), graphSolver.getMaxRouteInfo(1)],
    nextWords: histories.map(history => ({
      history,
      words: solver.getNextWords(history),
      terminal: isGameEnd(solver, history, false),
      stealableTerminal: isGameEnd(solver, history, true),
    })),
    search: search ? graphSolver.graphs.union().edges(1).map(([head, tail]) => {
      let completed;
      engineFunctions.startStreamingSingleThreadSearch(event => {
        if (event.action === "done") completed = event.payload;
      }, graphSolver.graphs.union(), [head, tail], precedence, flow);
      return {
        move: [head, tail],
        isWin: engineFunctions.searchIsWin(graphSolver.graphs.union(), [head, tail], precedence).isWin,
        optimalPath: completed.optimalPath,
      };
    }) : [],
  };
  if (search || name === "sample/change-0/flow-0" || name === "sample/change-1/flow-0") {
    entry.wordCards = [];
    for (const view of [0, 1]) {
      const nodes = graphSolver.graphs.union().nodes(view).sort(ordinal).slice(0, 5);
      for (const node of nodes) {
        for (const direction of [0, 1]) entry.wordCards.push({ name: node, view, direction, cards: solver.getWordsCardsFromChar(node, view, direction) });
      }
    }
    entry.removedWords = solver.getRemovedWordsFile();
    entry.winWords = [solver.getWinWordsFile(0), solver.getWinWordsFile(1)];
    entry.bangdan = [solver.getBangdanFile(0), solver.getBangdanFile(1)];
    entry.essentialWinWords = [solver.getEssentialWinWordsFile(0), solver.getEssentialWinWordsFile(1)];
    entry.routeWords = [[solver.getRouteWordsFile(0, 0), solver.getRouteWordsFile(0, 1)], [solver.getRouteWordsFile(1, 0), solver.getRouteWordsFile(1, 1)]];
    entry.sccData = [solver.getSccData(0, false), solver.getSccData(1, false)];
  }
  cases.push(entry);
}

for (let change = 0; change < 11; change++) {
  for (const flow of [0, 1]) await addCase(`sample/change-${change}/flow-${flow}`, makeRule(words, change), flow);
}
for (const [name, input] of Object.entries(provenance.specialCases)) {
  const change = name === "standardDueum" ? 1 : name === "consonantReversal" ? 8 : 0;
  const histories = [[], ...input.map(word => [word]), input];
  if (name === "parallelWords") histories.push(["가나", "나니가"], ["가나", "나니가", "가시나"]);
  for (const flow of [0, 1]) await addCase(`${name}/flow-${flow}`, makeRule(input, change), flow, histories, true);
}
for (const flow of [0, 1]) {
  for (const manner of [1, 2, 3]) {
    const rule = makeRule(words, 1);
    rule.content.postprocessing.manner = { type: manner, ...(manner === 3 ? { nextWordsLimit: 2 } : {}) };
    await addCase(`sample/manner-${manner}/flow-${flow}`, rule, flow);
  }
  const alterations = makeRule(provenance.specialCases.chain);
  alterations.content.wordRule.words.option.content += " 사과 사과";
  alterations.content.wordRule.addedWords = "가나 사과";
  alterations.content.wordRule.removedWords = "과자";
  alterations.content.postprocessing.removedWords = "가나";
  alterations.content.postprocessing.addedWords = "과자 과자";
  await addCase(`input-and-postprocessing-add-remove/flow-${flow}`, alterations, flow, [[], ["사과"]], true);
  const filtered = makeRule(words, 1);
  filtered.content.wordRule.regexFilter = "[가-힣]{2,3}";
  await addCase(`regex-filter/flow-${flow}`, filtered, flow);
  const backward = makeRule(words, 1);
  Object.assign(backward.content.wordConnectionRule, { headDir: 1, tailDir: 0 });
  await addCase(`backward-word-chain/flow-${flow}`, backward, flow);
  const middle = makeRule(words, 6);
  middle.content.wordRule.regexFilter = "[가-힣]{3,}";
  Object.assign(middle.content.wordConnectionRule, { rawHeadIdx: 2, rawTailIdx: 2 });
  await addCase(`inner-character-indices/flow-${flow}`, middle, flow);
  await addCase(`empty/flow-${flow}`, makeRule([]), flow, [[]]);
}

const inputs = new Set(words.flatMap(word => [word[0], word.at(-1)]));
for (const onset of [2, 5]) {
  for (let vowel = 0; vowel < 21; vowel++) {
    for (let coda = 0; coda < 28; coda++) inputs.add(String.fromCharCode(0xac00 + onset * 588 + vowel * 28 + coda));
  }
}
for (const input of ["양", "영", "이", "요", "유", "예", "야", "넝", "음", "름", "얘", "ㄹ", "ㄴ", "ㅇ", "ㅏ", "ㄷ", "ㅎ", "밖", "값", "각", "힣"]) inputs.add(input);
const changes = [];
for (let index = 0; index < 11; index++) {
  for (const input of [...inputs].sort(ordinal)) changes.push({ index, input, forward: sampleChangeFuncs[index].forward(input), backward: sampleChangeFuncs[index].backward(input) });
}
// Write only after every case succeeds, preserving a previous oracle on failure.
const result = {
  generator: "game-ai/dist/index.js original TypeScript engine; timings and random selections omitted",
  fixtureSha256: createHash("sha256").update(fixtureBytes).digest("hex"),
  cases, changes,
};
writeFileSync(resolve(fixtureRoot, "golden.json"), `${JSON.stringify(result)}\n`, "utf8");
console.log(`Generated ${cases.length} engine cases and ${changes.length} transformation checks.`);
