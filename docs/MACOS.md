# 1M Context Ticker for macOS

The macOS download is an unsigned, unnotarized community build. Its automated checks run on GitHub-hosted macOS; no physical-Mac acceptance is claimed.

## Verify and install

Download these three files from the same release:

- `1M-Context-Ticker-macOS-universal.dmg`
- `1M-Context-Ticker-macOS-universal.dmg.sha256`
- `1M-Context-Ticker-macOS-universal.manifest.json`

Verify the DMG before opening it:

```bash
shasum -a 256 -c 1M-Context-Ticker-macOS-universal.dmg.sha256
```

Open the DMG and drag **1M Context Ticker** to **Applications**. On first launch, macOS is expected to warn that the developer cannot be verified. After checking the SHA-256:

1. Control-click **1M Context Ticker** in Applications and choose **Open**.
2. If macOS still blocks it, open **System Settings > Privacy & Security**, find the blocked-app message, and choose **Open Anyway** for this app only.

Do not run commands that remove quarantine metadata, and never disable Gatekeeper system-wide.

The first launch requires `model = "gpt-5.6-sol"` to already be present in `~/.codex/config.toml`. It makes a byte-exact backup, refuses pre-existing owned keys, adds only the four documented 1M policy keys, copies the exact Sol catalog to the ticker's Application Support folder, and registers the ticker through the macOS Login Items service. It does not stop or restart Codex.

## Status, stop, and upgrade

```bash
TICKER="/Applications/1M Context Ticker.app/Contents/MacOS/OneMContextTicker"
"$TICKER" --action status
"$TICKER" --action stop
"$TICKER" --action upgrade
```

`status` reports config, catalog, backup, and Login Item state. A Login Item may report `requiresApproval`; use **System Settings > General > Login Items** to approve it. `stop` targets only other running processes with the ticker's bundle identifier.

## Uninstall

Run the owned-state uninstall before removing the app:

```bash
TICKER="/Applications/1M Context Ticker.app/Contents/MacOS/OneMContextTicker"
"$TICKER" --action uninstall
```

Then move **1M Context Ticker** from Applications to the Trash. Uninstall unregisters the ticker Login Item, restores the original config bytes when the config is otherwise unchanged, preserves later user edits, removes only values still owned by the ticker, and deletes only named app-owned state files.

## Feedback

Use the repository's GitHub Issues page. Include the macOS version, CPU architecture, ticker version, whether the Codex window was foreground, and a description of placement or lifecycle behavior. Do not attach rollout files, configuration contents, prompts, transcripts, tokens, or credentials.
