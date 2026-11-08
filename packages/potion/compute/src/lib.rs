//! Potion's two heavy steps, compiled to WebAssembly for the Tunarr extension.
//!
//! The extension's JavaScript keeps every decision (what to index, where to
//! store it, how many neighbours to ask for). This module only answers:
//!
//! - `init(tokenizer.json, model.safetensors)` loads the model once per run.
//! - `embed(["text", ...])` turns each text into a 256-number vector, the same
//!   way the JavaScript version does: BERT WordPiece tokens (unknown tokens
//!   dropped), the average of their rows, scaled to length 1.
//! - `score(vectors, target)` returns dot products for one chunk of stored vectors.
//!
//! Calling convention (Tunarr's compute API): each input is a byte buffer the
//! host copies in with `alloc`, passed as a (pointer, length) pair. Each
//! function returns `pointer << 32 | length` of its output, which starts with
//! one status byte: 0 for success, 1 for an error whose message follows.

use std::collections::HashMap;
use std::sync::OnceLock;

const DIM: usize = 256;

struct Model {
    vocab: HashMap<String, u32>,
    unknown: u32,
    max_chars: usize,
    prefix: String,
    rows: usize,
    embeddings: Vec<f32>,
}

static MODEL: OnceLock<Model> = OnceLock::new();

// ---- memory -------------------------------------------------------------

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len].into_boxed_slice()) as *mut u8
}

/// The host returns each output pointer with the exact length packed by answer.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(take(ptr, len));
}

// Inputs are allocated by alloc in this same module. The host transfers
// exactly one allocation for each pointer/length pair in the call.
unsafe fn take(ptr: *mut u8, len: usize) -> Box<[u8]> {
    Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len))
}

fn answer(result: Result<Vec<u8>, String>) -> u64 {
    let bytes = match result {
        Ok(mut payload) => {
            payload.insert(0, 0);
            payload
        }
        Err(message) => {
            let mut bytes = vec![1];
            bytes.extend_from_slice(message.as_bytes());
            bytes
        }
    };
    let len = bytes.len();
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8 as u32;
    ((ptr as u64) << 32) | len as u64
}

// ---- init ---------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn init(tok: *mut u8, tok_len: usize, st: *mut u8, st_len: usize) -> u64 {
    let tokenizer = take(tok, tok_len);
    let weights = take(st, st_len);
    answer(load(&tokenizer, &weights).map(|model| {
        let _ = MODEL.set(model);
        Vec::new()
    }))
}

fn load(tokenizer: &[u8], weights: &[u8]) -> Result<Model, String> {
    let json: serde_json::Value =
        serde_json::from_slice(tokenizer).map_err(|e| format!("tokenizer.json: {e}"))?;
    let model = &json["model"];
    if model["type"] != "WordPiece" {
        return Err("tokenizer.json is not a WordPiece tokenizer.".into());
    }
    let vocab: HashMap<String, u32> = model["vocab"]
        .as_object()
        .ok_or("tokenizer.json has no vocabulary.")?
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|id| (k.clone(), id as u32)))
        .collect();
    let unknown_token = model["unk_token"].as_str().unwrap_or("[UNK]");
    let unknown = *vocab
        .get(unknown_token)
        .ok_or("The unknown token is not in the vocabulary.")?;
    let max_chars = model["max_input_chars_per_word"].as_u64().unwrap_or(100) as usize;
    let prefix = model["continuing_subword_prefix"]
        .as_str()
        .unwrap_or("##")
        .to_string();

    // safetensors: 8-byte header length, JSON header, then the raw tensor.
    if weights.len() < 8 {
        return Err("model.safetensors is too short.".into());
    }
    let header_len = u64::from_le_bytes(weights[..8].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(
        weights
            .get(8..8 + header_len)
            .ok_or("model.safetensors header is cut off.")?,
    )
    .map_err(|e| format!("model.safetensors header: {e}"))?;
    let tensor = header
        .as_object()
        .and_then(|h| h.iter().find(|(name, _)| *name != "__metadata__"))
        .map(|(_, t)| t)
        .ok_or("model.safetensors has no tensor.")?;
    if tensor["dtype"] != "F32" || tensor["shape"][1].as_u64() != Some(DIM as u64) {
        return Err("model.safetensors is not a float32 table of 256 columns.".into());
    }
    let rows = tensor["shape"][0]
        .as_u64()
        .ok_or("model.safetensors has no row count.")? as usize;
    let start = 8 + header_len + tensor["data_offsets"][0].as_u64().unwrap_or(0) as usize;
    let data = weights
        .get(start..start + rows * DIM * 4)
        .ok_or("model.safetensors data is cut off.")?;
    let embeddings = data
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    Ok(Model {
        vocab,
        unknown,
        max_chars,
        prefix,
        rows,
        embeddings,
    })
}

