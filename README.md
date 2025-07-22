# FPGAd

[![fpgad](https://snapcraft.io/fpgad/badge.svg)](https://snapcraft.io/fpgad)

FPGAd is a WIP rust project.
It provides a strictly confined snap which will provide a mechanism to dynamically load and unload bitstreams to FPGA
devices on Ubuntu Core.
It mitigates the need for other snaps containing bitstreams to have super privileges.
Instead, those other snaps will connect to FPGAd and FPGAd will have the necessary privileges.
This is for several reasons, but mostly to:

1. make the snap store team happy (not publishing lots of super privileged snaps).
2. add a place for checking and verifying bitstreams before loading them for additional safety/security.
3. make it easier to port already existing bitstream packages to snap store.

In order to understand why the super privileges are necessary, you must understand that loading a bitstream
fundamentally changes the FPGA device's behaviour including what the connections to the CPU are/do.
To do this, device tree of the system must be changed (overlaid).
This device tree tells Ubuntu a lot of information about the device, such as how much ram the system has.
It should be clear to see, then, that compromising the device tree is very powerful.

# Key considerations

#### fw_search_path:

The kernel system only works with relative paths and searches relative to provided `fw_search_path` and hardcoded
firmware
locations see [the kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html) for details. When
loading an overlay, only the relative path of the loaded overlay file is retrievable. If there is a file with the same
name in the default search paths (such as `/lib/firmware`), there is no guarantee that the loaded file is the one
provided by the user since the system will successfully load from there even in the event that the file is not present
at the provided path. It is up to the user to verify that the correct overlay is applied, such as by checking that the
appropriate drivers are loaded.

# Anticipated Architecture

![anticipated_architecture.png](docs/assets/anticipated_architecture.png)
