// Potion similarity extension for Tunarr, extension API 0.5.
//
// Turns each title's text into a 256-number vector with Potion (a static
// Model2Vec model: tokenize, average the token vectors, normalize) and keeps
// those vectors in this extension's own storage. Tunarr asks it to index new
// and changed titles, and for the titles nearest to a given one.
import { Tokenizer } from '@huggingface/tokenizers';

const DIM = 256;
const VOCAB = 29528;
const UNKNOWN = 1;
// Vectors are stored in files of 1,024 rows (1 MiB), so an index pass only
// rewrites the files it touches.
const CHUNK = 1024;

// API 0.5 hosts that predate compute keep using their JavaScript implementation.
const engine = typeof compute === 'function' ? compute('potion.wasm') : null;
let tokenizer;
function loadTokenizer() {
  return tokenizer ??= new Tokenizer(
    JSON.parse(readTextFile('tokenizer.json')),
    JSON.parse(readTextFile('tokenizer_config.json')),
  );
}

let model;
function loadModel() {
  if (model) return model;
  const weights = readFile('model.safetensors');
  const view = new DataView(weights);
  const start = 8 + Number(view.getBigUint64(0, true));
  model = {
    embeddings: new Float32Array(weights.slice(start, start + VOCAB * DIM * 4)),
    tokenizer: loadTokenizer(),
  };
  return model;
}

function encode(text, out, offset) {
  const { tokenizer, embeddings } = loadModel();
  const ids = tokenizer
    .encode(text, { add_special_tokens: false })
    .ids.filter((id) => id !== UNKNOWN);
  for (let d = 0; d < DIM; d++) out[offset + d] = 0;
  if (!ids.length) return;
  for (const id of ids) {
    const row = id * DIM;
    for (let d = 0; d < DIM; d++) out[offset + d] += embeddings[row + d];
  }
  let norm = 0;
  for (let d = 0; d < DIM; d++) {
    out[offset + d] /= ids.length;
    norm += out[offset + d] * out[offset + d];
  }
  norm = Math.sqrt(norm) + 1e-32;
  for (let d = 0; d < DIM; d++) out[offset + d] /= norm;
}

function readIndex() {
  return {
    slots: JSON.parse(storage.get('slots') ?? '{}'),
    free: JSON.parse(storage.get('free') ?? '[]'),
    count: Number(storage.get('count') ?? 0),
  };
}

function readChunk(c) {
  const bytes = storage.readFile(`vectors-${c}.bin`);
  return bytes ? new Float32Array(bytes) : new Float32Array(CHUNK * DIM);
}

function chunkReader() {
  const loaded = new Map();
  const get = (c) => {
    if (!loaded.has(c)) loaded.set(c, readChunk(c));
    return loaded.get(c);
  };
  return { get, loaded };
}

function computeInput({ text }) {
  // The pinned tokenizer owns Unicode normalization and special-token handling.
  if (/[^ -~]|\[/.test(text)) return loadTokenizer().encode(text, { add_special_tokens: false }).ids;
  return text;
}

function index(items, removed) {
  const { slots, free, count: start } = readIndex();
  let count = start;
  const chunks = chunkReader();
  const dirty = new Set();
  for (const id of removed) {
    const slot = slots[id];
    if (slot === undefined) continue;
    const c = Math.floor(slot / CHUNK);
    chunks.get(c).fill(0, (slot % CHUNK) * DIM, (slot % CHUNK + 1) * DIM);
    dirty.add(c);
    delete slots[id];
    free.push(slot);
  }
  let vectors;
  if (engine && items.length) {
    engine.call('init', { file: 'tokenizer.json' }, { file: 'model.safetensors' });
    vectors = new Float32Array(engine.call('embed', JSON.stringify(items.map(computeInput))));
  }
  for (const [i, { id, text }] of items.entries()) {
    let slot = slots[id];
    if (slot === undefined) {
      slot = free.length ? free.pop() : count++;
      slots[id] = slot;
    }
    const c = Math.floor(slot / CHUNK);
    const offset = (slot % CHUNK) * DIM;
    if (vectors) chunks.get(c).set(vectors.subarray(i * DIM, (i + 1) * DIM), offset);
    else encode(text, chunks.get(c), offset);
    dirty.add(c);
  }
  for (const c of dirty) storage.writeFile(`vectors-${c}.bin`, chunks.get(c));
  storage.set('slots', JSON.stringify(slots));
  storage.set('free', JSON.stringify(free));
  storage.set('count', String(count));
  return { indexed: items.length };
}

function query(id, k) {
  const { slots, count } = readIndex();
  const target = slots[id];
  if (target === undefined) return { matches: [] };
  const owner = new Array(count);
  for (const [key, slot] of Object.entries(slots)) owner[slot] = key;
  const q = readChunk(Math.floor(target / CHUNK)).slice((target % CHUNK) * DIM, (target % CHUNK + 1) * DIM);
  const best = [];
  for (let c = 0; c * CHUNK < count; c++) {
    const rows = readChunk(c);
    const scores = engine ? new Float64Array(engine.call('score', rows, q)) : null;
    const length = Math.min(CHUNK, count - c * CHUNK);
    for (let i = 0; i < length; i++) {
      const slot = c * CHUNK + i;
      if (slot === target || owner[slot] === undefined) continue;
      let score = scores ? scores[i] : 0;
      if (!scores) {
        const base = i * DIM;
        for (let d = 0; d < DIM; d++) score += rows[base + d] * q[d];
      }
      if (best.length < k) {
        best.push({ slot, score });
        if (best.length === k) best.sort((a, b) => a.score - b.score);
      } else if (score > best[0].score) {
        best[0] = { slot, score };
        best.sort((a, b) => a.score - b.score);
      }
    }
  }
  best.sort((a, b) => b.score - a.score);
  return { matches: best.map(({ slot, score }) => ({ id: owner[slot], score })) };
}

globalThis.run = function run(input) {
  switch (input.action) {
    case 'similarity.index':
      return index(input.items ?? [], input.removed ?? []);
    case 'similarity.query':
      return query(String(input.id), Math.max(1, Math.min(200, Number(input.k) || 50)));
    case 'status': {
      const titles = Object.keys(readIndex().slots).length;
      return titles
        ? {
            message: `Potion has indexed ${titles} ${titles === 1 ? 'title' : 'titles'} and is ready for semantic recommendations.`,
            tone: 'success',
            details: { titles },
          }
        : {
            message: 'Potion hasn’t indexed anything yet. Tunarr sends your shows and movies within the hour.',
            tone: 'info',
          };
    }
    default:
      throw new Error('This version of the Potion extension doesn’t know that action.');
  }
};
