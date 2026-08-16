#![windows_subsystem = "windows"]

use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use i_slint_backend_winit::winit::event::WindowEvent;
use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
use slint::ComponentHandle;

slint::include_modules!();

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    bundle_directory: String,
    loader_file: String,
    support_directory: String,
    support_files: Vec<String>,
    plugin_entry: String,
}

#[derive(Clone, Debug)]
struct Inspection {
    valid: bool,
    layout_label: String,
    base_dir: PathBuf,
    plugins_dir: PathBuf,
    plugins_file: PathBuf,
    reason: String,
    is_reinstall: bool,
}

fn main() -> AppResult<()> {
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();

    let update_ui = |ui: &AppWindow, path: &Path| {
        let inspection = inspect_game_directory(path);
        let actual_folder = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        ui.set_folder_path(actual_folder.to_string_lossy().as_ref().into());
        ui.set_status_text(inspection.reason.as_str().into());
        ui.set_layout_text(inspection.layout_label.as_str().into());

        if inspection.valid {
            ui.set_plugin_target_text(inspection.plugins_dir.to_string_lossy().as_ref().into());
            ui.set_plugins_js_text(inspection.plugins_file.to_string_lossy().as_ref().into());
            ui.set_install_button_text(
                if inspection.is_reinstall { "치트 재설치" } else { "치트 설치" }.into()
            );
        } else {
            ui.set_plugin_target_text("찾을 수 없음".into());
            ui.set_plugins_js_text("찾을 수 없음".into());
            ui.set_install_button_text("치트 설치".into());
        }

        ui.set_install_enabled(inspection.valid);
    };

    let weak = ui_weak.clone();
    ui.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::DroppedFile(path) = event {
            if let Some(ui) = weak.upgrade() {
                update_ui(&ui, path);
            }
        }
        EventResult::Propagate
    });

    let weak = ui_weak.clone();
    ui.on_pick_folder_clicked(move || {
        let weak_inner = weak.clone();
        thread::spawn(move || {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_inner.upgrade() {
                        update_ui(&ui, &folder);
                    }
                });
            }
        });
    });

    let weak = ui_weak.clone();
    ui.on_install_clicked(move || {
        let Some(ui) = weak.upgrade() else { return };

        let folder = PathBuf::from(ui.get_folder_path().to_string());
        ui.set_install_enabled(false);
        ui.set_status_text("치트 설치 작업 진행 중...".into());

        let weak_inner = weak.clone();
        thread::spawn(move || {
            let inspection = inspect_game_directory(&folder);

            let result: Result<String, String> = if inspection.valid {
                install_cheat_plugin(&inspection).map_err(|e| e.to_string())
            } else {
                Err(inspection.reason.clone())
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = weak_inner.upgrade() {
                    match result {
                        Ok(msg) => {
                            let text = if inspection.is_reinstall {
                                format!("재설치 성공: {}", msg)
                            } else {
                                format!("설치 성공: {}", msg)
                            };
                            ui.set_status_text(text.as_str().into());
                            ui.set_install_button_text("치트 재설치하기".into());
                            ui.set_install_enabled(true);
                        }
                        Err(e) => {
                            ui.set_status_text(format!("설치 실패: {}", e).into());
                            ui.set_install_enabled(true);
                        }
                    }
                }
            });
        });
    });

    ui.run()?;
    Ok(())
}

fn inspect_game_directory(target_path: &Path) -> Inspection {
    let mut inspection = Inspection {
        valid: false,
        layout_label: "알 수 없음".into(),
        base_dir: PathBuf::new(),
        plugins_dir: PathBuf::new(),
        plugins_file: PathBuf::new(),
        reason: "[ERR-001] 지정된 경로가 존재하지 않거나 올바른 폴더가 아닙니다.".into(),
        is_reinstall: false,
    };

    if !target_path.exists() {
        return inspection;
    }

    let mut dir = if target_path.is_file() {
        target_path.parent().unwrap_or(target_path).to_path_buf()
    } else {
        target_path.to_path_buf()
    };

    if dir.file_name().and_then(|n| n.to_str()) == Some("www") {
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        }
    }

    let www_plugins_file = dir.join("www").join("js").join("plugins.js");
    let root_plugins_file = dir.join("js").join("plugins.js");

    let (base_dir, layout_name) = if www_plugins_file.exists() {
        (dir.join("www"), "MV (www 패키징)")
    } else if root_plugins_file.exists() {
        (dir.clone(), "MZ 또는 MV 루트")
    } else {
        inspection.reason = "[ERR-002] plugins.js 파일을 찾을 수 없습니다. (www/js/plugins.js 및 js/plugins.js 탐색 실패)".into();
        return inspection;
    };

    let js_dir = base_dir.join("js");
    let plugins_dir = js_dir.join("plugins");
    let plugins_file = js_dir.join("plugins.js");

    inspection.layout_label = layout_name.into();
    inspection.base_dir = base_dir;
    inspection.plugins_dir = plugins_dir.clone();
    inspection.plugins_file = plugins_file.clone();

    if !js_dir.exists() {
        inspection.reason = "[ERR-002] js 폴더를 찾을 수 없습니다.".into();
    } else if !plugins_dir.exists() {
        inspection.reason = "[ERR-002] js/plugins 폴더를 찾을 수 없습니다.".into();
    } else if !plugins_file.exists() {
        inspection.reason = "[ERR-002] js/plugins.js 파일을 찾을 수 없습니다.".into();
    } else {
        inspection.valid = true;
        if let Ok(content) = fs::read_to_string(&plugins_file) {
            if content.contains("CheatBridge") {
                inspection.is_reinstall = true;
                inspection.reason = "기존 치트 플러그인이 감지되었습니다. (재설치 가능)".into();
            } else {
                inspection.reason = "올바른 게임 폴더입니다. 설치가 가능합니다.".into();
            }
        } else {
            inspection.reason = "올바른 게임 폴더입니다. 설치가 가능합니다.".into();
        }
    }

    inspection
}

