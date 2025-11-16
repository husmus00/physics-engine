picotool load -f build/*.uf2
picotool reboot

PORT=/dev/ttyACM0 

# --- Wait for /dev/ttyACM0 to appear ---
echo "⏳ Waiting for $PORT to appear..."
for i in {1..20}; do
    if [ -e "$PORT" ]; then
        echo "Found $PORT!"
        break
    fi
    sleep 0.5
done

screen $PORT 115200 
