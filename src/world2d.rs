use crate::world2d::camera2d::Camera2DPlugin;
use crate::world2d::interaction2d::Interaction2DPlugin;
use bevy::prelude::*;

/// Ugly solution for ignoring input.
pub struct Lock(pub bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionMode {
    Select,
    Pan,
}

pub struct NodusWorld2DPlugin;

impl Plugin for NodusWorld2DPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InteractionMode::Select)
            .insert_resource(Lock(false))
            .add_plugin(Camera2DPlugin)
            .add_plugin(Interaction2DPlugin);
    }
}

/*
fn update_cursor_system(
    mut mode: ResMut<InteractionMode>,
    mut wnds: ResMut<Windows>,
) {
    if mode.is_changed() {
        eprintln!("change");
        let window = wnds.get_primary_mut().unwrap();

        match *mode {
            InteractionMode::Select => { window.set_cursor_icon(CursorIcon::Hand); },
            InteractionMode::Pan => { window.set_cursor_icon(CursorIcon::Hand); },
        }
        window.set_cursor_icon(CursorIcon::Hand);
        eprintln!("{:?}", window.cursor_icon());
    }
}
*/

pub mod camera2d {
    use super::{InteractionMode, Lock};
    use bevy::input::mouse::{MouseMotion, MouseWheel};
    use bevy::prelude::*;
    use core::ops::{Deref, DerefMut};

    pub struct Camera2DPlugin;

    impl Plugin for Camera2DPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(MouseWorldPos(Vec2::new(0., 0.)))
                .add_startup_system(setup.system())
                .add_system(cursor_system.system())
                .add_system(pan_zoom_camera_system.system());
        }
    }

    /// Used to help identify the main camera.
    #[derive(Component)]
    pub struct MainCamera;

    /// Position resource of the mouse cursor within the 2d world.
    pub struct MouseWorldPos(pub Vec2);

    impl Deref for MouseWorldPos {
        type Target = Vec2;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for MouseWorldPos {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    pub fn setup(mut commands: Commands) {
        // Spawn main camera with the orthographic projection
        // camera bundle.
        commands
            .spawn_bundle(OrthographicCameraBundle::new_2d())
            .insert(MainCamera);
    }

    /// Calculate the cursor position within a 2d world.
    /// This updates the MouseWorldPos resource.
    pub fn cursor_system(
        // need to get window dimensions.
        wnds: Res<Windows>,
        // need to update the mouse world position.
        mut mw: ResMut<MouseWorldPos>,
        // query to get camera transform.
        q_camera: Query<&Transform, With<MainCamera>>,
    ) {
        // get the primary window.
        let wnd = wnds.get_primary().unwrap();

        // check if the cursor is in the primary window.
        if let Some(pos) = wnd.cursor_position() {
            // get the size of the window.
            let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // the default orthographic projection is in pixels from the center,
            // so we need to undo the translation.
            let p = pos - size / 2.;

            // assuming there is exacly one main camera entity, so this is ok.
            let camera_transform = q_camera.single();

            // apply the camera transform.
            let pos_wld = camera_transform.compute_matrix() * p.extend(0.).extend(1.);
            //eprintln!("{}:{}", pos_wld.x, pos_wld.y);
            mw.x = pos_wld.x;
            mw.y = pos_wld.y;
        }
    }

    pub fn pan_zoom_camera_system(
        mut ev_motion: EventReader<MouseMotion>,
        mut ev_scroll: EventReader<MouseWheel>,
        input_mouse: Res<Input<MouseButton>>,
        mut q_camera: Query<&mut Transform, With<MainCamera>>,
        time: Res<Time>,
        mode: Res<InteractionMode>,
        lock: Res<Lock>,
    ) {
        if lock.0 {
            return;
        }

        let mut pan = Vec2::ZERO;
        let mut scroll = 0.0;

        if *mode == InteractionMode::Pan && input_mouse.pressed(MouseButton::Left) {
            for ev in ev_motion.iter() {
                pan += ev.delta;
            }
        }

        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }

        // assuming there is exacly one main camera entity, so this is ok.
        if let Ok(mut transform) = q_camera.get_single_mut() {
            if pan.length_squared() > 0.0 {
                let scale = transform.scale.x;
                transform.translation.x -= pan.x * scale;
                transform.translation.y += pan.y * scale;
            } else if scroll.abs() > 0.0 {
                let scale = (transform.scale.x - scroll * time.delta_seconds()).clamp(1.0, 5.0);
                transform.scale = Vec3::new(scale, scale, scale);
            }
        }
    }

    pub fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
        let window = windows.get_primary().unwrap();
        let window = Vec2::new(window.width() as f32, window.height() as f32);
        window
    }
}