fn install_cheat_plugin(inspection: &Inspection) -> AppResult<String> {
    let manifest_file = ASSETS_DIR
        .get_file("installer-manifest.json")
        .ok_or("[ERR-005] assets/installer-manifest.json 파일을 찾을 수 없습니다.")?;

    let manifest: Manifest = serde_json::from_slice(manifest_file.contents())
        .map_err(|e| format!("[ERR-004] 매니페스트 파싱 실패: {}", e))?;
    let target_plugins_dir = &inspection.plugins_dir;

    let bundle_asset_dir = ASSETS_DIR
        .get_dir(&manifest.bundle_directory)
        .ok_or_else(|| format!("[ERR-005] assets/{} 번들 디렉토리를 찾을 수 없습니다.", manifest.bundle_directory))?;

    let mut files_copied = 0;

    let loader_relative = Path::new(&manifest.bundle_directory).join(&manifest.loader_file);
    if let Some(loader_asset) = ASSETS_DIR.get_file(&loader_relative) {
        fs::write(target_plugins_dir.join(&manifest.loader_file), loader_asset.contents())
            .map_err(|e| format!("[ERR-006] loader 파일 쓰기 실패: {}", e))?;
        files_copied += 1;
    } else if let Some(loader_asset) = bundle_asset_dir.get_file(&manifest.loader_file) {
        fs::write(target_plugins_dir.join(&manifest.loader_file), loader_asset.contents())
            .map_err(|e| format!("[ERR-006] loader 파일 쓰기 실패: {}", e))?;
        files_copied += 1;
    } else {
        return Err(format!("[ERR-005] 번들에서 loader 파일({})을 찾을 수 없습니다.", manifest.loader_file).into());
    }

    let support_target_dir = target_plugins_dir.join(&manifest.support_directory);
    fs::create_dir_all(&support_target_dir)
        .map_err(|e| format!("[ERR-006] 지원 디렉터리 생성 실패: {}", e))?;

    for file_name in &manifest.support_files {
        let file_relative = Path::new(&manifest.bundle_directory).join(file_name);
        let asset_file = ASSETS_DIR.get_file(&file_relative)
            .or_else(|| bundle_asset_dir.get_file(file_name));

        if let Some(file) = asset_file {
            let target_file_path = support_target_dir.join(file_name);
            if let Some(parent) = target_file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("[ERR-006] 지원 하위 디렉터리 생성 실패: {}", e))?;
            }
            fs::write(target_file_path, file.contents())
                .map_err(|e| format!("[ERR-006] 지원 파일 쓰기 실패: {}", e))?;
            files_copied += 1;
        } else {
            return Err(format!("[ERR-005] 지원 파일({})을 찾을 수 없습니다.", file_name).into());
        }
    }

    let root_dir = inspection.base_dir.parent().unwrap_or(&inspection.base_dir);
    let _ = patch_package_json(root_dir);

    if let Some(version_file) = ASSETS_DIR.get_file("version.json") {
        let _ = fs::write(support_target_dir.join("version.json"), version_file.contents());
    }

    inject_plugin_entry(&inspection.plugins_file, &manifest.plugin_entry)?;

    Ok(format!("파일 {}개 복사 완료", files_copied))
}

fn patch_package_json(dir: &Path) -> AppResult<()> {
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&pkg_path)?;
    if let Ok(mut json_val) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(name) = json_val.get("name").and_then(|n| n.as_str()) {
            if name.is_empty() {
                let backup_path = dir.join("package.json.backup");
                if !backup_path.exists() {
                    let _ = fs::copy(&pkg_path, backup_path);
                }
                json_val["name"] = serde_json::Value::String("Game".into());
                let _ = fs::write(pkg_path, serde_json::to_string_pretty(&json_val)?);
            }
        }
    }
    Ok(())
}

fn inject_plugin_entry(plugins_file: &Path, plugin_entry: &str) -> AppResult<()> {
    let content = fs::read_to_string(plugins_file)
        .map_err(|e| format!("[ERR-003] plugins.js 읽기 실패: {}", e))?;

    if content.contains("CheatBridge") {
        return Ok(());
    }

    if let Some(parent) = plugins_file.parent() {
        let backup_path = parent.join("plugins.js.backup");
        if !backup_path.exists() {
            let _ = fs::copy(plugins_file, backup_path);
        }
    }

    let close_idx = content.rfind("];").ok_or("[ERR-007] plugins.js 구문 오류: '];' 닫힘 표시를 찾을 수 없습니다.")?;
    let (head, tail) = content.split_at(close_idx);

    let mut new_content = String::with_capacity(content.len() + plugin_entry.len() + 10);
    new_content.push_str(head);

    let head_trimmed = head.trim_end();
    if !head_trimmed.ends_with('[') && !head_trimmed.ends_with(',') {
        new_content.push(',');
    }
    new_content.push('\n');
    new_content.push_str(plugin_entry);
    new_content.push_str(tail);

    fs::write(plugins_file, new_content)
        .map_err(|e| format!("[ERR-007] plugins.js 쓰기 실패: {}", e))?;

    Ok(())
}