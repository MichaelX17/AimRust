# AimRust Packaging & AppImage Metadata

This directory contains metadata and build scripts for packaging AimRust as an AppImage or standalone package.

## AppDir Layout

When built with `build-appimage.sh`, the AppDir structure is prepared as follows:

```text
AppDir/
├── AppRun
├── .DirIcon -> aim-rust.png
├── io.github.michaelx17.AimRust.desktop
├── aim-rust.png
├── .appimage-hub.json
└── usr/
    ├── bin/
    │   ├── AimRust
    │   └── assets/
    └── share/
        ├── applications/
        │   └── io.github.michaelx17.AimRust.desktop
        ├── icons/hicolor/512x512/apps/
        │   └── aim-rust.png
        └── metainfo/
            └── io.github.michaelx17.AimRust.metainfo.xml
```

## Files

| Source File | Description / Destination |
| --- | --- |
| `build-appimage.sh` | Shell script to compile release binary and generate AppImage |
| `io.github.michaelx17.AimRust.desktop` | Freedesktop desktop specification entry |
| `io.github.michaelx17.AimRust.metainfo.xml` | AppStream metadata specification |
| `aim-rust.png` | 1500x1500 application icon |
| `.appimage-hub.json` | AppImage Hub catalog metadata |

## Portable Mode

AimRust supports portable mode. If a `portable.ini` file is placed in the application folder (alongside the executable / in the AppDir working directory), AimRust will create an `AimRust.config` folder there and save all settings and round scores locally instead of using system directories (`~/.config`).
