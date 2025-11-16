use crate::Id;

pub struct Transform {
    pub x_pos: f32,
    pub y_pos: f32,
    pub width: i32,
    pub height: i32,
    pub rotation: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub accel_x: f32,
    pub accel_y: f32,
}

impl Transform {
    pub(crate) fn new(x_pos: f32, y_pos: f32, width: i32, height: i32, rotation: f32) -> Self {
        Transform {
            x_pos,
            y_pos,
            width,
            height,
            rotation,
            vel_x: 0.0,
            vel_y: 0.0,
            accel_x: 0.0,
            accel_y: 0.0
        }
    }
}

pub enum CollisionType {
    Circle,
    Rectangle,
}

impl From<&Sprite> for CollisionType {
    fn from(value: &Sprite) -> Self {
        match value {
            Sprite::Circle => CollisionType::Circle,
            Sprite::Rectangle => CollisionType::Rectangle,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Dynamics {
    None,
    Dynamic,
    Kinematic,
}

pub struct Shape {
    pub transform: Transform,
    pub collision_type: CollisionType,
    pub dynamics: Dynamics,
    pub restitution: f32,
    pub colliding: bool,
}

impl Shape {
    pub fn set_rotation(&mut self, rotation: f32) {
        self.transform.rotation = rotation;
    }

    pub fn set_colliding(&mut self, colliding: bool) {
        self.colliding = colliding;
    }
}

#[derive(Debug)]
pub enum Sprite {
    Circle,
    Rectangle,
}




#[derive(Debug)]
pub struct Thing {
    pub id: Id, // Also reference to collision space
    pub sprite: Sprite,
    pub color: (u8, u8, u8),
    pub dynamic: bool,
}