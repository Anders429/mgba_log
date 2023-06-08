// These are bindings to mGBA's core itself.
//
// Defined here are all of the structs and functions necessary for the binary to set up and
// interact directly with an mGBA instance.

#include <stdbool.h>
#include <stdint.h>

struct MGBA;

// Creates a new instance of mGBA, loaded with the provided ROM.
struct MGBA* load(char* rom);

struct callback {
    void* data;
    void (*callback)(void*, char[], uint8_t);
    void (*destroy)(void*);
};

// Sets a function to be called when logs are received.
void set_log_callback(struct MGBA* mgba, struct callback callback);

// Reports whether the ROM processing has finished.
//
// This is reported by the ROM itself by writing the value `3` to `0x0203FFFF`.
bool is_finished(struct MGBA* mgba);

// Advance emulation by a single step.
void step(struct MGBA* mgba);

// Free the mGBA instance.
void drop(struct MGBA* mgba);
