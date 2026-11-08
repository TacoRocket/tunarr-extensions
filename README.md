# Tunarr extension source

The Tunarr Extensions trial reads [source.json](source.json) to show the extensions you can install from its default source. Each entry names the publisher’s repository, the version and the exact Git commit used for installation.

## Available extensions

[Bazarr](https://github.com/TacoRocket/tunarr-bazarr-extension) shows which subtitle languages are installed or missing for a movie. Run Bazarr and connect it to Radarr before installing the adapter.

To try it, open **Settings → Extensions** in the trial, choose **Bazarr**, then **Install**. Enter your Bazarr connection details when prompted. See the [trial host setup](https://github.com/TacoRocket/tunarr/tree/trial/bazarr-extension-install/docker/test/extensions) for the Docker Compose instructions.
