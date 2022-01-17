use crate::radial_menu::{OpenMenuEvent, PropagateSelectionEvent, UpdateCursorPositionEvent};
use crate::{FontAssets, GameState};
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_egui::{egui, EguiContext};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::SvgPathShape;
use nodus::world2d::camera2d::MouseWorldPos;
use nodus::world2d::interaction2d::*;
use std::sync::atomic::{AtomicI32, Ordering};
use lyon_tessellation::path::path::Builder;
use std::collections::HashMap;

use crate::gate::{
    core::{*, State, Name, trans},
    systems::*,
    graphics::{
        light_bulb::*,
        toggle_switch::*,
        gate::*,
        connector::*,
        connection_line::*,
        background::*,
        clk::*,
        Z_INDEX,
        GATE_SIZE, GATE_WIDTH, GATE_HEIGHT,
    },
};

pub struct LogicComponentSystem;

const NODE_GROUP: u32 = 1;
const CONNECTOR_GROUP: u32 = 2;

impl Plugin for LogicComponentSystem {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectEvent>()
            .add_event::<ChangeInput>()
            .add_event::<DisconnectEvent>()
            .insert_resource(MenuState(MenuStates::Idle))
            .insert_resource(LineResource {
                count: 0.,
                timestep: 0.5,
                update: false,
            })
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .before("interaction2d")
                    .label("level3_node_set")
                    .with_system(ui_node_info_system.system()),
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("level2_node_set")
                    .after("interaction2d")
                    // It's important to run disconnect before systems that delete
                    // nodes (and therefore connectors) because disconnect_event
                    // wants to insert(Free) connectors even if they are queued for
                    // deletion.
                    .with_system(disconnect_event_system.system().label("disconnect"))
                    .with_system(delete_gate_system.system().after("disconnect"))
                    .with_system(change_input_system.system().after("disconnect"))
                    .with_system(delete_line_system.system().after("disconnect"))
                    .with_system(transition_system.system().label("transition"))
                    .with_system(propagation_system.system().after("transition"))
                    .with_system(highlight_connector_system.system())
                    .with_system(drag_gate_system.system())
                    .with_system(drag_connector_system.system().label("drag_conn_system"))
                    .with_system(connect_event_system.system().after("drag_conn_system"))
                    // Draw Line inserts a new bundle into an entity that might has been
                    // deleted by delete_line_system, i.e. we run it before any deletions
                    // to prevent an segfault.
                    .with_system(
                        draw_line_system
                            .system()
                            .label("draw_line")
                            .before("disconnect"),
                    )
                    .with_system(draw_data_flow.system().after("draw_line"))
                    .with_system(light_bulb_system.system().before("disconnect"))
                    .with_system(toggle_switch_system.system().before("disconnect"))
                    .with_system(line_selection_system.system().after("draw_line"))
                    .with_system(draw_background_grid_system)
                    .with_system(clk_system)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("level1_node_set")
                    .after("level2_node_set")
                    .with_system(open_radial_menu_system.system())
                    .with_system(handle_radial_menu_event_system.system())
                    .with_system(update_radial_menu_system.system())
            )
            .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(setup.system()));

        info!("NodePlugin loaded");
    }
}



impl Gate {
    fn and_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.line_to(Vec2::new(-half, -half));
        path.line_to(Vec2::new(0., -half));
        path.arc(
            Vec2::new(0., 0.),
            Vec2::new(half, half),
            std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }

    fn invert(mut path: PathBuilder) -> PathBuilder {
        let radius = GATE_SIZE * 0.1;
        let half = GATE_SIZE / 2.;

        path.arc(
            Vec2::new(half + radius + 5., 0.),
            Vec2::new(radius, radius),
            std::f32::consts::PI * 2.,
            0.,
        );
        path
    }

    fn nand_gate_path() -> PathBuilder {
        Gate::invert(Gate::and_gate_path())
    }

