# Potion

The Potion extension gives Tunarr semantic recommendations. It turns each show's and movie's description into a short list of numbers with [Potion](https://huggingface.co/minishlab/potion-base-8M), a small static embedding model, and finds titles whose descriptions are alike. Tunarr uses it when you generate a channel with **Semantic** or **Balanced** matching.

The model runs inside Tunarr's extension sandbox. Nothing is sent anywhere.

## Prerequisites

- Tunarr with **Settings → Extensions** (extension API 0.5, an early beta).
- A 192 MiB extension memory allowance and up to 128 MiB of disk for its index (about 1 MiB per 1,000 titles).

## Configuration

None. Install it from **Settings → Extensions → Add extension**. It is on as soon as it is installed.

## Actions

### Check index

Shows how many titles Potion has indexed.

## Data updates

Tunarr hands Potion new and changed shows and movies about once an hour, a few hundred at a time. The first pass over a large library takes a few minutes.

## Known limitations

- English descriptions work best; Potion is an English model.
- It compares descriptions only, not artwork, ratings or what you watch.

## Troubleshooting

- **Semantic matching says it needs Potion:** wait for the first indexing pass, then use **Check index** to confirm titles are indexed.

## Removing the extension

In Settings → Extensions, choose the Potion card, then **Uninstall**. This removes the model and its index. Nothing changes in your library.

## For developers

The adapter stores normalized 256-dimensional vectors in chunks of 1,024 rows. Version 1.1 keeps the 1.0 index format. A query scores each chunk against the requested title and keeps the nearest matches in JavaScript.

On hosts with compute support, `compute/src/lib.rs` tokenizes ordinary ASCII text with WordPiece, averages the model rows, and scores vectors by dot product. The pinned Hugging Face tokenizer handles Unicode, control characters and special tokens. Queries read the index directly. On older API 0.5 hosts, the adapter uses its JavaScript implementation.

The published module was built with Rust 1.99.1 and the `wasm32-unknown-unknown` target. From this directory, rebuild both artifacts with:

```sh
rustup target add wasm32-unknown-unknown
./compute/build.sh
npm install --prefix .build --ignore-scripts --no-fund --no-audit --no-package-lock --no-save esbuild@0.21.5 @huggingface/tokenizers@0.2.0
NODE_PATH=.build/node_modules .build/node_modules/.bin/esbuild src/adapter.src.js --bundle --format=iife --target=es2020 --outfile=adapter.js
```

The model files are pinned to `minishlab/potion-base-8M` revision `bf8b056651a2c21b8d2565580b8569da283cab23`. See [model license](MODEL-LICENSE), [tokenizer license](TOKENIZERS-LICENSE) and [compiled module notices](compute/NOTICES).
