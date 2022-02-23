use crate::{
    gate::{
        core::{Name, State, *},
        file_browser::*,
        graphics::{clk::*, light_bulb::*, toggle_switch::*},
    },
    FontAssets,
};
use bevy::prelude::*;
use chrono::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Version {
    major: u8,
    minor: u8,
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub enum NodeType {
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
    Not,
    HighConst,
    LowConst,
    ToggleSwitch,
    Clock,
    LightBulb,
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub enum NodeState {
    ToggleSwitch(State),
    Clock(f32, f32, State),
    LightBulb(State),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodusComponent {
    pub id: Entity,
    pub name: String,
    pub inputs: Option<usize>,
    pub outputs: Option<usize>,
    pub targets: Option<Targets>,
    pub position: Vec2,
    pub rotation: Option<Quat>,
    pub ntype: NodeType,
    pub state: Option<NodeState>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodusSave {
    time: DateTime<chrono::Local>,
    application: String,
    version: Version,
    entities: Vec<NodusComponent>,
}

pub struct SaveEvent(pub String);
pub struct LoadEvent(pub String);

pub fn save_event_system(
    q_node: Query<(
        Entity,
        &Name,
        Option<&Inputs>,
        Option<&Outputs>,
        Option<&Targets>,
        Option<&Clk>,
        &Transform,
        &NodeType,
    )>,
    mut ev_save: EventReader<SaveEvent>,
    mut curr_open: ResMut<CurrentlyOpen>,
) {
    for ev in ev_save.iter() {
        let mut save = Vec::new();

        for (e, n, ip, op, t, clk, tr, nt) in q_node.iter() {
            let i = if let Some(i) = ip {
                Some(i.len())
            } else {
                None
            };
            let o = if let Some(o) = op {
                Some(o.len())
            } else {
                None
            };
            let t = if let Some(t) = t {
                Some(t.clone())
            } else {
                None
            };

            let state = match &nt {
                NodeType::ToggleSwitch => Some(NodeState::ToggleSwitch(op.unwrap()[0])),
                NodeType::Clock => {
                    let clk = clk.unwrap();
                    Some(NodeState::Clock(clk.0, clk.1, op.unwrap()[0]))
                }
                NodeType::LightBulb => Some(NodeState::LightBulb(ip.unwrap()[0])),
                _ => None,
            };

            let nc = NodusComponent {
                id: e,
                name: n.0.to_string(),
                inputs: i,
                outputs: o,
                targets: t,
                position: Vec2::new(tr.translation.x, tr.translation.y),
                rotation: Some(tr.rotation),
                ntype: nt.clone(),
                state: state,
            };
            save.push(nc);
        }

        let nsave = NodusSave {
            time: chrono::Local::now(),
            application: String::from("Nodus - A logic gate simulator"),
            version: Version { major: 0, minor: 1 },
            entities: save,
        };

        let pretty = PrettyConfig::new()
            .depth_limit(5)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        //eprintln!("RON: {}", to_string_pretty(&nsave, pretty).unwrap());
        eprintln!("{}", &ev.0);
        if let Ok(_res) = fs::write(&ev.0, &to_string_pretty(&nsave, pretty).unwrap()) {
            curr_open.path = Some(ev.0.clone());
            eprintln!("success");
        } else {
            eprintln!("failure");
        }
    }
}

#[derive(Component)]
pub struct LoadMapper {
    map: HashMap<Entity, Entity>,
    save: NodusSave,
}

pub fn link_gates_system(
    mut commands: Commands,
    mut cev: EventWriter<ConnectEvent>,
    q_children: Query<&Children>,
    q_conn: Query<(Entity, &Connector)>,
    q_map: Query<(Entity, &LoadMapper)>,
) {
    if let Ok((e, map)) = q_map.get_single() {
        for e in &map.save.entities {
            if let Some(targets) = &e.targets {
                // Iterate over the slot of each output connector.
                for i in 0..targets.len() {
                    // Get the associated output connector with index;
                    let mut out_id: Option<Entity> = None;
                    if let Ok(out_children) = q_children.get(map.map[&e.id]) {
                        for &child in out_children.iter() {
                            if let Ok((id, conn)) = q_conn.get(child) {
                                if conn.index == i && conn.ctype == ConnectorType::Out {
                                    out_id = Some(id);
                                    break;
                                }
                            }
                        }
                    }
                    if out_id == None {
                        break;
                    }

                    for (gate, tidx) in targets[i].iter() {
                        if let Ok(in_children) = q_children.get(map.map[&gate]) {
                            for &child in in_children.iter() {
                                if let Ok((id, conn)) = q_conn.get(child) {
                                    for &j in tidx.iter() {
                                        if conn.index == j && conn.ctype == ConnectorType::In {
                                            cev.send(ConnectEvent {
                                                output: out_id.unwrap(),
                                                output_index: i,
                                                input: id,
                                                input_index: j,
                                                signal_success: false,
                                            });
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        commands.entity(e).despawn_recursive();
    }
}

pub fn load_event_system(
    mut commands: Commands,
    mut ev_load: EventReader<LoadEvent>,
    font: Res<FontAssets>,
    mut curr_open: ResMut<CurrentlyOpen>,
    q_all: Query<Entity, Or<(With<NodeType>, With<ConnectionLine>)>>,
) {
    for ev in ev_load.iter() {
        if let Ok(loaded_save) = fs::read_to_string(&ev.0) {
            // Remove all entities currently in the world before inserting
            // the entities from the file.
            for e in q_all.iter() {
                commands.entity(e).despawn_recursive();
            }

            let save: Result<NodusSave, _> = ron::from_str(&loaded_save);
            let mut id_map: HashMap<Entity, Entity> = HashMap::new();

            if let Ok(save) = save {
                for e in &save.entities {
                    let id = match e.ntype {
                        NodeType::And => {
                            Some(Gate::and_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::Nand => {
                            Some(Gate::nand_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::Or => {
                            Some(Gate::or_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::Nor => {
                            Some(Gate::nor_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::Xor => {
                            Some(Gate::xor_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::Xnor => { None }
                        NodeType::Not => {
                            Some(Gate::not_gate_bs_(
                                &mut commands,
                                e.position,
                                e.inputs.unwrap(),
                                e.outputs.unwrap(),
                                font.main.clone(),
                            ))
                        }
                        NodeType::HighConst => {
                            Some(Gate::high_const(&mut commands, e.position, font.main.clone()))
                        }
                        NodeType::LowConst => {
                            Some(Gate::low_const(&mut commands, e.position, font.main.clone()))
                        }
                        NodeType::ToggleSwitch => {
                            if let Some(NodeState::ToggleSwitch(state)) = e.state {
                                Some(ToggleSwitch::new(&mut commands, e.position, state))
                            } else { None }
                        }
                        NodeType::Clock => {
                            if let Some(NodeState::Clock(x1, x2, x3)) = e.state {
                                Some(Clk::spawn(&mut commands, e.position, x1, x2, x3))
                            } else { None }
                        }
                        NodeType::LightBulb => {
                            if let Some(NodeState::LightBulb(state)) = e.state {
                                Some(LightBulb::spawn(&mut commands, e.position, state))
                            } else { None }
                        }
                    };

                    if let Some(id) = id {
                        id_map.insert(e.id, id);
                    }
                }

                // The different logical components must be connected to each other. This
                // is done in another system that always runs before this one to give the
                // ecs enough time to insert the spawned entities above into the world.
                commands.spawn().insert(LoadMapper {
                    map: id_map,
                    save: save,
                });

                // Remember the path of the file. This allows to save the
                // file without specifying the path all the time.
                curr_open.path = Some(ev.0.clone());

                eprintln!("file loaded and parsed");
            } else {
                eprintln!("unable to parse file");
            }
        } else {
            eprintln!("unable to load file");
        }
    }
}
