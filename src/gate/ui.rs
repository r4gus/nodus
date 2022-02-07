use bevy::prelude::*;
use bevy::app::AppExit;
use nodus::world2d::*;
use nodus::world2d::camera2d::MainCamera;
use nodus::world2d::interaction2d::*;
use bevy_egui::{egui, EguiContext, EguiSettings};
use crate::gate::{
    undo::*,
    serialize::*,
    core::{*, Name},
    graphics::clk::Clk,
    graphics::gate::ChangeInput,
    file_browser::*,
};
use crate::radial_menu::Menu;

const MIT: &str = "\
License

Copyright (c) 2021-2022 David Pierre Sugar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the Software), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED AS IS, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.";

const NODUS_LOGO_ID: u64 = 0;

pub fn update_lock(
    mut lock: ResMut<Lock>,
    about: Res<GuiMenu>,
    browser: Res<FileBrowser>,
    q_menu: Query<&Menu>,
) {
    let menu = if let Ok(_) = q_menu.get_single() { true } else { false };

    lock.0 = about.open || browser.open || menu;
}

pub fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.5;
    }
}

pub fn load_gui_assets(
    mut egui_context: ResMut<EguiContext>,
    assets: Res<AssetServer>,
) {
    let texture_handle = assets.load("misc/LOGO.png");
    egui_context.set_egui_texture(NODUS_LOGO_ID, texture_handle);
}

pub fn ui_scroll_system(
    egui_context: ResMut<EguiContext>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(mut transform) = q_camera.get_single_mut() {
        egui::Area::new("zoom_area")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(5., -5.))
            .show(egui_context.ctx(), |ui| {
                let mut x = transform.scale.x;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("\u{B1}").strong().color(egui::Color32::BLACK));
                    ui.add(egui::Slider::new(&mut x, 1.0..=5.0)
                           .show_value(false)
                    );
                    ui.label(egui::RichText::new("\u{1F50E}").strong().color(egui::Color32::BLACK));
                });
                transform.scale = Vec3::new(x, x, x);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMenuOptions {
    None,
    Handbook,
    About,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiMenu {
    pub option: GuiMenuOptions,
    pub open: bool,
}

pub fn ui_gui_about(
    egui_context: ResMut<EguiContext>,
    mut r: ResMut<GuiMenu>,
) {
    if let GuiMenuOptions::About = r.option {
        egui::Window::new("About")
            .resizable(false)
            .collapsible(false)
            .default_size(egui::Vec2::new(900.0, 600.0))
            .open(&mut r.open)
            .show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.add(egui::widgets::Image::new(
                            egui::TextureId::User(NODUS_LOGO_ID),
                            [80., 80.],
                        ));
                    });
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Nodus v0.1");

                            ui.horizontal_wrapped(|ui| {
                                ui.label("Nodus is a digital circuit simulator written in rust, using the");
                                ui.hyperlink_to(
                                    format!("bevy"),
                                    "https://bevyengine.org"
                                );
                                ui.label("game engine.");
                            });

                            ui.label("Copyright \u{A9} 2022 David Pierre Sugar <david(at)thesugar.de>.\n");

                            ui.collapsing("Third-party libraries", |ui| {
                                ui.label("Nodus is built on the following free software libraries:");

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Bevy Game Engine"),
                                        "https://bevyengine.org"
                                    );
                                    ui.label("MIT/ Apache Version 2.0");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Bevy-Prototype-Lyon"),
                                        "https://github.com/Nilirad/bevy_prototype_lyon"
                                    );
                                    ui.label("MIT/ Apache Version 2.0");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Bevy-Egui"),
                                        "https://github.com/mvlabat/bevy_egui"
                                    );
                                    ui.label("MIT");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Egui"),
                                        "https://github.com/emilk/egui"
                                    );
                                    ui.label("Apache Version 2.0");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Bevy-Asset-Loader"),
                                        "https://github.com/NiklasEi/bevy_asset_loader"
                                    );
                                    ui.label("MIT/ Apache Version 2.0");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Lyon"),
                                        "https://github.com/nical/lyon"
                                    );
                                    ui.label("Apache Version 2.0");
                                });

                                ui.horizontal_wrapped(|ui| {
                                    ui.hyperlink_to(
                                        format!("Rusty Object Notation"),
                                        "https://github.com/ron-rs/ron"
                                    );
                                    ui.label("Apache Version 2.0");
                                });
                            });

                            ui.collapsing("Your Rights", |ui| {
                                ui.label("Nodus is released under the MIT License.");
                                ui.label("You are free to use Nodus for any purpose.");
                                ui.with_layout(
                                    egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                                    |ui| {
                                        ui.label(egui::RichText::new(MIT).small().weak());
                                    },
                                );
                            });
                        });
                    });
                });
            });
    }
}