    fn not_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.line_to(Vec2::new(-half, -half));
        path.line_to(Vec2::new(half, 0.));
        path.close();
        Gate::invert(path)
    }

    fn or_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;

        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(-half, half));
        path.arc(
            Vec2::new(-half, 0.),
            Vec2::new(half / 2., half),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(0., -half));
        path.arc(
            Vec2::new(0., 0.),
            Vec2::new(half, half),
            std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }

    fn nor_gate_path() -> PathBuilder {
        Gate::invert(Gate::or_gate_path())
    }

    fn xor_gate_path() -> PathBuilder {
        let half = GATE_SIZE / 2.;
        let mut path = Gate::or_gate_path();

        path.move_to(Vec2::new(-half - 15., half));
        path.arc(
            Vec2::new(-half - 15., 0.),
            Vec2::new(half / 2., half),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(-half - 16., -half));
        path.arc(
            Vec2::new(-half - 16., 0.),
            Vec2::new(half / 2., half),
            std::f32::consts::PI,
            0.,
        );
        path
    }

    fn xnor_gate_path() -> PathBuilder {
        Gate::invert(Gate::xor_gate_path())
    }

    fn toggle_switch_path_a() -> PathBuilder {
        let radius = GATE_SIZE / 4.;
        let mut path = PathBuilder::new();

        path.move_to(Vec2::new(-radius, -radius));
        path.arc(
            Vec2::new(-radius, 0.),
            Vec2::new(radius, radius),
            -std::f32::consts::PI,
            0.,
        );
        path.line_to(Vec2::new(radius, radius));
        path.arc(
            Vec2::new(radius, 0.),
            Vec2::new(radius, radius),
            -std::f32::consts::PI,
            0.,
        );
        path.close();
        path
    }
}

fn setup(mut _commands: Commands, _font: Res<FontAssets>, _gate: Res<GateAssets>) {}

// ############################# Connector ##############################################





// ############################# Connection Line ########################################



// ############################# User Interface #########################################
#[derive(AssetCollection)]
pub struct GateAssets {
    #[asset(path = "gates/not.png")]
    pub not: Handle<Image>,

    #[asset(path = "gates/NOT_BS.png")]
    pub not_bs: Handle<Image>,

    #[asset(path = "gates/and.png")]
    pub and: Handle<Image>,

    #[asset(path = "gates/AND_BS.png")]
    pub and_bs: Handle<Image>,

    #[asset(path = "gates/nand.png")]
    pub nand: Handle<Image>,

    #[asset(path = "gates/NAND_BS.png")]
    pub nand_bs: Handle<Image>,

    #[asset(path = "gates/or.png")]
    pub or: Handle<Image>,

    #[asset(path = "gates/OR_BS.png")]
    pub or_bs: Handle<Image>,

    #[asset(path = "gates/nor.png")]
    pub nor: Handle<Image>,

    #[asset(path = "gates/NOR_BS.png")]
    pub nor_bs: Handle<Image>,

    #[asset(path = "gates/xor.png")]
    pub xor: Handle<Image>,

    #[asset(path = "gates/XOR_BS.png")]
    pub xor_bs: Handle<Image>,

    #[asset(path = "gates/back.png")]
    pub back: Handle<Image>,

    #[asset(path = "gates/close.png")]
    pub close: Handle<Image>,

    #[asset(path = "gates/circuit.png")]
    pub circuit: Handle<Image>,

    #[asset(path = "gates/in.png")]
    pub inputs: Handle<Image>,

    #[asset(path = "gates/out.png")]
    pub outputs: Handle<Image>,

    #[asset(path = "gates/high.png")]
    pub high: Handle<Image>,

    #[asset(path = "gates/low.png")]
    pub low: Handle<Image>,

    #[asset(path = "gates/toggle.png")]
    pub toggle: Handle<Image>,

    #[asset(path = "gates/bulb.png")]
    pub bulb: Handle<Image>,

    #[asset(path = "gates/CLK.png")]
    pub clk: Handle<Image>,
}

#[derive(Debug, Eq, PartialEq)]
enum MenuStates {
    Idle,
    Select,
    LogicGates,
    Inputs,
    Outputs,
}

struct MenuState(MenuStates);

