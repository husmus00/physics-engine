# 2D Physics Engine with Accelerometer Control


A modular 2D rigid body physics engine written in Rust, featuring real-time hardware control via a Raspberry Pi Pico 2W accelerometer. The engine simulates gravity, collisions, and bouncing dynamics for circles and rectangles.

https://github.com/user-attachments/assets/e7717ed8-85cd-471e-8d70-47ba47060c9d

(I was using that second Pico in the video for debugging)

## Architecture

The project is structured with separation of concerns:

- Collision Space: Handles physics simulation, collision detection, and resolution
- Visual Space: Manages rendering and visual representation
- Controller: Interfaces with hardware input devices

This modular design allows the rendering backend to be swapped out. While the current implementation uses Raylib, the physics engine operates independently and could use other graphics libraries.

## Hardware Setup

### Components
- Raspberry Pi Pico 2W
- Adafruit LIS3DH accelerometer breakout board

### Firmware
The Pico firmware reads accelerometer data via I2C and transmits it over USB serial as CSV format. Compile and flash the C firmware using the Pico SDK with USB stdio enabled.

## Current Features

- Gravity simulation with configurable strength
- Physics sub-stepping to prevent tunneling
- Dynamic, kinematic, and static rigid body support
- Collision detection for circles and rectangles with arbitrary rotation
- Positional correction for overlapping objects
- Impulse-based velocity resolution with configurable restitution
- Non-blocking Real-time serial communication with hardware accelerometer
- Smoothing filter for sensor noise
- Cross-platform support (Windows and Linux)
- Raylib-based rendering with debug visualization

## Planned Optimizations

- Spatial partitioning with uniform grid
- AABB broad phase collision filtering
- Resting contact detection and optimization
- Convex polygon collision via SAT
- Capsule and compound shape primitives
- Friction simulation (static and dynamic)
- Angular velocity and torque from off-center collisions
- Constraint solving for joints and springs
- Configurable material property combine modes
- Performance profiling and benchmarking
- Unit tests for collision algorithms
- Configuration file for physics parameters
