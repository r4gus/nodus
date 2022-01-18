use bevy::prelude::*;
use bevy::app::AppExit;
use nodus::world2d::camera2d::MainCamera;
use nodus::world2d::interaction2d::*;
use bevy_egui::{egui, EguiContext, EguiSettings};
use crate::gate::{
    core::{*, Name},
    graphics::clk::Clk,
    graphics::gate::ChangeInput,
};

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

pub fn ui_top_panel_system(
    egui_context: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
) {
    egui::TopBottomPanel::top("side").show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("\u{1F5C1} Open").clicked() {
                    // TODO: Open file...
                    ui.close_menu();
                }
                if ui.button("\u{1F4BE} Save All").clicked() {
                    // TODO: Save file...
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ui.close_menu();
                    exit.send(AppExit);
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