fn open_radial_menu_system(
    mb: Res<Input<MouseButton>>,
    mw: Res<MouseWorldPos>,
    assets: Res<GateAssets>,
    mut ms: ResMut<MenuState>,
    mut ev_open: EventWriter<OpenMenuEvent>,
) {
    if mb.just_pressed(MouseButton::Right) && ms.0 == MenuStates::Idle {
        ev_open.send(OpenMenuEvent {
            position: Vec2::new(mw.x, mw.y),
            mouse_button: MouseButton::Left,
            items: vec![
                (
                    assets.close.clone(),
                    "close".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.circuit.clone(),
                    "Show Logic\nGates".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.inputs.clone(),
                    "Show Input\nControls".to_string(),
                    Vec2::new(80., 80.),
                ),
                (
                    assets.outputs.clone(),
                    "Show Output\nControls".to_string(),
                    Vec2::new(80., 80.),
                ),
            ],
        });

        ms.0 = MenuStates::Select;
    }
}

fn update_radial_menu_system(
    mw: Res<MouseWorldPos>,
    mut ev_update: EventWriter<UpdateCursorPositionEvent>,
) {
    ev_update.send(UpdateCursorPositionEvent(mw.0));
}

fn handle_radial_menu_event_system(
    mut commands: Commands,
    mut ev_radial: EventReader<PropagateSelectionEvent>,
    mut ev_open: EventWriter<OpenMenuEvent>,
    font: Res<FontAssets>,
    assets: Res<GateAssets>,
    mut ms: ResMut<MenuState>,
) {
    for ev in ev_radial.iter() {
        match ms.0 {
            MenuStates::Select => match ev.id {
                1 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.and_bs.clone(),
                                "AND gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.nand_bs.clone(),
                                "NAND gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.or_bs.clone(),
                                "OR gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.nor_bs.clone(),
                                "NOR gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.not_bs.clone(),
                                "NOT gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.xor_bs.clone(),
                                "XOR gate".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::LogicGates;
                }
                2 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.high.clone(),
                                "HIGH const".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.low.clone(),
                                "LOW const".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.toggle.clone(),
                                "Toggle Switch".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.clk.clone(),
                                "Clock".to_string(),
                                Vec2::new(70., 70.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::Inputs;
                }
                3 => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (assets.back.clone(), "back".to_string(), Vec2::new(80., 80.)),
                            (
                                assets.bulb.clone(),
                                "Light Bulb".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });
                    ms.0 = MenuStates::Outputs;
                }
                _ => {
                    ms.0 = MenuStates::Idle;
                }
            },
            MenuStates::LogicGates => match ev.id {
                1 => {
                    Gate::and_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    Gate::nand_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    Gate::or_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                4 => {
                    Gate::nor_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                5 => {
                    Gate::not_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                6 => {
                    Gate::xor_gate_bs(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Inputs => match ev.id {
                1 => {
                    Gate::high_const(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    Gate::low_const(&mut commands, ev.position, font.main.clone());
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    ToggleSwitch::new(&mut commands, Vec2::new(ev.position.x, ev.position.y));
                    ms.0 = MenuStates::Idle;
                }
                4 => {
                    Clk::spawn(&mut commands, Vec2::new(ev.position.x, ev.position.y));
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Outputs => match ev.id {
                1 => {
                    LightBulb::spawn(&mut commands, Vec2::new(ev.position.x, ev.position.y));
                    ms.0 = MenuStates::Idle;
                }
                _ => {
                    ev_open.send(OpenMenuEvent {
                        position: ev.position,
                        mouse_button: MouseButton::Left,
                        items: vec![
                            (
                                assets.close.clone(),
                                "close".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.circuit.clone(),
                                "Show Logic\nGates".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.inputs.clone(),
                                "Show Input\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                            (
                                assets.outputs.clone(),
                                "Show Output\nControls".to_string(),
                                Vec2::new(80., 80.),
                            ),
                        ],
                    });

                    ms.0 = MenuStates::Select;
                }
            },
            MenuStates::Idle => {} // This should never happen
        }
    }
}

struct ChangeInput {
    gate: Entity,
    to: u32,
}

fn ui_node_info_system(
    egui_context: ResMut<EguiContext>,
    mut q_gate: Query<(Entity, &Name, Option<&Gate>, Option<&mut Clk>), With<Selected>>,
    mut ev_change: EventWriter<ChangeInput>,
    mut mb: ResMut<Input<MouseButton>>,
) {
    if let Ok((entity, name, gate, mut clk)) = q_gate.get_single_mut() {
        if let Some(response) = egui::Window::new(&name.0)
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-5., -5.))
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

    egui::TopBottomPanel::top("side").show(egui_context.ctx(), |ui| {
        ui.label("Hello World");
    });
}

fn change_input_system(
    mut commands: Commands,
    mut ev_connect: EventReader<ChangeInput>,
    mut ev_disconnect: EventWriter<DisconnectEvent>,
    mut q_gate: Query<(
        Entity,
        &mut Gate,
        &mut Inputs,
        &mut Interactable,
        &GlobalTransform,
        Option<&BritishStandard>,
    )>,
    q_connectors: Query<&Children>,
    mut q_connector: Query<(&mut Connector, &mut Transform, &Connections)>,
) {
    for ev in ev_connect.iter() {
        if let Ok((gent, mut gate, mut inputs, mut interact, transform, bs)) = q_gate.get_mut(ev.gate) {
            // Update input count
            gate.inputs = ev.to;

            let translation = transform.translation;

            // Update input vector
            inputs.resize(gate.inputs as usize, State::None);

            // If the logic component is BS it has a box as body.
            // We are going to resize it in relation to the number
            // of input connectors.
            let dists = if let Some(_) = bs {
                let dists = get_distances(
                    gate.inputs as f32,
                    gate.outputs as f32,
                    GATE_WIDTH,
                    GATE_HEIGHT,
                );

                // Update bounding box
                interact.update_size(0., 0., dists.width, dists.height);

                let gate = Gate::body(
                    Vec3::new(
                        translation.x,
                        translation.y,
                        translation.z,
                    ),
                    Vec2::new(
                        dists.width,
                        dists.height,
                    )
                );

                // Update body
                commands.entity(ev.gate).remove_bundle::<ShapeBundle>();
                commands.entity(ev.gate).insert_bundle(gate);

                dists
            } else {
                get_distances(
                    gate.inputs as f32,
                    gate.outputs as f32,
                    GATE_SIZE,
                    GATE_SIZE,
                )
            };

            // Update connectors attached to this gate
            let mut max = 0;
            if let Ok(connectors) = q_connectors.get(ev.gate) {
                for connector in connectors.iter() {
                    if let Ok((conn, mut trans, conns)) = q_connector.get_mut(*connector) {
                        if conn.ctype == ConnectorType::In {
                            if conn.index < ev.to as usize {
                                trans.translation = Vec3::new(
                                    -GATE_SIZE * 0.6,
                                    dists.offset + (conn.index + 1) as f32 * dists.in_step,
                                    0.,
                                );
                                if max < conn.index {
                                    max = conn.index;
                                }
                            } else {
                                // Remove connector if neccessary. This includes logical
                                // links between gates and connection line entities.
                                for &c in conns.iter() {
                                    ev_disconnect.send(DisconnectEvent {
                                        connection: c,
                                        in_parent: Some(gent),
                                    });
                                }

                                // Finally remove entity.
                                commands.entity(*connector).despawn_recursive();
                            }
                        }
                    }
                }
            }

            // If the expected amount of connectors exceeds the factual
            // amount, add new connectors to the gate.
            let mut entvec: Vec<Entity> = Vec::new();
            for i in (max + 2)..=ev.to as usize {
                entvec.push(Connector::with_line(
                    &mut commands,
                    Vec3::new(
                        -GATE_SIZE * 0.6,
                        dists.offset + i as f32 * dists.in_step,
                        translation.z,
                    ),
                    GATE_SIZE * 0.1,
                    ConnectorType::In,
                    (i - 1),
                ));
            }
            if !entvec.is_empty() {
                commands.entity(ev.gate).push_children(&entvec);
            }
        }
    }
}
