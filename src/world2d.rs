use bevy::prelude::*;

pub mod camera2d {
    use bevy::prelude::*;
    use core::ops::{Deref, DerefMut};

    pub struct Camera2DPlugin;

    impl Plugin for Camera2DPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.insert_resource(MouseWorldPos(Vec2::new(0., 0.)))
                .add_startup_system(setup.system())
                .add_system(cursor_system.system());
        }
    }

    /// Used to help identify the main camera.
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
        q_camera: Query<&Transform, With<MainCamera>>
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
            let camera_transform = q_camera.single().unwrap();

            // apply the camera transform.
            let pos_wld = camera_transform
                .compute_matrix() * p.extend(0.).extend(1.);
            //eprintln!("{}:{}", pos_wld.x, pos_wld.y);
            mw.x = pos_wld.x;
            mw.y = pos_wld.y;
        }
    }
}

pub mod interaction2d {
    use super::camera2d::MouseWorldPos;
    use bevy::prelude::*;
    use core::ops::{Deref, DerefMut};
    use std::collections::HashMap;

    pub struct Interaction2DPlugin;

    impl Plugin for Interaction2DPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.insert_resource(Selected(HashMap::new()))
                .add_system(interaction_system.system().label("interaction"))
                .add_system(selection_system.system().after("interaction"))
                .add_system(drag_system.system());
        }
    }
    
    /// Resource that holds all selected entities.
    #[derive(Debug)]
    pub struct Selected(pub HashMap<u32, Vec<Entity>>);

    impl Deref for Selected {
        type Target = HashMap<u32, Vec<Entity>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for Selected {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    
    /// Component that marks an entity as selectable.
    pub struct Selectable;
    
    /// Component that marks an entity interactable.
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
                bounding_box: (Vec2::new(position.x - dimensions.x / 2., 
                               position.y - dimensions.y / 2.), dimensions),
                group
            }
        }
        
        /// Update the position of the bounding box within the world.
        pub fn update_pos(&mut self, x: f32, y: f32) {
            self.bounding_box.0.x = x;
            self.bounding_box.0.y = y;
        }
    }
    
    /// Marker component to indicate that the mouse
    /// currently hovers over the given entity.
    pub struct Hover;
    
    /// Component that marks an entity as draggable.
    pub struct Draggable {
        /// The drag system should automatically update
        /// the entities transformation while being dragged
        /// [y/ n].
        pub update: bool,
    }

    /// Marker component to indicate that the
    /// given entity is currently dragged.
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
        q_interact: Query<(Entity, &Interactable, &GlobalTransform)>
    ) {
        for (entity, interactable, transform) in q_interact.iter() {
            if mw.x >= transform.translation.x + interactable.bounding_box.0.x &&
                mw.x <= transform.translation.x + interactable.bounding_box.0.x + 
                    interactable.bounding_box.1.x &&
                mw.y >= transform.translation.y + interactable.bounding_box.0.y &&
                mw.y <= transform.translation.y + interactable.bounding_box.0.y + 
                    interactable.bounding_box.1.y 
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
        mb: Res<Input<MouseButton>>,
        mut selected: ResMut<Selected>,
        // query all entities that are selectable and that 
        // the mouse currently hovers over.
        q_select: Query<
            (Entity, &Transform, &Interactable, Option<&Draggable>), 
            // Filter
            (With<Selectable>, With<Hover>)
        >
    ) {
        if mb.just_pressed(MouseButton::Left) {
            let mut e: Option<Entity> = None;
            let mut greatest: f32 = 0.;
            let mut group_id: u32 = 0;
            let mut drag: bool = false;
            let mut pos: Vec2 = Vec2::new(0., 0.);

            for (entity, transform, interact, draggable) in q_select.iter() {
                if transform.translation.z > greatest {
                    greatest = transform.translation.z;
                    group_id = interact.group;
                    drag = if let Some(_) = draggable { true } else { false };
                    pos.x = transform.translation.x;
                    pos.y = transform.translation.y;
                    e = Some(entity);
                }
            }
            
            // Clear table so information are only availabe for one tick.
            selected.clear();

            if let Some(entity) = e {
                if drag {
                    commands.entity(entity).insert(Drag { click_offset: pos - **mw });
                }
                selected.entry(group_id).or_insert(Vec::new()).push(entity);
            }
        }
    }

    pub fn drag_system(
        mw: Res<MouseWorldPos>,
        mut q_drag: Query<(&mut Transform, &Draggable, &Drag), ()>
    ) {
        for (mut transform, draggable, drag) in q_drag.iter_mut() {
            if draggable.update {
                transform.translation.x = mw.x + drag.click_offset.x;
                transform.translation.y = mw.y + drag.click_offset.y;
            }
        }
    }
}





























