use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;

use crate::{FontAssets};

pub struct RadialMenu;

impl Plugin for RadialMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(MenuSettings::default())
            .add_event::<OpenMenuEvent>()
            .add_event::<UpdateCursorPositionEvent>()
            .add_system_set(
            SystemSet::new()
                .label("RadialMenu")
                .with_system(open_menu_system.system())
                .with_system(execute_and_close_system.system())
                .with_system(update_system.system())
        );
    }
}

struct MenuSettings {
    outer_radius: f32,
    inner_radius: f32,
    main_color: Color,
    border_color: Color,
    select_color: Color,
}

impl MenuSettings {
    pub fn default() -> Self {
        MenuSettings {
            outer_radius: 220.0,
            inner_radius: 120.0,
            main_color: Color::rgba(1., 1., 1., 0.85),
            border_color: Color::rgba(1., 1., 1., 0.85),
            select_color: Color::TEAL,
        }
    }
}

pub struct Menu {
    position: Vec2,
    mouse_button: MouseButton,
    items: usize,
    selected: Entity,
}

struct MenuItem {
    id: usize,
    symbol: String,
    text: String,
    range: Vec2,
}

pub struct OpenMenuEvent {
    pub position: Vec2,
    pub mouse_button: MouseButton,
    pub items: Vec<(String, String)>,
}

fn create_menu_item_path(
    radians_distance: f32, 
    inner_radius: f32, 
    outer_radius: f32, 
    item_nr: usize
) -> PathBuilder {
    let factor = inner_radius / outer_radius;

    let inner_point = Vec2::new(
        (radians_distance * (item_nr + 1) as f32).cos() * inner_radius, 
        (radians_distance * (item_nr + 1) as f32).sin() * inner_radius
    );
    let outer_point = Vec2::new(
        (radians_distance * item_nr as f32).cos() * outer_radius, 
        (radians_distance * item_nr as f32).sin() * outer_radius
    );

    let mut arc_path = PathBuilder::new();
    arc_path.move_to(inner_point);
    arc_path.arc(
        Vec2::new(0.0, 0.0), 
        Vec2::new(
            inner_radius, 
            inner_radius
        ), 
        -radians_distance, 
        0.0
    );
    arc_path.line_to(outer_point);
    arc_path.arc(
        Vec2::new(0.0, 0.0),
        Vec2::new(
            outer_radius, 
            outer_radius
        ),
        radians_distance,
        0.,
    );
    arc_path.close();
    arc_path
}

fn create_menu_item_visual(
    radians_distance: f32, 
    inner_radius: f32, 
    outer_radius: f32, 
    item_nr: usize,
    color: Color,

) -> ShapeBundle {
    GeometryBuilder::build_as(
        &create_menu_item_path(radians_distance, inner_radius, outer_radius, item_nr).build(),
        ShapeColors::outlined(color, color),
        /*
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(6.0),
        },
        */
        DrawMode::Fill(Default::default()),
        Transform::from_xyz(0., 0., 0.),
    )
}

