use bevy::{core::FixedTimestep, ecs::system::Command, prelude::*};
use bevy::render::pass::ClearColor;

// Constants

const TOTAL_HEIGHT: u32 = 16;
const TOTAL_WIDTH: u32 = 8;

const WINDOW_REAL_HEIGHT: f32 = 500.0;

const FALL_TIMESTEP: f64 = 0.6;

// MARK: - Setup

fn main() {
    App::build()
    // Setup:
    .insert_resource(WindowDescriptor {
        title: "Tetris".to_string(),
        width: WINDOW_REAL_HEIGHT * (TOTAL_WIDTH as f32 / TOTAL_HEIGHT as f32),
        height: WINDOW_REAL_HEIGHT,
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .add_startup_stage("game_setup", SystemStage::single(spawn_active_block.system()))
    
    // Falling:
    .add_system_set(SystemSet::new()
    .with_run_criteria(FixedTimestep::step(FALL_TIMESTEP))
    .with_system(falling_movement.system())
    .with_system(place_active_blocks.system()))
    
    // Moving (control):
    .add_system_set(SystemSet::new()
    .with_run_criteria(FixedTimestep::step(FALL_TIMESTEP * 0.1))
    .with_system(control_movement.system()))
    
    // Scaling and translation:
    .add_system_set_to_stage(
        CoreStage::PostUpdate, 
        SystemSet::new()
        .with_system(position_translation.system())
        .with_system(size_scaling.system()))
        .run();
    }
    
    fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        commands.insert_resource(Materials {
            activeBlockMaterial: materials.add(Color::rgb(1., 0., 0.).into()),
            normalBlockMaterial: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
        });
    }
    
    fn spawn_active_block(mut commands: Commands, materials: Res<Materials>) {
        commands
        .spawn_bundle(SpriteBundle {
            material: materials.activeBlockMaterial.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .insert(ActiveBlock)
        .insert(Position { x: 5, y: TOTAL_HEIGHT as i32 - 1 })
        .insert(Size::square(1.0));
    }
    
    #[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
    struct Position {
        x: i32,
        y: i32,
    }
    
    struct Size {
        width: f32,
        height: f32,
    }
    impl Size {
        pub fn square(x: f32) -> Self {
            Self {
                width: x,
                height: x,
            }
        }
    }
    
    // MARK: - ?
    
    struct ActiveBlock;
    struct NormalBlock;
    
    struct Materials {
        activeBlockMaterial: Handle<ColorMaterial>,
        normalBlockMaterial: Handle<ColorMaterial>,
    }
    
    // MARK: - Systems
    
    // Moves the active block down
    fn falling_movement(mut active_block_positions: Query<&mut Position, With<ActiveBlock>>) {
        for (mut position) in active_block_positions.iter_mut() {
            if position.y > 0 {
                position.y -= 1
            }
        }
    }
    
    // Moves the active block left/right in response to keyboard
    fn control_movement(
        keyboard_input: Res<Input<KeyCode>>,
        mut active_block_positions: Query<&mut Position, With<ActiveBlock>>,
    ) {
        for mut position in active_block_positions.iter_mut() {
            if keyboard_input.pressed(KeyCode::Left) && position.x > 0 {
                position.x -= 1;
            } else if keyboard_input.pressed(KeyCode::Right) && position.x < (TOTAL_WIDTH as i32 - 1) {
                position.x += 1;
            } else if keyboard_input.pressed(KeyCode::Down) && position.y > 0 {
                position.y -= 1;
            }
        }
    }
    
    fn place_active_blocks(
        mut commands: Commands, 
        materials: Res<Materials>, 
        mut query: QuerySet<(
            Query<&mut Position, With<ActiveBlock>>, 
            Query<&Position, With<NormalBlock>>,
        )>
    ) {
        
        let mut act_x: i32 = 0;
        let mut act_y: i32 = 0;
        let mut did_place: bool = false;
        
        for position in query.q0_mut().iter_mut() {
            act_x = position.x;
            act_y = position.y;
        }
        
        if act_y == 0 {
            // We've hit the bottom of the field
            _place_normal_block(commands, materials, act_x, act_y);
            did_place = true;

        } else {
            for normal_position in query.q1().iter() {
                if act_x == normal_position.x
                && act_y == (normal_position.y + 1) {
                    // We've hit the top of a normal block
                    _place_normal_block(commands, materials, act_x, act_y);
                    did_place = true;
                    break;
                }
            }
        }
        
        if did_place {
            for mut position in query.q0_mut().iter_mut() {
                position.y = TOTAL_HEIGHT as i32 - 1
            }
        }
    }
    
    fn _place_normal_block(
        mut commands: Commands,
        materials: Res<Materials>,
        x: i32, 
        y: i32,
    ) {
        commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.normalBlockMaterial.clone(),
                        sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                        ..Default::default()
                    })
                    .insert(NormalBlock)
                    .insert(Position { 
                        x: x,
                        y: y,
                    })
                    .insert(Size::square(1.0));
                    
                    // Move active block back to the top
                    // position.y = TOTAL_HEIGHT as i32 - 1;
    }
    
    // MARK: - Systems - Size/pos scaling
    
    // Scales real sprite sizes according to their component sizes
    fn size_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Sprite)>) {
        let window = windows.get_primary().unwrap();
        let window_width = window.width();
        let window_height = window.height();
        
        for (sprite_size, mut sprite) in query.iter_mut() {
            sprite.size = Vec2::new(
                sprite_size.width / TOTAL_WIDTH as f32 * window_width,
                sprite_size.height / TOTAL_HEIGHT as f32 * window_height);
            }
        }
        
        // Updates real sprite transforms according to their component positions
        fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
            // Helper method:
            fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
                let tile_size = bound_window / bound_game;
                pos / bound_game * bound_window - (bound_window / 2.0) + (tile_size / 2.)
            }
            
            let window = windows.get_primary().unwrap();
            let window_width = window.width();
            let window_height = window.height();
            
            for (pos, mut transform) in query.iter_mut() {
                transform.translation = Vec3::new(
                    convert(pos.x as f32, window_width as f32, TOTAL_WIDTH as f32), 
                    convert(pos.y as f32, window_height as f32, TOTAL_HEIGHT as f32),
                    0.0); 
                }
            }