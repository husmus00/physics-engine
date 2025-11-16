use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use crate::{things, Id};
use crate::things::{CollisionType, Dynamics, Shape, Sprite, Thing};


#[derive(Default)]
pub struct CollisionSpace {
    pub(crate) shapes: HashMap<Id, Shape>,
}

struct CollisionInfo {
    normal: (f32, f32),    // Direction to push objects apart
    penetration: f32,       // How much they overlap
}

impl CollisionSpace {

    fn register(&mut self,
                id: Id,
                transform: things::Transform,
                collision_type: CollisionType,
                restitution: f32,
                dynamics: Dynamics
    ) {
        let new_shape = Shape {
            transform,
            collision_type,
            dynamics,
            restitution,
            colliding: false,
        };

        self.shapes.insert(id, new_shape);
    }

    pub(crate) fn update(&mut self) {

        const PHYSICS_SUBSTEPS: u32 = 4;

        // 1. Apply gravity (continuous force) to dynamic objects
        self.handle_gravity();

        for _ in 0..PHYSICS_SUBSTEPS {

            // No need to update kinematic objects here

            // 2. Detect collisions
            let collisions = self.detect_collisions();
            self.reset_colliding_debug();
            self.set_colliding_debug(&collisions);

            // 3. Resolve collisions (push apart + bounce/slide)
            self.resolve_collisions(collisions);

            // 4. Integrate motion (velocity -> position)
            self.integrate_motion_substep(1.0 / PHYSICS_SUBSTEPS as f32);
        }
    }

    fn integrate_motion_substep(&mut self, dt_fraction: f32) {
        for thing in self.shapes.values_mut() {
            thing.transform.x_pos += thing.transform.vel_x * dt_fraction;
            thing.transform.y_pos += thing.transform.vel_y * dt_fraction;
        }
    }

    fn handle_gravity(&mut self)  {

        for thing in self.shapes.values_mut()
            .filter(|shape| matches!(shape.dynamics, Dynamics::Dynamic))
        {
            thing.transform.accel_y = 9.8 / 60.0 * 12.0; // Per frame, scaled by some factor to speed up
            thing.transform.vel_y += thing.transform.accel_y
        }
    }

    fn detect_collisions(&mut self) -> Vec<(Id, Id, CollisionInfo)> {
        let mut collisions = Vec::new();
        let ids: Vec<_> = self.shapes.keys().copied().collect();

        for i in 0..ids.len() {
            for j in i+1..ids.len() {
                let shape_a = &self.shapes[&ids[i]];
                let shape_b = &self.shapes[&ids[j]];

                // Skip if neither is dynamic (static-static don't need collision)
                let needs_check = matches!(shape_a.dynamics, Dynamics::Dynamic)
                    || matches!(shape_b.dynamics, Dynamics::Dynamic);

                if !needs_check {
                    continue;
                }

                let collision = match (&shape_a.collision_type, &shape_b.collision_type) {
                    (CollisionType::Circle, CollisionType::Circle) => {
                        let radius_a = shape_a.transform.width as f32 / 2.0;
                        let radius_b = shape_b.transform.width as f32 / 2.0;
                        detect_circle_circle(
                            (shape_a.transform.x_pos, shape_a.transform.y_pos),
                            radius_a,
                            (shape_b.transform.x_pos, shape_b.transform.y_pos),
                            radius_b,
                        )
                    }
                    (CollisionType::Circle, CollisionType::Rectangle) => {
                        let radius = shape_a.transform.width as f32 / 2.0;
                        detect_circle_rect(
                            (shape_a.transform.x_pos, shape_a.transform.y_pos),
                            radius,
                            (shape_b.transform.x_pos, shape_b.transform.y_pos),
                            shape_b.transform.width as f32,
                            shape_b.transform.height as f32,
                            shape_b.transform.rotation,
                        )
                    }
                    (CollisionType::Rectangle, CollisionType::Circle) => {
                        let radius = shape_b.transform.width as f32 / 2.0;
                        detect_circle_rect(
                            (shape_b.transform.x_pos, shape_b.transform.y_pos),
                            radius,
                            (shape_a.transform.x_pos, shape_a.transform.y_pos),
                            shape_a.transform.width as f32,
                            shape_a.transform.height as f32,
                            shape_a.transform.rotation,
                        )
                        //     .map(|mut info| {
                        //     // Flip the normal since we swapped order
                        //     info.normal = (-info.normal.0, -info.normal.1);
                        //     info
                        // })
                    }
                    _ => None,
                };

                if let Some(info) = collision {

                    println!("COLLISION: {} vs {}", ids[i], ids[j]);
                    println!("  Normal: ({:.3}, {:.3})", info.normal.0, info.normal.1);
                    println!("  Penetration: {:.3}", info.penetration);
                    println!("  Shape A pos: ({:.1}, {:.1})", shape_a.transform.x_pos, shape_a.transform.y_pos);
                    println!("  Shape B pos: ({:.1}, {:.1})", shape_b.transform.x_pos, shape_b.transform.y_pos);

                    collisions.push((ids[i], ids[j], info));
                }
            }
        }

        collisions
    }

