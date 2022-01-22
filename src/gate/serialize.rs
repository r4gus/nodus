use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    gate::{
        core::{*, Name},
        graphics::{
            gate::*,
            toggle_switch::*,
            light_bulb::*,
        },
    },
    FontAssets,
};
use ron::ser::{to_string_pretty, PrettyConfig};
use chrono::prelude::*;
use std::fs::{self, DirEntry};
use std::collections::HashMap;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct NodusComponent {
    id: Entity,
    name: String,
    inputs: Option<usize>,
    outputs: Option<usize>,
    targets: Option<Targets>,
    position: Vec2,
    ntype: NodeType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodusSave {
    time: DateTime<chrono::Local>,
    application: String,
    entities: Vec<NodusComponent>,
}

pub struct SaveEvent(pub String);
pub struct LoadEvent(pub String);

pub fn save_event_system(
    q_node: Query<(Entity, &Name, Option<&Inputs>, Option<&Outputs>, Option<&Targets>, &Transform, &NodeType)>,
    mut ev_save: EventReader<SaveEvent>,
) {
    for ev in ev_save.iter() {
        let mut save = Vec::new();

        for (e, n, i, o, t, tr, nt) in q_node.iter() {
            let i = if let Some(i) = i { Some(i.len()) } else { None };
            let o = if let Some(o) = o { Some(o.len()) } else { None };
            let t = if let Some(t) = t { Some(t.clone()) } else { None };

            let nc = NodusComponent {
                id: e,
                name: n.0.to_string(),
                inputs: i,
                outputs: o,
                targets: t,
                position: Vec2::new(tr.translation.x, tr.translation.y),
                ntype: nt.clone(),
            };
            save.push(nc);
        }

        let nsave = NodusSave {
            time: chrono::Local::now(),
            application: String::from("Nodus - A logic gate simulator"),
            entities: save,
        };
        
        let pretty = PrettyConfig::new()
            .depth_limit(5)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        //eprintln!("RON: {}", to_string_pretty(&nsave, pretty).unwrap());
        eprintln!("{}", &ev.0);
        if let Ok(res) = fs::write(&ev.0, &to_string_pretty(&nsave, pretty).unwrap()) {
            eprintln!("success");
        } else {
            eprintln!("failure");
        }
    }
}

pub fn load_event_system(
    mut commands: Commands,
    mut ev_load: EventReader<LoadEvent>,
    font: Res<FontAssets>,
) {
    for ev in ev_load.iter() {
        if let Ok(loaded_save) = fs::read_to_string(&ev.0) {
            let save: Result<NodusSave, _> = ron::from_str(&loaded_save);
            let mut id_map: HashMap<Entity, Entity> = HashMap::new();
            
            if let Ok(save) = save {
                for e in &save.entities {
                    match e.ntype {
                        NodeType::And => {
                            let id = Gate::and_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::Nand => {
                            let id = Gate::nand_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::Or => {
                            let id = Gate::or_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::Nor => {
                            let id = Gate::nor_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::Xor => {
                            let id = Gate::xor_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::Xnor => {

                        },
                        NodeType::Not => {
                            let id = Gate::not_gate_bs_(
                                &mut commands, 
                                e.position, 
                                e.inputs.unwrap(), 
                                e.outputs.unwrap(), 
                                font.main.clone()
                            );
                            id_map.insert(e.id, id);
                        },
                        NodeType::HighConst => {
                            let id = Gate::high_const(&mut commands, e.position, font.main.clone());
                            id_map.insert(e.id, id);
                        },
                        NodeType::LowConst => {
                            let id = Gate::low_const(&mut commands, e.position, font.main.clone());
                            id_map.insert(e.id, id);
                        },
                        NodeType::ToggleSwitch => {
                            let id = ToggleSwitch::new(&mut commands, e.position);
                            id_map.insert(e.id, id);
                        },
                        NodeType::Clock => {

                        },
                        NodeType::LightBulb => {
                            let id = LightBulb::spawn(&mut commands, e.position);
                            id_map.insert(e.id, id);
                        },
                    }
                }

                eprintln!("file loaded and parsed");
            } else {
                eprintln!("unable to parse file");
            }
        } else {
            eprintln!("unable to load file");
        }
    }
}
