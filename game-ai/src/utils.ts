export function getIdx(rawIdx: number, dir: 0 | 1) {
  return dir === 0 ? rawIdx - 1 : -rawIdx;
}

export function getHeadTail(
  word: string,
  headIdx: number,
  tailIdx: number,
): [string, string] {
  const head = word.at(headIdx);
  if (!head) {
    throw `${word} has not ${headIdx}'th index`;
  }
  const tail = word.at(tailIdx);
  if (!tail) {
    throw `${word} has not ${tailIdx}'th index`;
  }
  return [head, tail];
}

export function toObject<T extends string>(keys: T[], values: (0 | 1)[]) {
  const obj = keys.reduce(
    (acc, key, i) => {
      acc[key] = values[i];
      return acc;
    },
    {} as Record<T, 0 | 1>,
  );
  return obj;
}

export function removeDup<T>(arr: T[]): T[] {
  return [...new Set(arr)];
}

export function getRegex(rawStr: string): RegExp | null {
  try {
    return new RegExp(rawStr);
  } catch {
    return null;
  }
}

export function toNestedRecord(
  pairs: [string, string][],
): Record<string, Record<string, number>> {
  const result: Record<string, Record<string, number>> = {};

  for (const [key1, key2] of pairs) {
    if (!result[key1]) result[key1] = {};
    result[key1][key2] = 0;
  }

  return result;
}

export function truncate<T>(elements: T[], toString: (e: T) => string) {
  const maxDisplay = 10;
  if (elements.length <= maxDisplay) {
    return elements.map((e) => toString(e)).join(", ");
  } else {
    return (
      elements
        .slice(0, maxDisplay)
        .map((e) => toString(e))
        .join(", ") + ",..."
    );
  }
}

export function compareEdge(a: [string, string], b: [string, string]) {
  return a[0].localeCompare(b[0]) || a[1].localeCompare(b[1]);
}