pub fn ui_top_panel_system(
    egui_context: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
    mut fbe: EventWriter<OpenBrowserEvent>,
    mut ev_save: EventWriter<SaveEvent>,
    mut ev_new: EventWriter<NewFileEvent>,
    mut ev_undo: EventWriter<UndoEvent>,
    mut r: ResMut<GuiMenu>,
    curr_open: Res<CurrentlyOpen>,
    mut mode: ResMut<InteractionMode>,
) {
    egui::TopBottomPanel::top("side").show(egui_context.ctx(), |ui| {
        ui.columns(2, |columns| {
            columns[0].horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    ui.add_enabled_ui(true, |ui| {
                        if ui.button("\u{2B} New").clicked() {
                            if let Some(path) = &curr_open.path {
                                ev_save.send(SaveEvent(path.clone()));
                            }
                            ev_new.send(NewFileEvent);
                            ui.close_menu();
                        }
                        if ui.button("\u{1F5C1} Open").clicked() {
                            fbe.send(OpenBrowserEvent(BrowserAction::Open));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("\u{1F4BE} Save").clicked() {
                            if let Some(path) = &curr_open.path {
                                ev_save.send(SaveEvent(path.clone()));
                            } else {
                                fbe.send(OpenBrowserEvent(BrowserAction::Save));
                            }
                            ui.close_menu();
                        }
                        if ui.button("\u{1F4BE} Save As...").clicked() {
                            fbe.send(OpenBrowserEvent(BrowserAction::Save));
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.close_menu();
                        exit.send(AppExit);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Back to Origin").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.separator();
                    if ui.button("\u{FF1F} About Nodus").clicked() {
                        r.option = GuiMenuOptions::About;
                        r.open = true;
                        ui.close_menu();
                    }
                });

            });

            columns[1].with_layout(egui::Layout::right_to_left(), |ui| {
                let blue = egui::Color32::BLUE;
                let grey = egui::Color32::DARK_GRAY;
                if ui.add(egui::Button::new("\u{1F542}")
                    .fill(if *mode == InteractionMode::Pan { blue } else { grey }))
                    .on_hover_text("Pan Camera")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked() 
                { // pan
                    *mode = InteractionMode::Pan;
                }
                if ui.add(egui::Button::new("\u{1F446}")
                    .fill(if *mode == InteractionMode::Select { blue } else { grey }))
                    .on_hover_text("Select")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked() 
                { // select
                    *mode = InteractionMode::Select;
                }

                if ui.add(egui::Button::new("redo"))
                    .on_hover_text("Redo last action")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    ev_undo.send(UndoEvent::Redo); 
                }
                if ui.add(egui::Button::new("undo"))
                    .on_hover_text("Undo last action")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    ev_undo.send(UndoEvent::Undo); 
                }
            });
        });
    });
}

pub fn ui_node_info_system(
    egui_context: ResMut<EguiContext>,
    mut q_gate: Query<(Entity, &Name, Option<&Gate>, Option<&mut Clk>), With<Selected>>,
    mut ev_change: EventWriter<ChangeInput>,
    mut mb: ResMut<Input<MouseButton>>,
) {
    if let Ok((entity, name, gate, mut clk)) = q_gate.get_single_mut() {
        if let Some(response) = egui::Window::new(&name.0)
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-5., -5.))
            .resizable(false)
            .show(egui_context.ctx(), |ui| {
                ui.label(&name.0);

                if let Some(gate) = gate {
                    if gate.in_range.min != gate.in_range.max {
                        if ui
                            .horizontal(|ui| {
                                ui.label("Input Count: ");
                                if ui.button("➖").clicked() {
                                    if gate.inputs > gate.in_range.min {
                                        ev_change.send(ChangeInput {
                                            gate: entity,
                                            to: gate.inputs - 1,
                                        });
                                    }
                                }
                                ui.label(format!("{}", gate.inputs));
                                if ui.button("➕").clicked() {
                                    if gate.inputs < gate.in_range.max {
                                        ev_change.send(ChangeInput {
                                            gate: entity,
                                            to: gate.inputs + 1,
                                        });
                                    }
                                }
                            })
                            .response
                            .hovered()
                        {
                            mb.reset(MouseButton::Left);
                        }
                    }
                }

                if let Some(ref mut clk) = clk {
                    let mut clk_f32 = clk.0 * 1000.;
                    if ui
                        .horizontal(|ui| {
                            ui.label("Signal Duration: ");
                            ui.add(egui::DragValue::new(&mut clk_f32)
                                    .speed(1.0)
                                    .clamp_range(std::ops::RangeInclusive::new(250.0, 600000.0))); 
                        })
                        .response
                        .hovered()
                    {
                        mb.reset(MouseButton::Left);
                    }
                    clk.0 = clk_f32 / 1000.;
                }
            })
        {
            if response.response.hovered() {
                mb.reset(MouseButton::Left);
            }
        }
    }
}
