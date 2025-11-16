mod collision;
mod loader;
mod things;
mod controller;

use crate::collision::{CollisionSpace, Space};
use raylib::prelude::*;
use things::*;
use crate::controller::{find_pico_port, AccelerometerReader, Input};

type Id = u16;

const SCREEN_WIDTH: i32 = 920;
const SCREEN_HEIGHT: i32 = 640;

const DEBUG: bool = false;


fn main() {
    let (mut rl, thread) = init_visualiser();

    // Load visual data if available (textures, sprites)

    // Create instance of empty collision space (check for collisions and update transform)
    let mut collision_space = CollisionSpace::default();

    // Create instance of space (list/tree of objects
    // each holding visual ref. + shape (collide ref. + transform) data)
    let mut space = Space::default();

    // Register the platform

    let p_x_pos = (SCREEN_WIDTH / 2) as f32;
    let p_y_pos = (SCREEN_HEIGHT / 2) as f32;
    let p_width = 600;
    let p_height = 40;
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

    let port_name = find_pico_port()
        .ok_or("Could not find Pico").unwrap();

    println!("Connecting to {}...", port_name);

    let mut controller = AccelerometerReader::new(&port_name, 115200, 0.6).unwrap();
    let mut platform_axes= Input::default();

    // Load objects from e.g., json file
    // Each object registers itself with the space (and potentially collision space)

    let num_objects: i32 = 100;
    let num_to_add: i32 = 10;
    let diameter = 20;

    let mut frame_count = 0;

    while !rl.window_should_close() {

        frame_count += 1;

        if space.things.len() < num_objects as usize && frame_count % 20 == 0 {
            for i in 0..num_to_add {
                let ball_transform = things::Transform::new((SCREEN_WIDTH / num_to_add * i) as f32, 30.0, diameter, diameter, 0.0);
                space.register(ball_transform, Sprite::Circle, None, Some(Dynamics::Dynamic), 0.6, &mut collision_space);
            }
        }

        // Handle player input
        if let Some(new_input) = controller.read_non_blocking() {
            platform_axes = new_input;
        }

        // Handle platform (kinematic) updates
        let platform : &mut Shape = &mut collision_space.shapes.get_mut(&platform_id).unwrap();
        platform.set_rotation(((platform_axes.x * 100.0).round() / 100.0) * 65.0);

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
        let debug_text = format!("FPS: {}\nNumber of Objects: {}", d.get_fps(), space.things.len());
        d.draw_text(&debug_text, 10, 10, 5, Color::BLACK);
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