pub mod interaction2d {
    use super::camera2d::MouseWorldPos;
    use super::{InteractionMode, Lock};
    use bevy::prelude::*;

    pub struct Interaction2DPlugin;

    impl Plugin for Interaction2DPlugin {
        fn build(&self, app: &mut App) {
            app.add_system_set(
                SystemSet::new()
                    .label("interaction2d")
                    .with_system(interaction_system.system().label("interaction"))
                    .with_system(selection_system.system().after("interaction"))
                    .with_system(drag_system.system()),
            );
        }
    }

    /// Marker component for selected entities.
    #[derive(Debug, Component)]
    pub struct Selected;

    /// Component that marks an entity as selectable.
    #[derive(Component)]
    pub struct Selectable;

    /// Component that marks an entity interactable.
    #[derive(Component)]
    pub struct Interactable {
        /// The bounding box defines the area where the mouse
        /// can interact with the entity.
        bounding_box: (Vec2, Vec2),
        /// The group this interactable entity belongs to. This
        /// can be a item, enemy, ... or just use one group for
        /// everything.
        group: u32,
    }

    impl Interactable {
        /// Create a new interactable component.
        ///
        /// # Arguments
        ///
        /// * `position` - The position of the bounding box within the world.
        /// * `dimensions` - The width and the height of the bounding box.
        /// * `group` - The group the selectable entity belongs to.
        ///
        /// The `position` marks the center of the bounding box.
        pub fn new(position: Vec2, dimensions: Vec2, group: u32) -> Self {
            Self {
                bounding_box: (
                    Vec2::new(
                        position.x - dimensions.x / 2.,
                        position.y - dimensions.y / 2.,
                    ),
                    dimensions,
                ),
                group,
            }
        }

        pub fn update_size(&mut self, x: f32, y: f32, width: f32, height: f32) {
            self.bounding_box.0 = Vec2::new(x - width / 2., y - height / 2.);
            self.bounding_box.1 = Vec2::new(width, height);
        }

        /// Update the position of the bounding box within the world.
        pub fn update_pos(&mut self, x: f32, y: f32) {
            self.bounding_box.0.x = x;
            self.bounding_box.0.y = y;
        }
    }

    /// Marker component to indicate that the mouse
    /// currently hovers over the given entity.
    #[derive(Component)]
    pub struct Hover;

    /// Component that marks an entity as draggable.
    #[derive(Component)]
    pub struct Draggable {
        /// The drag system should automatically update
        /// the entities transformation while being dragged
        /// [y/ n].
        pub update: bool,
    }

    /// Marker component to indicate that the
    /// given entity is currently dragged.
    #[derive(Component)]
    pub struct Drag {
        /// The click offset is the distance between the
        /// translation of the clicked entity and the position
        /// where the click occured.
        click_offset: Vec2,
    }

