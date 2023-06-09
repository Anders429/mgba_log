#include "runner.h"

#include <mgba/core/core.h>
#include <mgba/core/log.h>
#include <mgba/internal/gba/gba.h>
#include <mgba/internal/gba/io.h>
#include <stdio.h>

struct MGBA {
    struct mLogger logger;
    struct mCore* core;
    struct callback log_callback;
};

void log_catcher(struct mLogger* logger, int category, enum mLogLevel level, const char* format, va_list args) {
    // This is a safe cast, because the logger is the first entry in MGBA.
    struct MGBA* mgba = (struct MGBA*)logger;
    
    if (!strcmp(mLogCategoryName(category), "GBA Debug")) {
        int32_t size = 0;

        va_list args_copy;
        va_copy(args_copy, args);
        size += vsnprintf(NULL, 0, format, args_copy);

        // Account for null character.
        size += 1;

        char* str = calloc(size, sizeof(*str));
        vsnprintf(str, size, format, args);

        if (mgba->log_callback.callback != NULL) {
            mgba->log_callback.callback(mgba->log_callback.data, str, level);
        } else {
            printf("log_callback not set\n");
        }
 
        free(str);
    }
}

struct MGBA* load(char* rom) {
    struct MGBA* mgba = calloc(1, sizeof(struct MGBA));

    mgba->logger.log = log_catcher;
    mLogSetDefaultLogger(&mgba->logger);

    struct mCore* core = mCoreFind(rom);
    if (!core) {
        free(mgba);
        return NULL;
    }
    core->init(core);
    mCoreLoadFile(core, rom);
    mCoreConfigInit(&core->config, NULL);
    core->reset(core);
    mgba->core = core;

    return mgba;
}

void set_log_callback(struct MGBA* mgba, struct callback callback) {
    mgba->log_callback = callback;
}

bool is_finished(struct MGBA* mgba) {
    uint8_t status_register = ((uint8_t*)((struct GBA*)(mgba->core->board))->memory.wram)[0x3FFFF];
    return status_register == 3;
}

void step(struct MGBA* mgba) {
    mgba->core->step(mgba->core);
}

void drop(struct MGBA* mgba) {
    mgba->core->deinit(mgba->core);
    mgba->log_callback.destroy(mgba->log_callback.data);
    free(mgba);
}
