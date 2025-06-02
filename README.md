# FPGAd

FPGAd is a WIP rust project. 
It provides a strictly confined snap which will provide a mechanism to dynamically load and unload bitstreams to FPGA devices on Ubuntu Core.
It mitigates the need for other snaps containing bitstreams to have super privileges. 
Instead, those other snaps will connect to FPGAd and FPGAd will have the necessary privileges.
This is for several reasons, but mostly to:
1) make the snap store team happy (not publishing lots of super privileged snaps).
2) add a place for checking and verifying bitstreams before loading them for additional safety/security.
3) make it easier to port already existing bitstream packages to snap store.


In order to understand why the super privileges are necessary, you must understand that loading a bitstream fundamentally changes the FPGA device's behaviour including what the connections to the CPU are/do. 
To do this, device tree of the system must be changed (overlaid).
This device tree tells Ubuntu a lot of information about the device, such as how much ram the system has. 
It should be clear to see, then, that compromising the device tree is very powerful.