    fn reset_colliding_debug(&mut self) {
        // reset set_colliding for all shapes
        for (id, shape) in &mut self.shapes {
            shape.colliding = false
        }
    }

    fn set_colliding_debug(&mut self, collisions: &[(Id, Id, CollisionInfo)]) {
        for (id_a, id_b, _info) in collisions {
            if let Some(shape) = self.shapes.get_mut(id_a) {
                shape.set_colliding(true);
            }
            if let Some(shape) = self.shapes.get_mut(id_b) {
                shape.set_colliding(true);
            }
        }
    }

    fn resolve_collisions(&mut self, collisions: Vec<(Id, Id, CollisionInfo)>) {
        for (id_a, id_b, info) in collisions {
            // We need to handle different dynamics combinations
            let dynamics_a = self.shapes[&id_a].dynamics;
            let dynamics_b = self.shapes[&id_b].dynamics;

            match (dynamics_a, dynamics_b) {
                // Both dynamic - push both apart
                (Dynamics::Dynamic, Dynamics::Dynamic) => {
                    self.resolve_dynamic_dynamic(id_a, id_b, &info);
                }
                // One dynamic, one kinematic/static - only push the dynamic one
                (Dynamics::Dynamic, _) => {
                    self.resolve_dynamic_static(id_a, &info, false);
                }
                (_, Dynamics::Dynamic) => {
                    // Flip the normal since b is dynamic
                    // let flipped_info = CollisionInfo {
                    //     normal: (-info.normal.0, -info.normal.1),
                    //     penetration: info.penetration,
                    // };
                    self.resolve_dynamic_static(id_b, &info, false);
                }
                _ => {} // Both static/kinematic - no resolution needed
            }
        }
    }

    fn resolve_dynamic_dynamic(&mut self, id_a: Id, id_b: Id, info: &CollisionInfo) {
        // Get both shapes (we know they exist)
        let correction = info.penetration / 2.0;

        // Push A away from B
        if let Some(shape_a) = self.shapes.get_mut(&id_a) {
            shape_a.transform.x_pos -= info.normal.0 * correction;
            shape_a.transform.y_pos -= info.normal.1 * correction;
        }

        // Push B away from A
        if let Some(shape_b) = self.shapes.get_mut(&id_b) {
            shape_b.transform.x_pos += info.normal.0 * correction;
            shape_b.transform.y_pos += info.normal.1 * correction;
        }

        // Now handle velocity response (bounce)
        self.apply_bounce(id_a, id_b, info);
    }

    fn resolve_dynamic_static(&mut self, dynamic_id: Id, info: &CollisionInfo, _is_kinematic: bool) {
        if let Some(shape) = self.shapes.get_mut(&dynamic_id) {
            // Push ALONG the normal (away from platform)
            shape.transform.x_pos += info.normal.0 * info.penetration;  // Use +=
            shape.transform.y_pos += info.normal.1 * info.penetration;  // Use +=

            self.apply_bounce_static(dynamic_id, info);
        }
    }

    fn apply_bounce(&mut self, id_a: Id, id_b: Id, info: &CollisionInfo) {
        // Get velocities (need to borrow separately)
        let (vel_a, vel_b, restitution_a, restitution_b) = {
            let shape_a = &self.shapes[&id_a];
            let shape_b = &self.shapes[&id_b];
            (
                (shape_a.transform.vel_x, shape_a.transform.vel_y),
                (shape_b.transform.vel_x, shape_b.transform.vel_y),
                shape_a.restitution,
                shape_b.restitution,
            )
        };

        // Combined restitution (how bouncy the collision is)
        let restitution = restitution_a * restitution_b;

        // Relative velocity along collision normal
        let rel_vel_x = vel_b.0 - vel_a.0;
        let rel_vel_y = vel_b.1 - vel_a.1;
        let vel_along_normal = rel_vel_x * info.normal.0 + rel_vel_y * info.normal.1;

        // Don't resolve if velocities are separating
        if vel_along_normal > 0.0 {
            return;
        }

        // Calculate impulse scalar
        let impulse_magnitude = -(1.0 + restitution) * vel_along_normal / 2.0;

        // Apply impulse to both objects
        let impulse_x = info.normal.0 * impulse_magnitude;
        let impulse_y = info.normal.1 * impulse_magnitude;

        if let Some(shape_a) = self.shapes.get_mut(&id_a) {
            shape_a.transform.vel_x -= impulse_x;
            shape_a.transform.vel_y -= impulse_y;
        }

        if let Some(shape_b) = self.shapes.get_mut(&id_b) {
            shape_b.transform.vel_x += impulse_x;
            shape_b.transform.vel_y += impulse_y;
        }
    }