// The adapter only passes printable ASCII here. Everything needing Unicode
// normalization, special tokens or control handling arrives as IDs from the
// same tokenizer the JavaScript-only adapter uses.
fn tokenize_ascii(model: &Model, text: &str, ids: &mut Vec<u32>) {
    let lower = text.to_ascii_lowercase();
    let mut start = 0;
    for (i, byte) in lower.bytes().enumerate() {
        if byte == b' ' || byte.is_ascii_punctuation() {
            if start < i {
                word_pieces(model, &lower[start..i], ids);
            }
            if byte.is_ascii_punctuation() {
                word_pieces(model, &lower[i..i + 1], ids);
            }
            start = i + 1;
        }
    }
    if start < lower.len() {
        word_pieces(model, &lower[start..], ids);
    }
}

fn word_pieces(model: &Model, word: &str, ids: &mut Vec<u32>) {
    if word.chars().count() > model.max_chars {
        ids.push(model.unknown);
        return;
    }
    let first = ids.len();
    let mut start = 0;
    let mut piece = String::new();
    while start < word.len() {
        let mut end = word.len();
        let mut found = None;
        while start < end {
            piece.clear();
            if start > 0 {
                piece.push_str(&model.prefix);
            }
            piece.push_str(&word[start..end]);
            if let Some(&id) = model.vocab.get(piece.as_str()) {
                found = Some(id);
                break;
            }
            end -= word[start..end].chars().last().map_or(1, char::len_utf8);
        }
        match found {
            Some(id) => {
                ids.push(id);
                start = end;
            }
            None => {
                ids.truncate(first);
                ids.push(model.unknown);
                return;
            }
        }
    }
}

// ---- embed --------------------------------------------------------------

fn pool(model: &Model, ids: &mut Vec<u32>, out: &mut [f32]) {
    ids.retain(|&id| id != model.unknown && (id as usize) < model.rows);
    out.fill(0.0);
    if ids.is_empty() {
        return;
    }
    // Same arithmetic as the JavaScript version: float32 sums, then the
    // average and length in double precision, stored back as float32.
    for &id in ids.iter() {
        let row = &model.embeddings[id as usize * DIM..(id as usize + 1) * DIM];
        for (o, r) in out.iter_mut().zip(row) {
            *o += *r;
        }
    }
    let count = ids.len() as f64;
    let mut norm = 0f64;
    for o in out.iter_mut() {
        *o = (*o as f64 / count) as f32;
        norm += (*o as f64) * (*o as f64);
    }
    let norm = norm.sqrt() + 1e-32;
    for o in out.iter_mut() {
        *o = (*o as f64 / norm) as f32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn embed(texts: *mut u8, len: usize) -> u64 {
    let input = take(texts, len);
    answer((|| {
        let model = MODEL.get().ok_or("Call init before embed.")?;
        let texts: Vec<serde_json::Value> =
            serde_json::from_slice(&input).map_err(|e| format!("embed input: {e}"))?;
        let mut vectors = vec![0f32; texts.len() * DIM];
        let mut ids = Vec::new();
        for (text, out) in texts.iter().zip(vectors.chunks_exact_mut(DIM)) {
            ids.clear();
            match text {
                serde_json::Value::String(s) if s.is_ascii() => tokenize_ascii(model, s, &mut ids),
                serde_json::Value::Array(tokens) => {
                    for token in tokens {
                        ids.push(
                            token
                                .as_u64()
                                .and_then(|n| n.try_into().ok())
                                .ok_or("Invalid token ID.")?,
                        );
                    }
                }
                _ => return Err("Expected ASCII text or tokenizer IDs.".to_string()),
            }
            pool(model, &mut ids, out);
        }
        Ok(vectors.iter().flat_map(|v| v.to_le_bytes()).collect())
    })())
}

// The host passes allocations from this module's alloc, transferring both
// buffers to score. Float32 values are stored little-endian in the index.
#[no_mangle]
pub unsafe extern "C" fn score(v: *mut u8, v_len: usize, q: *mut u8, q_len: usize) -> u64 {
    let vectors = take(v, v_len);
    let target = take(q, q_len);
    answer((|| {
        if target.len() != DIM * 4 || vectors.len() % (DIM * 4) != 0 {
            return Err("score input is malformed.".to_string());
        }
        let floats = |bytes: &[u8]| f32::from_le_bytes(bytes.try_into().unwrap()) as f64;
        let query: Vec<_> = target.chunks_exact(4).map(floats).collect();
        let mut scores = Vec::with_capacity(vectors.len() / (DIM * 4) * 8);
        for row in vectors.chunks_exact(DIM * 4) {
            let dot: f64 = row
                .chunks_exact(4)
                .map(floats)
                .zip(&query)
                .fold(0.0, |acc, (v, q)| acc + v * q);
            scores.extend_from_slice(&dot.to_le_bytes());
        }
        Ok(scores)
    })())
}
