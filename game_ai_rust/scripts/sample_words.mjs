import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const crateRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourcePath = resolve(process.argv[2] ?? resolve(crateRoot, "../data/all_words.txt"));
const outputPath = resolve(process.argv[3] ?? resolve(crateRoot, "tests/fixtures"));
const source = readFileSync(sourcePath);
const records = source.toString("utf8").replace(/^\uFEFF/, "").split(/\r?\n/)
  .map((word, index) => ({ word: word.trim(), line: index + 1 })).filter(({ word }) => word.length > 0);
const unique = new Map(records.map(record => [record.word, record]));
const selected = new Map();
const add = record => selected.set(record.word, record);
const specialCases = {
  chain: ["사과", "과자", "자두"],
  cycle: ["가나", "나무바다", "다가"],
  parallelWords: ["가나", "가시나", "나니가"],
  standardDueum: ["역량", "양력"],
  consonantReversal: ["가론강", "악어"],
  selfLoop: ["가가"],
  compatibilityJamo: ["ㄷ형강", "삿ㅎ"],
};
for (const words of Object.values(specialCases)) {
  for (const word of words) {
    const record = unique.get(word);
    if (!record) throw new Error(`Required real-word fixture is absent: ${word}`);
    add(record);
  }
}

// Broad lexical coverage, plus every available onset/vowel/coda index at each end.
const quantiles = 256;
for (let i = 0; i < quantiles; i++) add(records[Math.floor(i * (records.length - 1) / (quantiles - 1))]);
const categories = new Set();
for (const record of records) {
  const characters = [...record.word];
  for (const [end, character] of [["head", characters[0]], ["tail", characters.at(-1)]]) {
    const code = character.codePointAt(0) - 0xac00;
    if (code < 0 || code >= 11172) continue;
    for (const [part, value] of [["onset", Math.floor(code / 588)], ["vowel", Math.floor(code / 28) % 21], ["coda", code % 28]]) {
      const key = `${end}:${part}:${value}`;
      if (!categories.has(key)) {
        categories.add(key);
        add(record);
      }
    }
  }
}
const samples = [...selected.values()].sort((a, b) => a.line - b.line);
mkdirSync(outputPath, { recursive: true });
writeFileSync(resolve(outputPath, "words.txt"), `${samples.map(({ word }) => word).join("\n")}\n`, "utf8");
writeFileSync(resolve(outputPath, "provenance.json"), `${JSON.stringify({
  source: "data/all_words.txt",
  sourceSha256: createHash("sha256").update(source).digest("hex"),
  sourceBytes: source.length,
  sourceWordCount: records.length,
  sampling: "256 evenly spaced source entries; first occurrence of each available Hangul onset/vowel/coda index at both endpoints; explicit real-word chain, cycle, parallel-edge, dueum, reversal, self-loop and compatibility-jamo examples. Deduplicate and retain source order.",
  endpointCategoryCount: categories.size,
  sampleWordCount: samples.length,
  specialCases,
  samples,
}, null, 2)}\n`, "utf8");
console.log(`Selected ${samples.length} genuine words from ${records.length}; SHA-256 ${createHash("sha256").update(source).digest("hex")}.`);
