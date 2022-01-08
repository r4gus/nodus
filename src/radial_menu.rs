use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;

pub struct RadialMenu;

impl Plugin for RadialMenu {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MenuSettings::default())
            .add_event::<OpenMenuEvent>()
            .add_event::<UpdateCursorPositionEvent>()
            .add_event::<PropagateSelectionEvent>()
            .add_system_set(
                SystemSet::new()
                    .label("RadialMenu")
                    .with_system(open_menu_system.system())
                    .with_system(execute_and_close_system.system())
                    .with_system(update_system.system()),
            );
    }
}

struct MenuSettings {
    outer_radius: f32,
    inner_radius: f32,
    main_color: Color,
    second_color: Color,
    select_color: Color,
}

impl MenuSettings {
    pub fn default() -> Self {
        MenuSettings {
            outer_radius: 220.0,
            inner_radius: 120.0,
            main_color: Color::rgba(1., 1., 1., 0.85),
            second_color: Color::rgba(0., 0., 0., 0.85),
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
    text: String,
    range: Vec2,
}

struct ItemInfo;

pub struct OpenMenuEvent {
    pub position: Vec2,
    pub mouse_button: MouseButton,
    pub items: Vec<(Handle<ColorMaterial>, String, Vec2)>,
}

fn create_menu_item_path(
    radians_distance: f32,
    inner_radius: f32,
    outer_radius: f32,
    item_nr: usize,
) -> PathBuilder {
    let inner_point = Vec2::new(
        (radians_distance * (item_nr + 1) as f32).cos() * inner_radius,
        (radians_distance * (item_nr + 1) as f32).sin() * inner_radius,
    );
    let outer_point = Vec2::new(
        (radians_distance * item_nr as f32).cos() * outer_radius,
        (radians_distance * item_nr as f32).sin() * outer_radius,
    );

    let mut arc_path = PathBuilder::new();
    arc_path.move_to(inner_point);
    arc_path.arc(
        Vec2::new(0.0, 0.0),
        Vec2::new(inner_radius, inner_radius),
        -radians_distance,
        0.0,
    );
    arc_path.line_to(outer_point);
    arc_path.arc(
        Vec2::new(0.0, 0.0),
        Vec2::new(outer_radius, outer_radius),
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
    if let Ok(_) = q_menu.single() {
        return;
    }

    for ev in ev_open.iter() {
        let radians_distance = (std::f32::consts::PI * 2.) / ev.items.len() as f32;

        // Create the required menu items.
        let mut evec = Vec::new();
        for i in 0..ev.items.len() {
            let center = radians_distance * i as f32 + radians_distance * 0.5;
            let factor =
                settings.inner_radius + (settings.outer_radius - settings.inner_radius) * 0.5;

            evec.push(
                commands
                    .spawn_bundle(create_menu_item_visual(
                        radians_distance,
                        settings.inner_radius,
                        settings.outer_radius,
                        i,
                        if i == 0 {
                            settings.select_color
                        } else {
                            settings.main_color
                        },
                    ))
                    .insert(MenuItem {
                        id: i,
                        text: ev.items[i].1.clone(),
                        range: Vec2::new(
                            radians_distance * i as f32,
                            radians_distance * (i + 1) as f32,
                        ),
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(SpriteBundle {
                            material: ev.items[i].0.clone(),
                            sprite: Sprite::new(ev.items[i].2),
                            transform: Transform::from_xyz(
                                center.cos() * factor,
                                center.sin() * factor,
                                1.,
                            ),
                            ..Default::default()
                        });
                    })
                    .id(),
            );
        }

        commands
            .spawn()
            .insert(GlobalTransform::from_xyz(
                ev.position.x,
                ev.position.y,
                100.,
            ))
            .insert(Transform::from_xyz(ev.position.x, ev.position.y, 100.))
            .insert(Menu {
                position: ev.position,
                mouse_button: ev.mouse_button,
                items: ev.items.len(),
                selected: evec[0],
            })
            .push_children(&evec)
            .with_children(|parent| {
                let inner_circle = GeometryBuilder::build_as(
                    &shapes::Circle {
                        radius: settings.inner_radius * 0.9,
                        center: Vec2::new(0., 0.),
                    },
                    ShapeColors::new(settings.second_color),
                    DrawMode::Fill(Default::default()),
                    Transform::from_xyz(0., 0., 0.),
                );

                parent
                    .spawn_bundle(inner_circle)
                    .insert(ItemInfo)
                    .with_children(|parent| {
                        parent.spawn_bundle(Text2dBundle {
                            text: Text::with_section(
                                &ev.items[0].1,
                                TextStyle {
                                    font: asset_server.load("fonts/hack.bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                                TextAlignment {
                                    horizontal: HorizontalAlign::Center,
                                    ..Default::default()
                                },
                            ),
                            transform: Transform::from_xyz(0., 0., 1.),
                            ..Default::default()
                        });
                    });
            });
    }
}

/// Information about the item selected by the user.
pub struct PropagateSelectionEvent {
    /// Index/ ID of the selected item.
    pub id: usize,
    /// Center of the radial menu.
    pub position: Vec2,
}

fn execute_and_close_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    q_menu: Query<(Entity, &Menu), ()>,
    q_item: Query<&MenuItem>,
    mut ev_propagate: EventWriter<PropagateSelectionEvent>,
) {
    // There should only be one radial menu open at
    // any given moment.
    if let Ok((entity, menu)) = q_menu.single() {
        if mb.just_pressed(menu.mouse_button) {
            if let Ok(item) = q_item.get(menu.selected) {
                eprintln!("sending");
                ev_propagate.send(PropagateSelectionEvent {
                    id: item.id,
                    position: menu.position,
                });
            }

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
    q_item_info: Query<(Entity, &Children), With<ItemInfo>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((children, mut menu)) = q_menu.single_mut() {
        for ev in ev_open.iter() {
            let distance = ev.0 - menu.position;
            let mut rad = distance.y.atan2(distance.x);
            if rad < 0.0 {
                rad = rad + std::f32::consts::PI * 2.;
            }
            //eprintln!("{}", rad);

            for &child in children.iter() {
                if let Ok((entity, item)) = q_item.get(child) {
                    if rad >= item.range.x && rad < item.range.y {
                        //eprintln!("{}", item.id);

                        if entity != menu.selected {
                            let radians_distance = (std::f32::consts::PI * 2.) / menu.items as f32;

                            // Highlight the new selected item.
                            commands.entity(entity).remove_bundle::<ShapeBundle>();
                            commands
                                .entity(entity)
                                .insert_bundle(create_menu_item_visual(
                                    radians_distance,
                                    settings.inner_radius,
                                    settings.outer_radius,
                                    item.id,
                                    settings.select_color,
                                ));

                            // Remove highlighting from old item.
                            if let Ok((_, item)) = q_item.get(menu.selected) {
                                commands
                                    .entity(menu.selected)
                                    .remove_bundle::<ShapeBundle>();
                                commands.entity(menu.selected).insert_bundle(
                                    create_menu_item_visual(
                                        radians_distance,
                                        settings.inner_radius,
                                        settings.outer_radius,
                                        item.id,
                                        settings.main_color,
                                    ),
                                );
                            }

                            // Update info text.
                            if let Ok((entity, children)) = q_item_info.single() {
                                for &child in children.iter() {
                                    commands.entity(child).despawn_recursive();
                                }

                                let id = commands
                                    .spawn_bundle(Text2dBundle {
                                        text: Text::with_section(
                                            &item.text,
                                            TextStyle {
                                                font: asset_server.load("fonts/hack.bold.ttf"),
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                            },
                                            TextAlignment {
                                                horizontal: HorizontalAlign::Center,
                                                ..Default::default()
                                            },
                                        ),
                                        transform: Transform::from_xyz(0., 0., 1.),
                                        ..Default::default()
                                    })
                                    .id();

                                commands.entity(entity).push_children(&[id]);
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
