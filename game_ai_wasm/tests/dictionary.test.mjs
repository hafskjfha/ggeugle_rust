import assert from 'node:assert/strict';
import { File } from 'node:buffer';
import test from 'node:test';

const loaded = await import('../examples/dictionary.mjs').catch((error) => ({ error }));

test('dictionary helpers are available to the browser example', () => {
  assert.equal(loaded.error, undefined);
});

if (!loaded.error) {
  const { parseDictionary, readDictionaryFile } = loaded;

  test('manual dictionary separates words across spaces and newlines', () => {
    assert.deepEqual(parseDictionary('  사과\t과자\r\n자두\n '), {
      content: '사과\t과자\r\n자두', wordCount: 3,
    });
  });

  test('manual dictionary strips a leading BOM and keeps duplicate entries', () => {
    assert.deepEqual(parseDictionary('\uFEFF사과 사과 과자'), {
      content: '사과 사과 과자', wordCount: 3,
    });
  });

  test('blank dictionaries contain zero words', () => {
    assert.deepEqual(parseDictionary('\uFEFF \n\t\r\n'), { content: '', wordCount: 0 });
  });

  test('uploaded UTF-8 text keeps its filename and strips its BOM', async () => {
    const file = new File([new Uint8Array([0xef, 0xbb, 0xbf]), '사과\n과자\n자두\n'], 'words.txt');
    assert.deepEqual(await readDictionaryFile(file), {
      name: 'words.txt', content: '사과\n과자\n자두', wordCount: 3,
    });
  });

  test('uploaded blank text does not become a phantom word', async () => {
    assert.deepEqual(await readDictionaryFile(new File(['\uFEFF \n\t'], 'empty.txt')), {
      name: 'empty.txt', content: '', wordCount: 0,
    });
  });

  test('UTF-16 text with a byte order mark is decoded correctly', async () => {
    const littleEndian = new Uint8Array([0xff, 0xfe, 0xac, 0xc0, 0xfc, 0xac, 0x0a, 0x00, 0xfc, 0xac, 0x90, 0xc7]);
    const bigEndian = new Uint8Array([0xfe, 0xff, 0xc0, 0xac, 0xac, 0xfc, 0x00, 0x0a, 0xac, 0xfc, 0xc7, 0x90]);
    for (const bytes of [littleEndian, bigEndian]) {
      assert.deepEqual(await readDictionaryFile(new File([bytes], 'unicode.txt')), {
        name: 'unicode.txt', content: '사과\n과자', wordCount: 2,
      });
    }
  });

  test('invalid UTF-8 bytes are rejected instead of silently changing the words', async () => {
    const file = new File([new Uint8Array([0xc3, 0x28])], 'broken.txt');
    await assert.rejects(readDictionaryFile(file), { name: 'TypeError' });
  });

  test('a failed file read is reported to the caller', async () => {
    const unreadable = {
      name: 'unreadable.txt',
      async arrayBuffer() { throw new Error('The file could not be read'); },
    };
    await assert.rejects(readDictionaryFile(unreadable), /The file could not be read/);
  });
}
