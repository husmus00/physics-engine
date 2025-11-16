mod collision;
mod loader;
mod things;
mod controller;

use crate::collision::{CollisionSpace, Space};
use raylib::prelude::*;
use things::*;
use crate::controller::{find_pico_port, AccelerometerReader, Input};

type Id = u16;

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

const DEBUG: bool = true;


fn main() {
    let (mut rl, thread) = init_visualiser();

    // Load visual data if available (textures, sprites)

    // Create instance of empty collision space (check for collisions and update transform)
    let mut collision_space = CollisionSpace::default();

    // Create instance of space (list/tree of objects
    // each holding visual ref. + shape (collide ref. + transform) data)
    let mut space = Space::default();

    // Load objects from e.g., json file
    // Each object registers itself with the space (and potentially collision space)
    let ball_transform_1 = things::Transform::new(240.0, 30.0, 30, 30, 0.0);
    let ball_transform_2 =  things::Transform::new(400.0, 30.0, 30, 30, 0.0);

    space.register(ball_transform_1, Sprite::Circle, None, Some(Dynamics::Dynamic), 0.7, &mut collision_space);
    space.register(ball_transform_2, Sprite::Circle, None, Some(Dynamics::Dynamic), 0.7, &mut collision_space);

    // Register the platform

    let p_x_pos = (SCREEN_WIDTH / 2) as f32;
    let p_y_pos = (SCREEN_HEIGHT / 2) as f32;
    let p_width = 300;
    let p_height = 25;
    let p_rotation = 0.0; //
    let platform_transform = things::Transform::new(
        p_x_pos,
        p_y_pos,
        p_width,
        p_height,
        p_rotation,
    );
    let platform_id = space.register(
        platform_transform,
        Sprite::Rectangle,
        Some((0,0,255)),
        Some(Dynamics::Kinematic),
        0.0,
        &mut collision_space
    );


    // let mut frame = 0;
    // let mut pos_x = 320;
    // let mut pos_y = 0;

    let port_name = find_pico_port()
        .ok_or("Could not find Pico").unwrap();

    println!("Connecting to {}...", port_name);

    // Use 0.8 as smoothing factor
    let mut controller = AccelerometerReader::new(&port_name, 115200, 0.6).unwrap();


    while !rl.window_should_close() {

        // Handle player input

        let input = controller.read();
        let platform_axes: Input;
        match input {
            Ok(input) => {
                // println!("Acceleration: X={:.2}g Y={:.2}g Z={:.2}g", input.x, input.y, input.z);
                platform_axes = Input{ x: input.x, y: input.y, z: input.z};
            },
            Err(e) => {
                println!("Error reading accel: {:?}", e);
                platform_axes = Input { x: 0.0, y: 0.0, z: 0.0 };
            }
        }

        // Handle platform (kinematic) updates
        let platform : &mut Shape = &mut collision_space.shapes.get_mut(&platform_id).unwrap();
        platform.set_rotation(((platform_axes.x * 100.0).round() / 100.0) * 65.0);
        // println!("Platform registered at: ({}, {})", p_x_pos, p_y_pos);


        // Handle collision / transform updates
        collision_space.update();


        // Handle visualisation
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);


        // For each object, draw and handle off-screen behaviour
        for (id, t) in &space.things {
            // println!("Drawing thing {}", id);
            let shape = collision_space.shapes.get_mut(&id).unwrap();
            let transform: &mut things::Transform = &mut shape.transform;
            let (r, g, b) = t.color;

            let mut color = Color::new(r, g, b, 255);
            if DEBUG && shape.colliding {
                color = Color::RED;
            }

            match t.sprite {
                Sprite::Circle => {
                    d.draw_circle(
                        transform.x_pos as i32,
                        transform.y_pos as i32,
                        (transform.height / 2) as f32,
                        color
                    )
                },
                Sprite::Rectangle => {

                    d.draw_rectangle_pro(
                        Rectangle::new(
                            transform.x_pos,
                            transform.y_pos,
                            transform.width as f32,
                            transform.height as f32
                        ),
                        Vector2::new(
                            (transform.width / 2) as f32,
                            (transform.height / 2) as f32
                        ),
                        transform.rotation,
                        color
                    );
                }
            }

            // Handle off-screen
            if transform.x_pos > (SCREEN_WIDTH + transform.width) as f32 {
                transform.x_pos =  0.0 - transform.width as f32
            }

            if transform.x_pos < (0 - transform.width) as f32 {
                transform.x_pos =  (SCREEN_WIDTH + transform.width) as f32
            }

            if transform.y_pos > (SCREEN_HEIGHT + transform.height) as f32 {
                transform.y_pos =  0.0 - transform.height as f32
            }

            if transform.y_pos < (0 - transform.height) as f32 {
                transform.y_pos =  (SCREEN_HEIGHT + transform.height) as f32
            }
        }

        // Draw debug information
    }
}

fn init_visualiser() -> (RaylibHandle, RaylibThread) {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Hello, World")
        .build();

    rl.set_target_fps(60);

    (rl, thread)
}
