# Tunarr Extensions

Browse extensions for Tunarr here, then install them from **Settings → Extensions** in your trial instance. Each extension has its own repository and can be updated independently of Tunarr.

## Get started

1. In Tunarr, open **Settings → Extensions**.
2. Choose an extension from the default source and select **Install**.
3. Follow the extension’s setup instructions.

## Available extensions

| Extension | What it does |
| --- | --- |
| [Bazarr](https://github.com/TacoRocket/tunarr-bazarr-extension) | Shows installed and missing subtitle languages for movies in your Bazarr library. Requires a running Bazarr instance connected to Radarr. |

Tunarr reads the store listings from [source.json](source.json). Each listing names the publisher’s repository, the version and the exact Git commit to install.

## If your Tunarr has no Extensions page

Deploy the extension-enabled trial once using the [Docker Compose setup instructions](https://github.com/TacoRocket/tunarr/tree/trial/extension-store/docker/test/extensions). Then open its **Settings → Extensions** page to install extensions.
