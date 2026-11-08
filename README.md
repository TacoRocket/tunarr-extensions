# Tunarr Extensions

Browse extensions for Tunarr here, then install them from **Settings → Extensions** in an extension-enabled Tunarr instance. Each extension has its own repository and can be updated independently of Tunarr.

## Get started

1. In Tunarr, open **Settings → Extensions**.
2. Choose an extension from the default source and select **Install**.
3. Follow the extension’s setup instructions.

## Available extensions

| Extension | What it does |
| --- | --- |
| [Bazarr](https://github.com/TacoRocket/tunarr-bazarr-extension) | Shows installed and missing subtitle languages for movies in your Bazarr library. Requires a running Bazarr instance connected to Radarr. |

Each listing names the publisher's repository, the version and the exact Git commit to install.

| Host | Source | Bazarr package |
| --- | --- | --- |
| Original extension trial (API v1) | [source.json](source.json) | Python v0.5.0 |
| Proposed fork integration (API v2) | [source-v2.json](source-v2.json) | JavaScript v1.0.1 |

The v2 source is prepared for the upcoming fork PR. The original trial can continue installing and updating from its v1 source.

## If your Tunarr has no Extensions page

Deploy the extension-enabled trial once using the [Docker Compose setup instructions](https://github.com/TacoRocket/tunarr/tree/trial/extension-store/docker/test/extensions). Then open its **Settings → Extensions** page to install extensions.
