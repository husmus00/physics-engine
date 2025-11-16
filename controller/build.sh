rm -rf build
mkdir build
cd build
cmake -DPICO_PLATFORM=rp2350 -DPICO_BOARD=pico2_w ..
make -j$(nproc)