fn open_menu_system(
    mut commands: Commands,
    mut ev_open: EventReader<OpenMenuEvent>,
    settings: Res<MenuSettings>,
    q_menu: Query<&Menu>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(_) = q_menu.single() { return; }

    for ev in ev_open.iter() {
        let radians_distance = (std::f32::consts::PI * 2.) / ev.items.len() as f32;

        // Create the required menu items.
        let mut evec = Vec::new();
        for i in 0..ev.items.len() {
            let center = radians_distance * i as f32 + radians_distance * 0.5;
            let factor = settings.inner_radius + 
                (settings.outer_radius - settings.inner_radius) * 0.5;

            evec.push(commands.spawn_bundle(
                    create_menu_item_visual(
                        radians_distance,
                        settings.inner_radius, 
                        settings.outer_radius, 
                        i,
                        if i == 0 { settings.select_color } else { settings.main_color },
                    )
                )
                .insert(MenuItem {
                    id: i,
                    symbol: ev.items[i].0.clone(),
                    text: ev.items[i].1.clone(),
                    range: Vec2::new(
                        radians_distance * i as f32, 
                        radians_distance * (i + 1) as f32
                    ),
                }).with_children(|parent| {
                    parent.spawn_bundle(Text2dBundle {
                        text: Text::with_section(
                            &ev.items[i].0,
                            TextStyle {
                                font: asset_server.load("fonts/hack.bold.ttf"),
                                font_size: 60.0,
                                color: Color::BLACK,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        transform: Transform::from_xyz(
                            center.cos() * factor, 
                            center.sin() * factor - 30.,
                            1.
                        ),
                        ..Default::default()
                    });
                }).id());
        }

        commands.spawn()
            .insert(GlobalTransform::from_xyz(ev.position.x, ev.position.y, 100.))
            .insert(Transform::from_xyz(ev.position.x, ev.position.y, 100.))
            .insert(Menu {
                position: ev.position,
                mouse_button: ev.mouse_button,
                items: ev.items.len(),
                selected: evec[0],
            })
            .push_children(&evec)
            .with_children(|parent| {
                // TODO: Insert circle where additional information is displayed.
                /*
                parent.spawn_bundle(
                    create_menu_item_visual(
                        radians_distance,
                        settings.inner_radius, 
                        settings.outer_radius, 
                        i,
                        if i == 0 { settings.select_color } else { settings.main_color },
                    )
                );
                */
            });
    }
}

fn execute_and_close_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    q_menu: Query<(Entity, &Menu), ()>
) {
    // There should only be one radial menu open at
    // any given moment.
    if let Ok((entity, menu)) = q_menu.single() {

        if mb.just_pressed(menu.mouse_button) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct UpdateCursorPositionEvent(pub Vec2);

fn update_system(
    mut commands: Commands,
    mut ev_open: EventReader<UpdateCursorPositionEvent>,
    settings: Res<MenuSettings>,
    mut q_menu: Query<(&Children, &mut Menu), ()>,
    q_item: Query<(Entity, &MenuItem)>,
) {
    if let Ok((children, mut menu)) = q_menu.single_mut() {
        for ev in ev_open.iter() {
            let distance = ev.0 - menu.position;
            let mut rad = distance.y.atan2(distance.x);
            if rad < 0.0 { rad = rad + std::f32::consts::PI * 2.; }
            eprintln!("{}", rad);

            for &child in children.iter() {
                if let Ok((entity, item)) = q_item.get(child) {
                    if rad >= item.range.x && rad < item.range.y {
                        eprintln!("{}", item.id);

                        if entity != menu.selected {
                            let radians_distance = (std::f32::consts::PI * 2.) / menu.items as f32;

                            commands.entity(entity).remove_bundle::<ShapeBundle>();
                            commands.entity(entity).insert_bundle(
                                create_menu_item_visual(
                                    radians_distance,
                                    settings.inner_radius, 
                                    settings.outer_radius, 
                                    item.id,
                                    settings.select_color,
                                )
                            );

                            if let Ok((_, item)) = q_item.get(menu.selected) {
                                commands.entity(menu.selected).remove_bundle::<ShapeBundle>();
                                commands.entity(menu.selected).insert_bundle(
                                    create_menu_item_visual(
                                        radians_distance,
                                        settings.inner_radius, 
                                        settings.outer_radius, 
                                        item.id,
                                        settings.main_color,
                                    )
                                );
                            }

                            menu.selected = entity;
                        }

                        break;
                    }
                }
            }
        }
    }
}

/*
struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    let elements = 3;
    let radius = 180.;
    let radians_distance = (std::f32::consts::PI * 2.) / elements as f32;
    let x = 600.;
    let y = 600.;

    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    let parent = commands
        .spawn()
        .insert(Menu).id();

    let mut entvec: Vec<Entity> = Vec::new();
    for i in 0..elements as usize {
        entvec.push(commands.spawn_bundle(Text2dBundle {
            text: Text::with_section(
                &format!("Button {}", i),
                TextStyle {
                    font: asset_server.load("fonts/hack.bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            //transform: Transform::from_xyz(x * i as f32 + (radians_distance * i as f32).cos() * radius, y * i as f32 + (radians_distance * i as f32).sin() * radius, 100.),
            ..Default::default()
        }).id());
    }
    commands.entity(parent).push_children(&entvec);
}
*/
