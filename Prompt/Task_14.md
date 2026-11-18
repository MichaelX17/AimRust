Target Files: assets/models/pistol.glb, assets/LICENSES.md, src/props/gun.rs, src/props/mod.rs
Read-Only Files: none (PlayerCamera, Settings.show_gun are in Progress.md Interfaces)
Task_14: Pistol viewmodel (download + convert)

1. DOWNLOAD (allowed network step). Asset: Quaternius "Ultimate Gun Pack", license CC0.
   - Primary: https://opengameart.org/content/low-poly-guns-pack (file ultimate_gun_pack_by_quaternius.zip, ~7 MB)
   - Alternative: https://quaternius.itch.io/50-lowpoly-guns
   Download with curl/wget into /tmp (NOT into the project), unzip in /tmp.
   If the download is blocked, needs login/browser, or fails: STOP. Tell the user in 3 lines: download the zip manually, extract it, tell you the path of the chosen pistol file.
2. Allowed ONE directory listing, only inside the /tmp extraction. Choose the pistol closest to a Glock-style 9mm semi-auto (name contains "pistol"; prefer lowest poly, avoid revolvers). Record the chosen file in Progress.md Decisions.
3. Convert to GLB -> assets/models/pistol.glb:
   - If `blender` is on PATH: run headless with a short Python script (import FBX/OBJ, export `bpy.ops.export_scene.gltf(export_format='GLB')`).
   - Else if node is available: try `npx obj2gltf` / FBX2glTF.
   - Else STOP and tell the user to export manually: Blender > File > Import > FBX/OBJ, then File > Export > glTF 2.0 (.glb) to assets/models/pistol.glb.
   Do NOT add Rust crates for OBJ/FBX.
4. Budget: pistol.glb under 1 MB, under 5k triangles, textures max 512 px. If over, decimate in the Blender script or STOP and report.
5. assets/LICENSES.md: model name, author Quaternius, license CC0, source URL, original file name, date.
6. gun.rs: load `models/pistol.glb#Scene0` via AssetServer. Spawn as child of PlayerCamera at bottom-right (consts GUN_OFFSET, GUN_SCALE, GUN_ROT at the top of the file; barrel points toward -Z). Spawn on OnEnter(Countdown) if Settings.show_gun, despawn on OnEnter(Results)/OnEnter(Menu), and react live to the show_gun toggle. Static: no sway, no recoil. It must NOT have the Target component. If the file is missing: log one warning, skip, game keeps running.
7. Done when: check + clippy clean. Do not run the game. Tell the user to run it, open Sandbox, and report if the gun is visible bottom-right and correctly oriented; tune GUN_* consts from their feedback.

Scope: Target Files + Progress.md (+ /tmp for download only). Do not read or modify files outside of this scope.