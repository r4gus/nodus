use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub struct RadialMenu;

impl Plugin for RadialMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(MenuSettings::default())
            .add_event::<OpenMenuEvent>()
            .add_system_set(
            SystemSet::new()
                .label("RadialMenu")
                .with_system(open_menu_system.system())
                .with_system(execute_and_close_system.system())
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
            inner_radius: 90.0,
            main_color: Color::rgba(0., 0., 0., 0.7),
            border_color: Color::BLACK,
            select_color: Color::TEAL,
        }
    }
}

pub struct Menu {
    mouse_button: MouseButton,
}

pub struct OpenMenuEvent {
    pub position: Vec2,
    pub mouse_button: MouseButton,
}

fn open_menu_system(
   mut commands: Commands,
   mut ev_open: EventReader<OpenMenuEvent>,
   settings: Res<MenuSettings>,
) {
    for ev in ev_open.iter() {
        let gap: f32 = 0.1;

        commands.spawn()
            .insert(GlobalTransform::from_xyz(ev.position.x, ev.position.y, 100.))
            .insert(Transform::from_xyz(ev.position.x, ev.position.y, 100.))
            .insert(Menu {
                mouse_button: ev.mouse_button,
            }).with_children(|parent| {
                let radians_distance = (std::f32::consts::PI * 2.) / 3.;
            
                for i in 0..3 {
                    let inner_point = Vec2::new(
                        (radians_distance * (i + 1) as f32).cos() * settings.inner_radius, 
                        (radians_distance * (i + 1) as f32).sin() * settings.inner_radius
                    );
                    let outer_point = Vec2::new(
                        (radians_distance * i as f32).cos() * settings.outer_radius, 
                        (radians_distance * i as f32).sin() * settings.outer_radius
                    );

                    let mut arc_path = PathBuilder::new();
                    arc_path.move_to(inner_point);
                    arc_path.arc(
                        Vec2::new(0.0, 0.0), 
                        Vec2::new(
                            settings.inner_radius, 
                            settings.inner_radius
                        ), 
                        -radians_distance + gap, 
                        0.
                    );
                    arc_path.line_to(outer_point);
                    arc_path.arc(
                        Vec2::new(0.0, 0.0),
                        Vec2::new(settings.outer_radius, settings.outer_radius),
                        radians_distance - (settings.inner_radius / settings.outer_radius * gap),
                        0.,
                    );
                    arc_path.close();

                    let arc = GeometryBuilder::build_as(
                        &arc_path.build(),
                        ShapeColors::outlined(settings.main_color, settings.border_color),
                        DrawMode::Outlined {
                            fill_options: FillOptions::default(),
                            outline_options: StrokeOptions::default().with_line_width(4.0),
                        },
                        Transform::from_xyz(0., 0., 0.),
                    );

                   parent.spawn_bundle(arc);
                }
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

        if mb.just_released(menu.mouse_button) {
            commands.entity(entity).despawn_recursive();
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
