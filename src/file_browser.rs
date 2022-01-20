use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiSettings};
use std::fs::{self, DirEntry};
use std::ffi::OsString;
use std::path::Path;
use std::io;
use dirs;

pub struct EguiFileBrowserPlugin;

impl Plugin for EguiFileBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OpenBrowserEvent>();
        app.insert_resource(
            FileBrowser {
                open: false,
                path: dirs::home_dir().expect("home dir to exist").into_os_string(),
                title: String::from(""),
            }
        );
        app.add_system(draw_browser_system);
        app.add_system(open_browser_event_system);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BrowserAction {
    Open,
    Save,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenBrowserEvent(pub BrowserAction);

fn open_browser_event_system(
    mut ev: EventReader<OpenBrowserEvent>,
    mut fb: ResMut<FileBrowser>,
) {
    for ev in ev.iter() {
        match ev.0 {
            BrowserAction::Open => {
                fb.open = true;
                fb.path = dirs::home_dir().expect("home dir to exist").into_os_string();
                fb.title = String::from("Open File");
            },
            BrowserAction::Save => {
                fb.open = true;
                fb.path = dirs::home_dir().expect("home dir to exist").into_os_string();
                fb.title = String::from("Save File As...");
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FileBrowser {
    open: bool,
    path: OsString,
    title: String,
}

fn draw_browser_system(
    egui_context: ResMut<EguiContext>,
    mut fb: ResMut<FileBrowser>,
) {
    if !fb.open {
        return;
    }

    let mut s = fb.path.clone().into_string().expect("OsString to be convertible");
   egui::Window::new(&fb.title)
        .resizable(false)
        .collapsible(false)
        .default_size(egui::Vec2::new(640.0, 320.0))
        .show(egui_context.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("\u{1F3E0}"))
                    .on_hover_text("Home Directory")
                    .clicked() 
                {
                    if let Some(home_dir) = dirs::home_dir() {
                        if let Ok(home_dir_str) = home_dir.into_os_string().into_string() {
                            s = home_dir_str;
                        }
                    }
                }
                ui.separator();
                if ui.add(egui::Button::new("\u{1F5C0}"))
                    .on_hover_text("New Folder")
                    .clicked() 
                {
                    
                }
                if ui.add(egui::Button::new("\u{274C}"))
                    .on_hover_text("Delete")
                    .clicked() {

                }
            });

            ui.add(egui::TextEdit::singleline(&mut s).desired_width(f32::INFINITY));

            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(320.)
                    .show(ui, |ui| {
                        egui::ScrollArea::horizontal()
                            .max_width(640.)
                            .show(ui, |ui| {
                                ui.set_min_width(640.);
                                let path = dirs::home_dir().expect("home dir to exist");
                                visit_dirs(&path, ui, 0, &mut s);
                        });
                });
            });

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("ok")).clicked() {

                }
                if ui.add(egui::Button::new("cancle")).clicked() {
                    fb.open = false;
                }
            });
        });
        fb.path = OsString::from(&s);
}

fn visit_dirs(dir: &Path, ui: &mut egui::Ui, depth: usize, selected: &mut String) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Ok(fname) = entry.file_name().into_string() {
                let full_path = dir.join(fname.clone()).to_str().unwrap().to_string();

                if !fname.starts_with('.') {
                    if path.is_dir() {
                        let response = egui::CollapsingHeader::new(format!("\u{1F5C0} {}", fname))
                            .selected(full_path == *selected)
                            .show(ui, |ui| {
                                visit_dirs(&path, ui, depth + 1, selected);
                            });

                        if response.header_response.clicked() {
                            *selected = full_path;
                        }
                    } else {
                        if ui.add(egui::Label::new(format!("\u{1F5B9} {}", fname))
                                .background_color(if full_path == *selected { egui::Color32::from_rgba_premultiplied(0, 0, 255, 125) } else { egui::Color32::TRANSPARENT }) 
                                .sense(egui::Sense::click())).clicked() {
                            *selected = full_path;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
