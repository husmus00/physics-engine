mod collision;
mod loader;
mod things;
mod controller;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU16,Ordering};
use raylib::ffi::KeyboardKey::{KEY_LEFT, KEY_RIGHT};
use raylib::prelude::*;
use things::*;
use crate::controller::{find_pico_port, AccelerometerReader, Input};

type Id = u16;


#[derive(Default)]
struct CollisionSpace {
    shapes: HashMap<Id, Shape>,
}

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

impl CollisionSpace {

    fn register(&mut self, id: Id, transform: things::Transform, collision_type: CollisionType) {
        let new_shape = Shape {
            transform,
            collision_type
        };

        self.shapes.insert(id, new_shape);
    }

    fn handle_collisions(&mut self) {
        // Do nothing for now
    }
}



#[derive(Default)]
struct Space {
    counter: AtomicU16,
    things: HashMap<Id, Thing>,
}

impl Space {
    fn gen_id(&self) -> Id {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    fn register(&mut self, init_transform: things::Transform, sprite: Sprite, dynamic: bool, collision_space: &mut CollisionSpace) {
        let new_thing = Thing {
            id: self.gen_id(),
            sprite,
            color: (0,0,0),
            dynamic,
        };

        if dynamic {
            collision_space.register(new_thing.id, init_transform, CollisionType::from(&new_thing.sprite));
        }

        self.things.insert(new_thing.id, new_thing);
    }
}

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
    let transform_1 = things::Transform { x_pos: 160, y_pos: 30, width: 20, height: 20, rotation: 0 };
    let transform_2 = things::Transform { x_pos: 480, y_pos: 30, width: 20, height: 20, rotation: 0 };

    space.register(transform_1, Sprite::Circle, true, &mut collision_space);
    space.register(transform_2, Sprite::Circle, true, &mut collision_space);


    let mut frame = 0;
    let mut pos_x = 320;
    let mut pos_y = 0;

    let port_name = find_pico_port()
        .ok_or("Could not find Pico").unwrap();

    println!("Connecting to {}...", port_name);

    // Use 0.8 as smoothing factor
    let mut accel = AccelerometerReader::new(&port_name, 115200, 0.6).unwrap();


    while !rl.window_should_close() {

        // Handle player input

        let input = accel.read();
        let platform_axes: Input;
        match input {
            Ok(input) => {
                println!("Acceleration: X={:.2}g Y={:.2}g Z={:.2}g", input.x, input.y, input.z);
                platform_axes = Input{ x: input.x, y: input.y, z: input.z};
            },
            Err(e) => {
                println!("Error reading accel: {:?}", e);
                platform_axes = Input { x: 0.0, y: 0.0, z: 0.0 };
            }
        }

        // Handle collision detection

        // Handle rigid object dynamics

        // Handle visualisation
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for (id, t) in &space.things {
            // println!("Drawing thing {}", id);
            let transform = &collision_space.shapes.get(&id).unwrap().transform;
            let (r, g, b) = t.color;

            match t.sprite {
                Sprite::Circle => {
                    d.draw_circle(transform.x_pos, transform.y_pos, (transform.height / 2) as f32, Color::new(r, g, b, 255))
                },
                Sprite::Rectangle => {
                    d.draw_rectangle(transform.x_pos, transform.y_pos, transform.width, transform.height, Color::new(r, g, b, 255))
                }
            }
        }

        // Print platform

        let p_width = 300.0;
        let p_height = 35.0;
        let p_rotation = ((platform_axes.x * 100.0).round() / 100.0) * 65.0;
        d.draw_rectangle_pro(Rectangle::new((SCREEN_WIDTH / 2) as f32, (SCREEN_HEIGHT / 2) as f32, p_width, p_height), Vector2::new(p_width / 2.0, p_height / 2.0), p_rotation, Color::RED);


        // ----


        // println!("frame: {}", frame);
        // frame += 1;

        d.draw_circle(pos_x, pos_y, 20., Color::BLACK);

        if frame % 2 == 0 {
            pos_y += 5;
        }

        if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
            d.draw_rectangle(0, 0, 12, 12, Color::BLACK);
            d.draw_text("WORKING", 20, 20, 12, Color::BLACK);
        }

        if d.is_key_down(KEY_RIGHT) {
            pos_x += 1;
        }

        if d.is_key_down(KEY_LEFT) {
            pos_x -= 1;
        }
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
