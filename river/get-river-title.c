
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wayland-client.h>
#include "river-status.h"

struct river_status_data {
    struct zriver_status_manager_v1 *manager;
    struct zriver_seat_status_v1 *seat_status;
    struct wl_seat *seat;
    char *focused_title;
    int done;
};

static void seat_status_focused_output(void *data, struct zriver_seat_status_v1 *zriver_seat_status_v1, struct wl_output *output) {}
static void seat_status_unfocused_output(void *data, struct zriver_seat_status_v1 *zriver_seat_status_v1, struct wl_output *output) {}
static void seat_status_mode(void *data, struct zriver_seat_status_v1 *zriver_seat_status_v1, const char *name) {}

static void seat_status_focused_view(void *data, struct zriver_seat_status_v1 *zriver_seat_status_v1, const char *title) {
    struct river_status_data *d = data;
    if (d->focused_title) free(d->focused_title);
    d->focused_title = strdup(title);
    d->done = 1;
}

static const struct zriver_seat_status_v1_listener seat_status_listener = {
    .focused_output = seat_status_focused_output,
    .unfocused_output = seat_status_unfocused_output,
    .focused_view = seat_status_focused_view,
    .mode = seat_status_mode,
};

static void registry_handle_global(void *data, struct wl_registry *registry, uint32_t name, const char *interface, uint32_t version) {
    struct river_status_data *d = data;
    if (strcmp(interface, zriver_status_manager_v1_interface.name) == 0) {
        d->manager = wl_registry_bind(registry, name, &zriver_status_manager_v1_interface, 1);
    } else if (strcmp(interface, "wl_seat") == 0) {
        d->seat = wl_registry_bind(registry, name, &wl_seat_interface, 1);
    }
}

static void registry_handle_global_remove(void *data, struct wl_registry *registry, uint32_t name) {}

static const struct wl_registry_listener registry_listener = {
    .global = registry_handle_global,
    .global_remove = registry_handle_global_remove,
};

int main() {
    struct wl_display *display = wl_display_connect(NULL);
    if (!display) {
        fprintf(stderr, "Failed to connect to wayland display\n");
        return 1;
    }

    struct river_status_data data = {0};
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, &data);

    // Initial roundtrip to find all globals
    wl_display_roundtrip(display);

    if (!data.manager || !data.seat) {
        fprintf(stderr, "Could not find river_status_manager or wl_seat\n");
        return 1;
    }

    // Now bind the seat status
    data.seat_status = zriver_status_manager_v1_get_river_seat_status(data.manager, data.seat);
    zriver_seat_status_v1_add_listener(data.seat_status, &seat_status_listener, &data);

    // Wait for events
    while (!data.done && wl_display_dispatch(display) != -1);

    if (data.focused_title) {
        printf("%s\n", data.focused_title);
        free(data.focused_title);
    }

    if (data.seat_status) zriver_seat_status_v1_destroy(data.seat_status);
    if (data.manager) zriver_status_manager_v1_destroy(data.manager);
    if (data.seat) wl_seat_destroy(data.seat);
    wl_registry_destroy(registry);
    wl_display_disconnect(display);

    return 0;
}
