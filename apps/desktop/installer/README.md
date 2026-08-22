# Fluxa webOS Installer

Lives in this repo so its version tracks the `.ipk` that `npm run package:webos` builds.

Installs Fluxa onto an LG webOS TV without the webOS CLI, Node, or a terminal.

## What it does

1. Finds the TV on the network over SSDP, or takes an IP address you type in
2. Fetches the TV's developer key from `http://<tv>:9991/webos_rsa`
3. Unlocks that key with the passphrase from the Developer Mode app
4. Connects over SSH to `<tv>:9922`, uploads the `.ipk`, and installs it through
   `luna://com.webos.appInstallService/dev/install`

Everything runs from the app. There is no dependency on `ares-cli`.

## Before you start

LG gates sideloading, and no installer can work around it:

1. Create a free LG account and sign in on the TV
2. Install **Developer Mode** from the LG Content Store
3. Open it, turn Dev Mode on, and let the TV restart
4. Note the **passphrase** it shows

Developer Mode sessions expire roughly every 50 hours. When Fluxa stops
launching, open the Developer Mode app and extend the session — you do not need
to reinstall.

## Building

```bash
npm install
npm run tauri dev      # run it
npx tauri build        # produce an installer for the current platform
```

Tagged pushes build Windows, macOS, and Linux installers in CI and attach them
to the release.

## Limitations

The TV must be reachable on the same network, and Developer Mode must be running
with its key server enabled — that is what serves the key on port 9991.
