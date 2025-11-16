use serialport::{SerialPort, SerialPortType, DataBits, FlowControl, Parity, StopBits};
use std::time::Duration;
use std::thread;

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct AccelerometerReader {
    port: Box<dyn SerialPort>,
    smoothed_x: f32,
    smoothed_y: f32,
    smoothed_z: f32,
    smoothing: f32,
}

impl AccelerometerReader {
    pub fn new(port_name: &str, baud_rate: u32, smoothing: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut port = serialport::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_millis(100))
            .open()?;

        port.write_data_terminal_ready(true)?;
        port.write_request_to_send(true)?;

        println!("Port opened, resetting Pico...");
        thread::sleep(Duration::from_millis(3000));

        port.clear(serialport::ClearBuffer::All)?;

        println!("Ready to read data");

        Ok(Self {
            port,
            smoothed_x: 0.0,
            smoothed_y: 0.0,
            smoothed_z: 0.0,
            smoothing,
        })
    }

    pub fn read(&mut self) -> Result<Input, Box<dyn std::error::Error>> {
        let mut byte_buf = [0u8; 1];
        let mut line = String::new();

        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            match self.port.read(&mut byte_buf) {
                Ok(1) => {
                    let ch = byte_buf[0];

                    if ch == b'\n' {
                        if !line.is_empty() {
                            let parts: Vec<&str> = line.trim().split(',').collect();
                            if parts.len() == 3 {
                                if let (Ok(x), Ok(y), Ok(z)) = (
                                    parts[0].parse::<i16>(),
                                    parts[1].parse::<i16>(),
                                    parts[2].parse::<i16>(),
                                ) {
                                    let raw_x = x as f32 / 256.0;
                                    let raw_y = y as f32 / 256.0;
                                    let raw_z = z as f32 / 256.0;

                                    // Apply smoothing
                                    self.smoothed_x = self.smoothed_x * self.smoothing + raw_x * (1.0 - self.smoothing);
                                    self.smoothed_y = self.smoothed_y * self.smoothing + raw_y * (1.0 - self.smoothing);
                                    self.smoothed_z = self.smoothed_z * self.smoothing + raw_z * (1.0 - self.smoothing);

                                    return Ok(Input {
                                        x: self.smoothed_x,
                                        y: self.smoothed_y,
                                        z: self.smoothed_z,
                                    });
                                }
                            }
                        }
                        line.clear();
                    } else if ch != b'\r' && ch.is_ascii() {
                        line.push(ch as char);
                    }
                }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e.into()),
            }
        }

        Err("Timeout reading from accelerometer".into())
    }
}

pub fn find_pico_port() -> Option<String> {
    let ports = serialport::available_ports().ok()?;
    for port in ports {
        if let SerialPortType::UsbPort(info) = &port.port_type {
            if info.vid == 0x2E8A {
                return Some(port.port_name);
            }
        }
    }
    None
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let port_name = find_pico_port()
//         .ok_or("No Pico found")?;
//
//     println!("Found Pico at: {}", port_name);
//
//     let mut accel = AccelerometerReader::new(&port_name, 115200, 0.8)?;  // 0.8 = smoothing factor
//
//     for i in 0..50 {
//         let input = accel.read()?;  // Already smoothed!
//         println!("Sample {}: X={:6.2}g  Y={:6.2}g  Z={:6.2}g",
//                  i+1, input.x, input.y, input.z);
//     }
//
//     Ok(())
// }