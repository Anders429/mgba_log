#include <stdbool.h>
#include <stdint.h>

struct MGBA;

struct MGBA* load(char* rom);

struct callback {
    void* data;
    void (*callback)(void*, char[], uint8_t);
    void (*destroy)(void*);
};

void set_log_callback(struct MGBA* mgba, struct callback callback);

bool is_finished(struct MGBA* mgba);

void step(struct MGBA* mgba);

void drop(struct MGBA* mgba);
