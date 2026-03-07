#!/bin/sh
gcc -O2 -o get-river-title get-river-title.c river-status.c $(pkg-config --cflags --libs wayland-client)
