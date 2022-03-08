use crate::radial_menu::{OpenMenuEvent, PropagateSelectionEvent, UpdateCursorPositionEvent};
use crate::{GameState};
use bevy::prelude::*;
use crate::gate::systems::InsertGateEvent;
use bevy_asset_loader::AssetCollection;

use nodus::world2d::camera2d::MouseWorldPos;

pub struct GateMenuPlugin;

impl Plugin for GateMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuState(MenuStates::Idle))
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(open_radial_menu_system)
                    .with_system(handle_radial_menu_event_system.label("handle_rad_event"))
                    .with_system(update_radial_menu_system),
            );
    }
}

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

    #[asset(path = "gates/sevenseg.png")]
    pub seg: Handle<Image>,
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
    mut ev_radial: EventReader<PropagateSelectionEvent>,
    mut ev_open: EventWriter<OpenMenuEvent>,
    mut ev_insert: EventWriter<InsertGateEvent>,
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
                            (assets.clk.clone(), "Clock".to_string(), Vec2::new(70., 70.)),
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
                            (
                                assets.seg.clone(),
                                "7-Segment Display".to_string(),
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
                    ev_insert.send(InsertGateEvent::and(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    ev_insert.send(InsertGateEvent::nand(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    ev_insert.send(InsertGateEvent::or(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                4 => {
                    ev_insert.send(InsertGateEvent::nor(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                5 => {
                    ev_insert.send(InsertGateEvent::not(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                6 => {
                    ev_insert.send(InsertGateEvent::xor(ev.position));
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
                    ev_insert.send(InsertGateEvent::high(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                2 => {
                    ev_insert.send(InsertGateEvent::low(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                3 => {
                    ev_insert.send(InsertGateEvent::toggle(ev.position));
                    ms.0 = MenuStates::Idle;
                }
                4 => {
                    ev_insert.send(InsertGateEvent::clk(ev.position));
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
                    ev_insert.send(InsertGateEvent::light(ev.position));
                    ms.0 = MenuStates::Idle;
                },
                2 => {
                    ev_insert.send(InsertGateEvent::seg(ev.position));
                    ms.0 = MenuStates::Idle;
                },
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
