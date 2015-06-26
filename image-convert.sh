#!/bin/bash
for FILE in src/xcf/*; do
    echo xcf2png "$FILE" -o "src/png/${FILE:8:-4}.png"
    xcf2png "$FILE" -o "src/png/${FILE:8:-4}.png"
done