    fn apply_bounce_static(&mut self, dynamic_id: Id, info: &CollisionInfo) {
        if let Some(shape) = self.shapes.get_mut(&dynamic_id) {
            let vel_along_normal = shape.transform.vel_x * info.normal.0
                + shape.transform.vel_y * info.normal.1;

            let restitution = shape.restitution * 0.8;
            let impulse = -(1.0 + restitution) * vel_along_normal;

            shape.transform.vel_x += info.normal.0 * impulse;
            shape.transform.vel_y += info.normal.1 * impulse;
        }
    }

    fn integrate_motion(&mut self) {
        for thing in self.shapes.values_mut() {
            thing.transform.x_pos += thing.transform.vel_x;
            thing.transform.y_pos += thing.transform.vel_y;
        }
    }
}

fn detect_circle_circle(
    pos_a: (f32, f32),
    radius_a: f32,
    pos_b: (f32, f32),
    radius_b: f32
) -> Option<CollisionInfo> {
    let dx = pos_b.0 - pos_a.0;
    let dy = pos_b.1 - pos_a.1;
    let distance = (dx * dx + dy * dy).sqrt();
    let min_distance = radius_a + radius_b;

    if distance < min_distance {
        // Calculate collision normal (direction from a to b)
        let normal_x = dx / distance;
        let normal_y = dy / distance;
        let penetration = min_distance - distance;

        Some(CollisionInfo {
            normal: (normal_x, normal_y),
            penetration,
        })
    } else {
        None
    }
}

fn detect_circle_rect(
    circle_pos: (f32, f32),
    radius: f32,
    rect_pos: (f32, f32),
    rect_width: f32,
    rect_height: f32,
    rotation_deg: f32,  // Just pass 0.0 for axis-aligned
) -> Option<CollisionInfo> {
    let angle = rotation_deg.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();

    // Transform circle into rectangle's local space
    let dx = circle_pos.0 - rect_pos.0;
    let dy = circle_pos.1 - rect_pos.1;
    let local_x = dx * cos + dy * sin;
    let local_y = -dx * sin + dy * cos;

    // AABB check in local space
    let half_w = rect_width / 2.0;
    let half_h = rect_height / 2.0;

    let closest_x = local_x.clamp(-half_w, half_w);
    let closest_y = local_y.clamp(-half_h, half_h);

    let dx_local = local_x - closest_x;
    let dy_local = local_y - closest_y;
    let distance_sq = dx_local * dx_local + dy_local * dy_local;

    if distance_sq < radius * radius {
        let distance = distance_sq.sqrt();

        // Safety check for zero distance
        if distance < 0.001 {
            return None; // Objects are exactly overlapping, skip this frame
        }

        let normal_x_local = dx_local / distance;
        let normal_y_local = dy_local / distance;

        // Rotate normal back to world space
        let normal_x = normal_x_local * cos - normal_y_local * sin;
        let normal_y = normal_x_local * sin + normal_y_local * cos;

        Some(CollisionInfo {
            normal: (normal_x, normal_y),
            penetration: radius - distance,
        })
    } else {
        None
    }
}



#[derive(Default)]
pub struct Space {
    counter: AtomicU16,
    pub(crate) things: HashMap<Id, Thing>,
}

impl Space {
    fn gen_id(&self) -> Id {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn register(&mut self,
                           init_transform: things::Transform,
                           sprite: Sprite,
                           mut color: Option<(u8, u8, u8)>,
                           dynamics: Option<Dynamics>,
                           restitution: f32,
                           collision_space: &mut CollisionSpace
    ) -> Id {
        let new_id = self.gen_id();

        if let None = color {
            color = Some((0,0,0));
        }

        let dynamic = if let Some(_) = dynamics { true } else { false };

        let new_thing = Thing {
            id: new_id,
            sprite,
            color: color.unwrap(),
            dynamic,
        };

        if matches!(dynamics.unwrap(), Dynamics::Dynamic | Dynamics::Kinematic) {
            collision_space.register(new_thing.id, init_transform, CollisionType::from(&new_thing.sprite), restitution, dynamics.unwrap());
        }

        self.things.insert(new_thing.id, new_thing);

        new_id
    }
}
