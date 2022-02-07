use crate::gate::core::*;
use crate::gate::serialize::*;
use bevy::prelude::*;
use bevy_egui::{egui, egui::RichText, EguiContext};
use dirs;
use std::ffi::OsString;
use std::fs::{self};
use std::io;
use std::path::Path;

pub struct EguiFileBrowserPlugin;

impl Plugin for EguiFileBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OpenBrowserEvent>();
        app.add_event::<NewFileEvent>();
        app.insert_resource(FileBrowser {
            open: false,
            path: dirs::home_dir()
                .expect("home dir to exist")
                .into_os_string(),
            fname: "".to_string(),
            file_type: FileType::Ron,
            title: String::from(""),
            action: BrowserAction::Open,
        });
        app.insert_resource(CurrentlyOpen { path: None });
        app.add_system(draw_browser_system);
        app.add_system(open_browser_event_system);
        app.add_system(new_file_event_system.label("new_file"));
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BrowserAction {
    Open,
    Save,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenBrowserEvent(pub BrowserAction);

fn open_browser_event_system(mut ev: EventReader<OpenBrowserEvent>, mut fb: ResMut<FileBrowser>) {
    for ev in ev.iter() {
        match ev.0 {
            BrowserAction::Open => {
                fb.open = true;
                fb.path = dirs::home_dir()
                    .expect("home dir to exist")
                    .into_os_string();
                fb.title = String::from("Open File");
                fb.action = BrowserAction::Open;
            }
            BrowserAction::Save => {
                fb.open = true;
                fb.path = dirs::home_dir()
                    .expect("home dir to exist")
                    .into_os_string();
                fb.title = String::from("Save File As...");
                fb.action = BrowserAction::Save;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FileType {
    Ron,
}

impl FileType {
    fn to_string(&self) -> String {
        match self {
            FileType::Ron => Self::RON.to_string(),
        }
    }

    fn ending(&self) -> &str {
        match self {
            FileType::Ron => Self::RON_ENDING,
        }
    }

    const RON: &'static str = "Rusty Object Notation";
    const RON_ENDING: &'static str = "ron";
}

pub struct FileBrowser {
    pub open: bool,
    path: OsString,
    fname: String,
    file_type: FileType,
    title: String,
    action: BrowserAction,
}

pub struct CurrentlyOpen {
    pub path: Option<String>,
}

fn draw_browser_system(
    egui_context: ResMut<EguiContext>,
    mut fb: ResMut<FileBrowser>,
    mut ev_save: EventWriter<SaveEvent>,
    mut ev_open: EventWriter<LoadEvent>,
) {
    if !fb.open {
        return;
    }

    let mut s = fb
        .path
        .clone()
        .into_string()
        .expect("OsString to be convertible");
    egui::Window::new(&fb.title)
        .resizable(false)
        .collapsible(false)
        .default_size(egui::Vec2::new(640.0, 320.0))
        .show(egui_context.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("\u{1F3E0}"))
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
                if ui
                    .add(egui::Button::new("\u{1F5C0}"))
                    .on_hover_text("New Folder")
                    .clicked()
                {}
                if ui
                    .add(egui::Button::new("\u{274C}"))
                    .on_hover_text("Delete")
                    .clicked()
                {}
            });

            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(320.)
                    .show(ui, |ui| {
                        egui::ScrollArea::horizontal()
                            .max_width(640.)
                            .show(ui, |ui| {
                                ui.set_min_width(640.);
                                let path = dirs::home_dir().expect("home dir to exist");
                                let ftype = fb.file_type.clone();
                                visit_dirs(&path, ui, 0, &mut s, &mut fb.fname, ftype.ending());
                            });
                    });
            });

            egui::Grid::new("browser_grid")
                .num_columns(3)
                .spacing([10.0, 4.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Look in: ");
                    ui.label(&s);
                    ui.end_row();

                    ui.label("File name:");
                    ui.add(egui::TextEdit::singleline(&mut fb.fname).desired_width(480.));
                    if fb.action == BrowserAction::Save {
                        let mut p = Path::new(&s).join(&fb.fname);

                        if ui.add(egui::Button::new("Save")).clicked() {
                            p.set_extension(fb.file_type.ending());
                            ev_save.send(SaveEvent(p.into_os_string().into_string().unwrap()));
                            fb.open = false;
                        }
                    } else {
                        let p = Path::new(&s).join(&fb.fname);

                        if ui.add(egui::Button::new("Open")).clicked() {
                            ev_open.send(LoadEvent(p.into_os_string().into_string().unwrap()));
                            fb.open = false;
                        }
                    }
                    ui.end_row();

                    ui.label("File type: ");
                    egui::ComboBox::from_label("Type")
                        .selected_text(format!("{}", fb.file_type.to_string()))
                        .width(320.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut fb.file_type, FileType::Ron, FileType::RON);
                        });
                    if ui.add(egui::Button::new("cancle")).clicked() {
                        fb.open = false;
                    }
                    ui.end_row();
                });
        });
    fb.path = OsString::from(&s);
}

fn visit_dirs(
    dir: &Path,
    ui: &mut egui::Ui,
    depth: usize,
    selected_path: &mut String,
    selected_file: &mut String,
    ending: &str,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Ok(fname) = entry.file_name().into_string() {
                if !fname.starts_with('.') {
                    if path.is_dir() {
                        let response = egui::CollapsingHeader::new(format!("\u{1F5C0} {}", fname))
                            .selected(path.to_str().unwrap().to_string() == *selected_path)
                            .show(ui, |ui| {
                                visit_dirs(
                                    &path,
                                    ui,
                                    depth + 1,
                                    selected_path,
                                    selected_file,
                                    ending,
                                );
                            });

                        if response.header_response.clicked() {
                            *selected_path = path.to_str().unwrap().to_string();
                        }
                    } else if fname.ends_with(ending) {
                        if ui
                            .add(
                                egui::Label::new(
                                    RichText::new(format!("\u{1F5B9} {}", fname)).background_color(
                                        if fname == *selected_file {
                                            egui::Color32::from_rgba_premultiplied(0, 0, 255, 30)
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        },
                                    ),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .clicked()
                        {
                            if let Some(parent) = path.parent() {
                                *selected_path = parent.to_str().unwrap().to_string();
                            }
                            *selected_file = fname;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub struct NewFileEvent;

pub fn new_file_event_system(
    mut commands: Commands,
    mut nev: EventReader<NewFileEvent>,
    q_all: Query<Entity, Or<(With<NodeType>, With<ConnectionLine>)>>,
    mut curr_open: ResMut<CurrentlyOpen>,
) {
    for _ev in nev.iter() {
        for e in q_all.iter() {
            commands.entity(e).despawn_recursive();
        }
        curr_open.path = None;
    }
}
