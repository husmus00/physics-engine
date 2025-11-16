#include <stdio.h>
#include "pico/stdlib.h"
#include "hardware/i2c.h"

#define LIS3DH_ADDR 0x18  // Default I2C address (0x19 if SDO high)
#define LIS3DH_CTRL_REG1 0x20
#define LIS3DH_OUT_X_L 0x28

// Enable auto-increment for multi-byte reads
#define LIS3DH_OUT_X_L_AUTO 0xA8

void lis3dh_init(i2c_inst_t *i2c) {
    uint8_t data[2];
    
    // Configure CTRL_REG1: 100Hz ODR, normal mode, XYZ enabled
    data[0] = LIS3DH_CTRL_REG1;
    data[1] = 0x57;  // 0101 0111
    i2c_write_blocking(i2c, LIS3DH_ADDR, data, 2, false);
}

void lis3dh_read_xyz(i2c_inst_t *i2c, int16_t *x, int16_t *y, int16_t *z) {
    uint8_t buffer[6];
    uint8_t reg = LIS3DH_OUT_X_L_AUTO;
    
    i2c_write_blocking(i2c, LIS3DH_ADDR, &reg, 1, true);
    i2c_read_blocking(i2c, LIS3DH_ADDR, buffer, 6, false);
    
    *x = (int16_t)((buffer[1] << 8) | buffer[0]);
    *y = (int16_t)((buffer[3] << 8) | buffer[2]);
    *z = (int16_t)((buffer[5] << 8) | buffer[4]);
    
    // LIS3DH in normal mode gives 10-bit data left-justified in 16 bits
    *x >>= 6;
    *y >>= 6;
    *z >>= 6;
}

int main() {
    stdio_init_all();
    
    // Initialize I2C0 at 400kHz
    i2c_init(i2c0, 400 * 1000);
    gpio_set_function(0, GPIO_FUNC_I2C);
    gpio_set_function(1, GPIO_FUNC_I2C);
    gpio_pull_up(0);
    gpio_pull_up(1);
    
    sleep_ms(2000);  // Wait for serial connection
    printf("LIS3DH Test\n");
    
    lis3dh_init(i2c0);
    
    int16_t x, y, z;
    
    while (true) {
        lis3dh_read_xyz(i2c0, &x, &y, &z);
	printf("%d,%d,%d\n", x, y, z);
        sleep_ms(100);
    }
}
