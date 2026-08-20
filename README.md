## OpenAction Linkwarden plugin

An OpenAction ([OpenDeck](https://github.com/nekename/OpenDeck) / [Tacto](https://tacto.rivul.us)) plugin for
[Linkwarden](https://linkwarden.app), a self-hosted bookmark manager.

#### Actions

- **Add Link** — Sends the URL currently in your clipboard to Linkwarden. Optionally uses the linked
  page's `<title>` as the link name instead of the raw URL, and can be pre-configured with a
  description, tags, and a collection to file the link into.
- **Open Linkwarden** — Opens your Linkwarden instance in the default browser.

Both actions' property inspectors show a live connection indicator that confirms the configured
instance URL and API token can actually reach Linkwarden (by pulling your tags and collections).

#### Configuration

Set your Linkwarden instance URL and API token once in either action's property inspector — they're
shared globally. Each button can also override the URL/token individually if you want a button tied
to a different Linkwarden instance or account.

#### Requirements

- Rust toolchain (stable) with the target triples you want to build for, e.g.:
  ```
  rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu
  ```
- [Deno](https://deno.com) for building the property inspector (`pi/`).
- `jq` (used by `build.sh`/`package.sh` to read `assets/manifest.json`).

#### Building

Build the property inspector once (or whenever `pi/` changes):

```
cd pi && deno task build
```

Build and install the plugin into a local OpenDeck instance with `build.sh`:

```
./build.sh                 # native OpenDeck config dir, all targets in manifest.json, reload if OpenDeck is running
./build.sh --flatpak       # OpenDeck installed via Flatpak
./build.sh --output <path> # custom install directory
```

Run `./build.sh --help` for the full flag list (custom binary name, specific targets, reload
behavior).

Package a distributable `.streamDeckPlugin` file (works in both OpenDeck and Elgato's Stream Deck
software):

```
./package.sh
```

#### License

MIT — see [LICENSE](LICENSE).

---

**Disclaimer:** This project's code, scripts, and documentation were developed with substantial
assistance from [Claude](https://claude.com) (Anthropic's AI assistant), used as a coding tool
throughout development. Review the source before relying on it in contexts where that matters to
you.