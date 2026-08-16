# RPG Maker MV/MZ Cheat UI Plugin Installer

A native desktop GUI application installer for RPG Maker MV/MZ Cheat UI Plugin.

Based on [RPG-Maker-MV-MZ-Cheat-UI-Plugin](https://github.com/paramonos/RPG-Maker-MV-MZ-Cheat-UI-Plugin) by paramonos and [RPG-Maker-MV-MZ-Cheat-UI-Plugin-Web-Installer](https://github.com/nt7011/RPG-Maker-MV-MZ-Cheat-UI-Plugin-Web-Installer) by nt7011.

---

## Key Features

- Native Desktop GUI application built with Rust and Slint UI.
- Priority-based game layout detection supporting both RPG Maker MV (`www` packaged) and RPG Maker MZ / Non-www root structures.
- Prevents false-positive path detection errors caused by existing subfolders.
- Automatic existing plugin detection with dynamic Reinstall mode.
- Drag and Drop game folder support.
- Detailed error code reporting for quick troubleshooting.

---

## Usage

1. Download the executable from the Releases section.
2. Launch the application.
3. Select your game directory using the "게임 폴더 선택" button, or drag and drop the game folder into the window.
4. Verify the detected layout and folder information.
5. Click "치트 설치하기" (or "치트 재설치하기" if already installed).

---

## Error Codes

- **[ERR-001]**: Selected path does not exist or is not a directory.
- **[ERR-002]**: `plugins.js` was not found in both `www/js/plugins.js` and `js/plugins.js`.
- **[ERR-003]**: Failed to read `plugins.js` or file permission error.
- **[ERR-004]**: Failed to parse or serialize `plugins.js` JSON structure.
- **[ERR-005]**: Embedded installer manifest or asset directory not found.
- **[ERR-006]**: Failed to create target directories or write plugin files.
- **[ERR-007]**: Failed to update or save modified `plugins.js`.

---

## Building from Source

Prerequisites: Rust (2021 edition) installed.

```cmd
git clone https://github.com/sco119/RPG-Maker-MV-MZ-Cheat-UI-Plugin-Installer.git
cd RPG-Maker-MV-MZ-Cheat-UI-Plugin-Installer
cargo build --release
```

The compiled binary will be located at target/release/RPG-Maker-MV-MZ-Cheat-UI-Plugin-Installer.exe.

---

## Credits & Lineage

Original Cheat UI Plugin: paramonos
Web Installer: nt7011
Desktop Installer Application: SCO119

---

## License
This project is licensed under the MIT License - see the LICENSE file for details.
