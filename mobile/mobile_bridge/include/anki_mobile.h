// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

#ifndef ANKI_MOBILE_H
#define ANKI_MOBILE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AnkiMobileBackend AnkiMobileBackend;

enum {
    ANKI_MOBILE_OK = 0,
    ANKI_MOBILE_BACKEND_ERROR = 1,
    ANKI_MOBILE_INVALID_INPUT = 2,
    ANKI_MOBILE_PANIC = 3,
};

const char *anki_mobile_buildhash(void);

int anki_mobile_backend_create(
    const uint8_t *init_msg,
    size_t init_len,
    AnkiMobileBackend **out_backend);

void anki_mobile_backend_destroy(AnkiMobileBackend *backend);

int anki_mobile_backend_command(
    AnkiMobileBackend *backend,
    uint32_t service,
    uint32_t method,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

void anki_mobile_bytes_free(uint8_t *ptr, size_t len);

int anki_mobile_last_error(const char **out);

#ifdef __cplusplus
}
#endif

#endif
