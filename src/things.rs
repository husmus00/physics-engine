use crate::Id;

pub struct Transform {
    pub x_pos: i32,
    pub y_pos: i32,
    pub width: i32,
    pub height: i32,
    pub rotation: i32,
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

pub struct Shape {
    pub transform: Transform,
    pub collision_type: CollisionType,
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