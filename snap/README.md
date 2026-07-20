# Consuming `fpgad_cli` from another snap

This document is for authors of provider snaps intending to use FPGAd to control the FPGA subsystems on Linux, such as Ubuntu Core.

For `fpgad_cli`'s own command usage, see [`cli/README.md`](../cli/README.md).
For the canonical definitions of the interfaces mentioned below, see [`snapcraft.yaml`](snapcraft.yaml).

There are two supported approaches:

1. [Content interface + wrapper script](#option-1-content-interface--wrapper-script-recommended) (recommended) — stage [`wrapper/fpgad_cli_wrapper.py`](wrapper/fpgad_cli_wrapper.py) and connect to `fpgad`'s `fpgad-cli-app` content slot at runtime.
   This keeps `fpgad_cli` in sync with whichever `fpgad` daemon is installed, avoiding version drift between the CLI and daemon.
2. [`stage-snaps`](#option-2-stage-snaps-direct-binary-staging) — pull the `fpgad_cli` binary directly into your snap at build time.

Pick whichever fits your update/versioning requirements — see the [comparison](#comparison) at the end.

## Option 1: content interface + wrapper script (recommended)

Instead of copying the CLI binary at build time, you can stage the small [`fpgad_cli_wrapper.py`](wrapper/fpgad_cli_wrapper.py) script from the published `fpgad` snap and connect to `fpgad`'s `fpgad-cli-app` content slot (which exposes `$SNAP/cli` from the installed `fpgad` snap, see [`snapcraft.yaml:55-60`](snapcraft.yaml)) at runtime.
This way your snap always calls whichever `fpgad_cli` is currently installed and connected, with no rebuild required when `fpgad` updates.

**This is the recommended approach.**
Because `fpgad_cli` and the `fpgad` daemon are versioned and released together, staging the binary yourself (Option 2) risks a `fpgad_cli` build that is out of sync with the `fpgad` daemon actually running on the user's system (e.g. after the user refreshes `fpgad` but not `provider-example`).
Connecting to the content interface at runtime guarantees the CLI and daemon versions always match.

Below is a real, working example (trimmed to just the parts relevant to `fpgad_cli`.

```yaml
name: provider-example
# ...

plugs:
  fpgad-dbus:
    interface: dbus
    bus: system
    name: com.canonical.fpgad
  fpgad-cli-app:
    interface: content
    content: fpgad-cli-content
    target: $SNAP/fpgad

apps:
  init:
    command: wrapper/fpgad_cli_wrapper.py...
    plugs:
      - fpgad-dbus

parts:
  cli-wrapper:
    plugin: nil
    stage-snaps:
      - fpgad/latest/<edge|beta|candidate|stable>
    stage:
      - wrapper/fpgad_cli_wrapper.py
```

Note that the `fpgad-dbus` plug will need to be connected to each app calling `cli/fpgad_cli_wrapper.py`.

Pull the wrapper from whichever `fpgad` track/risk your snap targets .
Pick the channel that matches the `fpgad` compatibility you need, e.g.`fpgad/latest/stable`.
See the following subsection for instructions for staging from GitHub if you prefer:

```yaml
parts:
  cli_wrapper:
    plugin: dump
    source: https://github.com/canonical/fpgad.git
    source-subdir: snap/wrapper
    organize:
      fpgad_cli_wrapper.py: cli/fpgad_cli_wrapper.py
```

At runtime, the user needs to install `fpgad` and connect both the content interface and the D-Bus interface (since `fpgad_cli` talks to the `fpgad` daemon over D-Bus):

```shell
sudo snap install fpgad
sudo snap install fpgad+<component_name>   # if you require an optional component, e.g. dfx-mgr
sudo snap connect provider-example:fpgad-cli-app fpgad:fpgad-cli-content
sudo snap connect provider-example:fpgad-dbus fpgad:daemon-dbus
```

This matches the guidance printed by the wrapper script itself if it can't find `fpgad_cli` at runtime (see [`wrapper/fpgad_cli_wrapper.py`](wrapper/fpgad_cli_wrapper.py)).

Notes:

- The wrapper always invokes whatever `fpgad_cli` binary is exposed by the currently installed and connected `fpgad` snap, so it tracks `fpgad` updates automatically — no rebuild of `provider-example` needed, and the CLI stays in sync with the daemon.
- Requires explicit `snap connect` steps for both the content and D-Bus interfaces.
  Auto-connection is only possible if the interface attributes are pre-approved by the Snap Store; do not assume this happens automatically.
- Decouples your snap's release cadence from `fpgad`'s.




## Option 2: `stage-snaps` (direct binary staging)

`stage-snaps` pulls the built, published `fpgad` snap apart at build time and lets you copy `cli/fpgad_cli` straight into your own snap.
No `snap connect` step is required at install/runtime because the binary ships inside your snap.

Be aware that this approach can let `fpgad_cli` drift out of sync with the `fpgad` daemon version actually installed on the user's system, since the CLI binary is frozen at your build time while the daemon may be refreshed independently.
Prefer Option 1 unless you have a specific reason to pin the CLI binary (e.g. you need a guaranteed-available binary with no runtime connect step).

Example `snapcraft.yaml` for a consumer snap (`provider-example`):

```yaml
name: provider-example
# ...

plugs:
  fpgad-dbus:
    interface: dbus
    bus: system
    name: com.canonical.fpgad

parts:
  fpgad-cli-binary:
    plugin: nil
    stage-snaps:
      - fpgad
    stage:
      - cli/fpgad_cli
    organize:
      cli/fpgad_cli: bin/fpgad_cli
    override-prime: |
      craftctl default
      chmod +x $CRAFT_PRIME/bin/fpgad_cli

apps:
  provider-example:
    command: bin/fpgad_cli
    plugs:
      - fpgad-dbus # fpgad_cli talks to the fpgad daemon over dbus, see below
```

`fpgad_cli` itself communicates with the `fpgad` daemon over D-Bus (`com.canonical.fpgad` on the system bus — the same interface `fpgad`'s own `cli-dbus` plug uses, see [`snapcraft.yaml:61-65`](snapcraft.yaml)), so your app still needs a matching `dbus` plug connected to the running `fpgad` daemon's `daemon-dbus` slot:

```shell
sudo snap connect provider-example:fpgad-dbus fpgad:daemon-dbus
```

Notes:

- The binary is pinned to whatever revision of `fpgad` was available when you built your snap.
  It will **not** pick up updates automatically when the `fpgad` snap is refreshed on a user's system.
  You must rebuild/republish `provider-example` to pick up a newer `fpgad_cli`, and until you do, the CLI and daemon versions may not match.
- You are responsible for tracking which channel/revision of `fpgad` you build against.
- Simpler to reason about: no plug/slot wiring for the binary itself, only the D-Bus plug is needed at runtime.
