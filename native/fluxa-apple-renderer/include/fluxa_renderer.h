#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef struct FluxaRenderer FluxaRenderer;

static const uint32_t FLUXA_POINTER_MOVE = 0;
static const uint32_t FLUXA_POINTER_DOWN = 1;
static const uint32_t FLUXA_POINTER_UP = 2;

static const uint32_t FLUXA_KEY_UP = 1;
static const uint32_t FLUXA_KEY_DOWN = 2;
static const uint32_t FLUXA_KEY_LEFT = 3;
static const uint32_t FLUXA_KEY_RIGHT = 4;
static const uint32_t FLUXA_KEY_ENTER = 5;
static const uint32_t FLUXA_KEY_BACK = 6;
static const uint32_t FLUXA_KEY_ESCAPE = 7;
static const uint32_t FLUXA_KEY_TAB = 8;
static const uint32_t FLUXA_KEY_SHIFT_TAB = 9;
static const uint32_t FLUXA_KEY_BACKSPACE = 10;
static const uint32_t FLUXA_GAMEPAD_SOUTH = 20;
static const uint32_t FLUXA_GAMEPAD_EAST = 21;
static const uint32_t FLUXA_GAMEPAD_WEST = 22;
static const uint32_t FLUXA_GAMEPAD_NORTH = 23;
static const uint32_t FLUXA_GAMEPAD_DPAD_UP = 24;
static const uint32_t FLUXA_GAMEPAD_DPAD_DOWN = 25;
static const uint32_t FLUXA_GAMEPAD_DPAD_LEFT = 26;
static const uint32_t FLUXA_GAMEPAD_DPAD_RIGHT = 27;
static const uint32_t FLUXA_GAMEPAD_START = 28;
static const uint32_t FLUXA_GAMEPAD_SELECT = 29;

FluxaRenderer *fluxa_renderer_create(float density, const char *artwork_cache_dir);
bool fluxa_renderer_start_session(const FluxaRenderer *renderer, const char *data_dir);
void fluxa_renderer_push_action(const FluxaRenderer *renderer, const char *json);
bool fluxa_renderer_back(const FluxaRenderer *renderer);
char *fluxa_renderer_poll_video(void);
void fluxa_renderer_video_status(double position, double duration, bool paused, bool has_frame, float buffering, const char *error);
void fluxa_renderer_destroy(FluxaRenderer *renderer);
void fluxa_renderer_string_free(char *value);

void fluxa_renderer_surface_created(const FluxaRenderer *renderer, void *metal_layer, uint32_t width, uint32_t height);
void fluxa_renderer_surface_changed(const FluxaRenderer *renderer, uint32_t width, uint32_t height);
void fluxa_renderer_surface_destroyed(const FluxaRenderer *renderer);
void fluxa_renderer_render(const FluxaRenderer *renderer);

void fluxa_renderer_set_safe_bottom_inset(const FluxaRenderer *renderer, float inset);
void fluxa_renderer_set_form_factor(const FluxaRenderer *renderer, const char *form_factor);
void fluxa_renderer_set_home_state(const FluxaRenderer *renderer, const char *json);
void fluxa_renderer_set_core_snapshot(const FluxaRenderer *renderer, const char *json);
char *fluxa_renderer_snapshot(const FluxaRenderer *renderer);
char *fluxa_renderer_poll_actions(const FluxaRenderer *renderer);

void fluxa_renderer_scroll(const FluxaRenderer *renderer, float delta_y);
void fluxa_renderer_pointer(const FluxaRenderer *renderer, uint32_t phase, float x, float y);
void fluxa_renderer_key_down(const FluxaRenderer *renderer, uint32_t key);
void fluxa_renderer_text_input(const FluxaRenderer *renderer, const char *text);
int64_t fluxa_renderer_focused_node(const FluxaRenderer *renderer);
void fluxa_renderer_focus_node(const FluxaRenderer *renderer, uint64_t node);
void fluxa_renderer_activate_node(const FluxaRenderer *renderer, uint64_t node);
