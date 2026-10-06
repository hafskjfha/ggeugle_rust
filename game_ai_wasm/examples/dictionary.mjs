/** Prepare local dictionary text without filtering entries used by the engine. */
export function parseDictionary(text) {
  const content = text.replace(/^\uFEFF/u, '').trim();
  return { content, wordCount: content ? content.split(/\s+/u).length : 0 };
}

/** Read a browser File or Blob with a name, rejecting invalid encoded text. */
export async function readDictionaryFile(file) {
  const bytes = new Uint8Array(await file.arrayBuffer());
  let encoding = 'utf-8';
  if (bytes[0] === 0xff && bytes[1] === 0xfe) encoding = 'utf-16le';
  if (bytes[0] === 0xfe && bytes[1] === 0xff) encoding = 'utf-16be';
  const text = new TextDecoder(encoding, { fatal: true }).decode(bytes);
  return { name: file.name, ...parseDictionary(text) };
}
