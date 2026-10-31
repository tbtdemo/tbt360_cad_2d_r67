# R47 run/build note

R47 source contains the all-in-one hardening changes. The conversation TEST-NOW package includes the known runnable baseline executable only so it can be opened without installing a development toolchain. That baseline executable is not an R47 build and will not expose R47 UI/features.

To expose R47 in Windows, `CAD-2D-TBT.exe` must be compiled from this R47 source with its declared Rust/Windows dependencies. Do not rename an older executable as R47.
