cmake_minimum_required(VERSION 3.13)

# Include the Pico SDK
include($ENV{PICO_SDK_PATH}/external/pico_sdk_import.cmake)

project(lis3dh_test C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Initialize the Pico SDK
pico_sdk_init()

add_executable(lis3dh_test
    main.c
)

target_link_libraries(lis3dh_test
    pico_stdlib
    hardware_i2c
)

pico_enable_stdio_usb(lis3dh_test 1)
pico_enable_stdio_uart(lis3dh_test 0)

pico_add_extra_outputs(lis3dh_test)
