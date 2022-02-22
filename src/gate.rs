pub mod core;
pub mod file_browser;
pub mod graphics;
pub mod serialize;
pub mod systems;
pub mod ui;
pub mod undo;

use crate::gate::{
    core::*,
    graphics::{
        background::*, clk::*, connection_line::*, connector::*, gate::*, highlight::*,
        light_bulb::*, selector::*, toggle_switch::*,
    },
    serialize::*,
    systems::*,
    ui::*,
    undo::*,
};
use crate::rmenu::*;
use bevy::prelude::*;

use super::GameState;

pub struct LogicComponentSystem;

const NODE_GROUP: u32 = 1;
const CONNECTOR_GROUP: u32 = 2;

impl Plugin for LogicComponentSystem {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectEvent>()
            .add_event::<ChangeInput>()
            .add_event::<DisconnectEvent>()
            .add_event::<SaveEvent>()
            .add_event::<LoadEvent>()
            .add_event::<InsertGateEvent>()
            .add_event::<NewConnectionEstablishedEvent>()
            .add_plugin(GateMenuPlugin)
            .add_plugin(UndoPlugin)
            .insert_resource(LineResource {
                count: 0.,
                timestep: 0.5,
                update: false,
            })
            .insert_resource(GuiMenu {
                option: GuiMenuOptions::None,
                open: false,
            })
            .add_startup_system(update_ui_scale_factor)
            .add_startup_system(load_gui_assets)
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .before("interaction2d")
                    .label("level3_node_set")
                    .with_system(ui_node_info_system)
                    .with_system(ui_top_panel_system)
                    .with_system(ui_scroll_system)
                    .with_system(ui_gui_about)
                    .with_system(ui_reset_input)
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
                    .with_system(insert_gate_system.after("handle_rad_event"))
                    // Draw Line inserts a new bundle into an entity that might has been
                    // deleted by delete_line_system, i.e. we run it before any deletions
                    // to prevent an segfault.
                    .with_system(
                        draw_line_system
                            .system()
                            .label("draw_line")
                            .before("disconnect"),
                    )
                    //.with_system(draw_data_flow.system().after("draw_line"))
                    .with_system(highlight_system.before("disconnect"))
                    .with_system(remove_highlight_system.before("disconnect"))
                    .with_system(change_highlight_system.before("disconnect"))
                    .with_system(light_bulb_system.system().before("disconnect"))
                    .with_system(toggle_switch_system.system().before("disconnect"))
                    .with_system(line_selection_system.system().after("draw_line"))
                    .with_system(draw_background_grid_system)
                    .with_system(clk_system),
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .label("level1_node_set")
                    .after("level2_node_set")
                    .with_system(selector_system)
                    .with_system(save_event_system.before("new_file"))
                    // The link_gates_system requires entities spawned by the
                    // load_event_system. To make sure the entities can be
                    // queried the link_gates_system must always run before the
                    // load_event_system, so one can be sure that all entites
                    // have been inserted into the world.
                    .with_system(link_gates_system.label("link_gates_system"))
                    .with_system(load_event_system.after("link_gates_system"))
                    .with_system(shortcut_system)
                    .with_system(update_lock),
            )
            .add_system_set(
                SystemSet::on_enter(GameState::InGame), //.with_system(setup.system())
            );

        info!("NodePlugin loaded");
    }
}
