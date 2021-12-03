use bevy::prelude::*;

pub struct World2DPlugin;

impl Plugin for World2DPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(camera::MouseWorldPos(Vec2::new(0., 0.)))
            .add_startup_system(camera::setup.system())
            .add_system(camera::cursor_system.system())
            .add_system(interaction::interaction_system.system());
    }
}

pub mod camera {
    use bevy::prelude::*;
    use core::ops::{Deref, DerefMut};

    /// Used to help identify the main camera.
    pub struct MainCamera;
    
    /// Position of the mouse cursor within the 2d world.
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

pub mod interaction {
    use super::camera::MouseWorldPos;
    use bevy::prelude::*;
    
    /// Component that marks an entity interactable.
    pub struct Interactable {
        /// The bounding box defines the area where the mouse
        /// can interact with the entity.
        bounding_box: (Vec2, Vec2),
    }
    
    impl Interactable {
        /// Create a new interactable component.
        ///
        /// # Arguments
        ///
        /// * `position` - The position of the bounding box within the world.
        /// * `dimensions` - The width and the height of the bounding box.
        ///
        /// The `position` marks the center of the bounding box.
        pub fn new(position: Vec2, dimensions: Vec2) -> Self {
            Self {
                bounding_box: (Vec2::new(position.x - dimensions.x / 2., 
                               position.y - dimensions.y / 2.), dimensions),
            }
        }
    }
    
    /// Component that marks an entity as draggable.
    pub struct Draggable;
    
    /// Marker component to indicate that the mouse
    /// currently hovers over the given entity.
    pub struct Hover;
    
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
    /// * If the mouse hovers over an interactable entity, the `Hover` marker
    /// component is inserted. Otherwise the marker component is removed.
    pub fn interaction_system(
        mut commands: Commands,
        // we need the mouse position within the world.
        mw: Res<MouseWorldPos>,
        // query to get all interactable entities.
        q_interact: Query<(Entity, &Interactable)>
    ) {
        for (entity, interactable) in q_interact.iter() {
            if mw.x >= interactable.bounding_box.0.x &&
                mw.x <= interactable.bounding_box.0.x + interactable.bounding_box.1.x &&
                mw.y >= interactable.bounding_box.0.y &&
                mw.y <= interactable.bounding_box.0.y + interactable.bounding_box.1.y 
            {
                //eprintln!("hover {:?}", entity);    
                commands.entity(entity).insert(Hover);
            } else {
                commands.entity(entity).remove::<Hover>();
            }
        }
    }
    
}





