    /// Check if the mouse interacts with interactable entities in the world.
    ///
    /// If the mouse hovers over an interactable entity, the `Hover` marker
    /// component is inserted. Otherwise the marker component is removed.
    ///
    /// To check if the mouse hovers over an entity, the system uses the bounding
    /// box of the entity relative to its global transform.
    pub fn interaction_system(
        mut commands: Commands,
        // we need the mouse position within the world.
        mw: Res<MouseWorldPos>,
        // query to get all interactable entities.
        q_interact: Query<(Entity, &Interactable, &GlobalTransform)>,
    ) {
        for (entity, interactable, transform) in q_interact.iter() {
            if mw.x >= transform.translation.x + interactable.bounding_box.0.x
                && mw.x
                    <= transform.translation.x
                        + interactable.bounding_box.0.x
                        + interactable.bounding_box.1.x
                && mw.y >= transform.translation.y + interactable.bounding_box.0.y
                && mw.y
                    <= transform.translation.y
                        + interactable.bounding_box.0.y
                        + interactable.bounding_box.1.y
            {
                //eprintln!("hover {:?}", entity);
                commands.entity(entity).insert(Hover);
            } else {
                commands.entity(entity).remove::<Hover>();
            }
        }
    }

    /// Select interactable elements.
    ///
    /// A left click on an interactable entity will move it into its dedicated group.
    pub fn selection_system(
        mut commands: Commands,
        mw: Res<MouseWorldPos>,
        mb: ResMut<Input<MouseButton>>,
        mode: Res<InteractionMode>,
        // query all entities that are selectable and that
        // the mouse currently hovers over.
        q_select: Query<
            (Entity, &Transform, &Interactable, Option<&Draggable>),
            // Filter
            (With<Selectable>, With<Hover>),
        >,
        q_selected: Query<Entity, With<Selected>>,
        q_drag: Query<&Transform, With<Draggable>>,
        lock: Res<Lock>,
    ) {
        if !lock.0 && *mode == InteractionMode::Select && mb.just_pressed(MouseButton::Left) {
            let mut e: Option<Entity> = None;
            let mut greatest: f32 = -1.;
            let mut drag: bool = false;
            let mut pos: Vec2 = Vec2::new(0., 0.);

            for (entity, transform, _interact, draggable) in q_select.iter() {
                if transform.translation.z > greatest {
                    greatest = transform.translation.z;
                    drag = if let Some(_) = draggable { true } else { false };
                    pos.x = transform.translation.x;
                    pos.y = transform.translation.y;
                    e = Some(entity);
                }
            }

            let mut entities = Vec::new();
            for entity in q_selected.iter() {
                entities.push(entity);
            }

            if entities.len() > 1 {
                if let Some(entity) = e {
                    if entities.contains(&entity) {
                        for &e in entities.iter() {
                            if let Ok(t) = q_drag.get(e) {
                                let p = Vec2::new(t.translation.x, t.translation.y);

                                commands.entity(e).insert(Drag {
                                    click_offset: p - **mw,
                                });
                            }
                        }
                    } else {
                        for e in entities {
                            commands.entity(e).remove::<Selected>();
                        }

                        if drag {
                            commands.entity(entity).insert(Drag {
                                click_offset: pos - **mw,
                            });
                        }

                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    // Clicked on empty space -> deselect all
                    for entity in entities {
                        commands.entity(entity).remove::<Selected>();
                    }
                }
            } else {
                for entity in entities {
                    commands.entity(entity).remove::<Selected>();
                }

                if let Some(entity) = e {
                    if drag {
                        commands.entity(entity).insert(Drag {
                            click_offset: pos - **mw,
                        });
                    }

                    commands.entity(entity).insert(Selected);
                }
            }
        }
    }

    pub fn drag_system(
        mw: Res<MouseWorldPos>,
        mut q_drag: Query<(&mut Transform, &Draggable, &Drag), ()>,
    ) {
        for (mut transform, draggable, drag) in q_drag.iter_mut() {
            if draggable.update {
                transform.translation.x = mw.x + drag.click_offset.x;
                transform.translation.y = mw.y + drag.click_offset.y;
            }
        }
    }
}
