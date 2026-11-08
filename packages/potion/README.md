# Potion

The Potion extension gives Tunarr semantic recommendations. It turns each show's and movie's description into a short list of numbers with [Potion](https://huggingface.co/minishlab/potion-base-8M), a small static embedding model, and finds titles whose descriptions are alike. Tunarr uses it when you generate a channel with **Semantic** or **Balanced** matching.

The model runs inside Tunarr's extension sandbox. Nothing is sent anywhere.

## Prerequisites

- Tunarr with **Settings → Extensions** (extension API 0.5, an early beta).
- About 128 MB of memory while it runs, and up to 128 MB of disk for its index (about 1 MB per 1,000 titles).

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

`src/adapter.src.js` is the source; `adapter.js` is it bundled with the Hugging Face tokenizer (esbuild, IIFE, ES2020). The model files are pinned to `minishlab/potion-base-8M` revision `bf8b056651a2c21b8d2565580b8569da283cab23`. Model license: [MODEL-LICENSE](MODEL-LICENSE) (MIT). Tokenizer license: [TOKENIZERS-LICENSE](TOKENIZERS-LICENSE) (Apache-2.0).
