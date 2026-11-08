# Tunarr Extensions

Browse extensions for Tunarr here, then install them from **Settings → Extensions** in an extension-enabled Tunarr instance. Each extension has its own repository and can be updated independently of Tunarr.

## Get started

1. In Tunarr, open **Settings → Extensions**.
2. Choose an extension from the default source and select **Install**.
3. Follow the extension’s setup instructions.

## Available extensions

| Extension | What it does |
| --- | --- |
| [Potion](packages/potion) | Semantic recommendations: finds shows and movies with similar descriptions, using the small Potion model inside Tunarr. Nothing extra to run. |
| [Bazarr](https://github.com/TacoRocket/tunarr-bazarr-extension) | Checks which subtitles Bazarr has for a movie and asks Bazarr to search for the missing ones. Requires a running Bazarr connected to Radarr. |

Each listing names the publisher's repository, the version and the exact Git commit to install.

| Host | Source | Bazarr package |
| --- | --- | --- |
| Tunarr extension API 0.x (early beta) | [source-v0.json](source-v0.json) | JavaScript v1.1.2 |
| Original extension trial (API v1) | [source.json](source.json) | Python v0.5.0 |
| Proposed fork integration (API v2) | [source-v2.json](source-v2.json) | JavaScript v1.0.1 |

Tunarr reads the listing for its extension API's major version, so a Tunarr update within 0.x keeps using `source-v0.json`. The extension store is under review in [Tunarr PR #219](https://github.com/TacoRocket/tunarr/pull/219). The earlier trial hosts keep installing from their own listings.

## If your Tunarr has no Extensions page

Deploy the extension-enabled trial once using the [Docker Compose setup instructions](https://github.com/TacoRocket/tunarr/tree/trial/extension-store/docker/test/extensions). Then open its **Settings → Extensions** page to install extensions.
