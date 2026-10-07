# cosmic-ext-applet-niri-workspaces

A native [COSMIC](https://github.com/pop-os/cosmic-epoch) panel applet for people running the
[niri](https://github.com/YaLTeR/niri) compositor with the COSMIC panel.

COSMIC's built-in workspaces applet talks to `cosmic-comp`, so it shows nothing under niri.
This applet talks to niri instead.

- One button per workspace on the focused monitor, numbered by position.
- The focused workspace is highlighted using your COSMIC accent colour.
- Click a button to switch to that workspace.
- The focused window's title is shown after the buttons (shortened after 60 characters).
- Live updates from `niri msg --json event-stream`; no polling.

Workspaces are always focused by index. niri treats an all-numeric reference as an index, so a
workspace *named* `1080` would otherwise send you to workspace 1080.

## Requirements

- niri, with `niri` on `PATH` for the panel
- COSMIC panel (`cosmic-panel`)
- Rust toolchain and the libcosmic build dependencies

## Build and install

```sh
cargo build --release
install -Dm0755 target/release/cosmic-ext-applet-niri-workspaces \
    ~/.local/bin/cosmic-ext-applet-niri-workspaces
sed 's|^Exec=.*|Exec=/home/YOU/.local/bin/cosmic-ext-applet-niri-workspaces|' \
    resources/dev.paul.CosmicExtAppletNiriWorkspaces.desktop \
    > ~/.local/share/applications/dev.paul.CosmicExtAppletNiriWorkspaces.desktop
```

Replace `YOU` with your username, or install the binary and desktop file system-wide.

## Add it to the panel

Add `dev.paul.CosmicExtAppletNiriWorkspaces` to the panel's applet list in
COSMIC Settings, or edit
`~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`:

```ron
Some(([
    "dev.paul.CosmicExtAppletNiriWorkspaces",
], [
    "com.system76.CosmicAppletTime",
]))
```

Restart the panel afterwards. Start it from niri (`niri msg action spawn -- cosmic-panel`)
rather than a terminal, otherwise it exits when the terminal closes.

## Licence

GPL-3.0-only.
