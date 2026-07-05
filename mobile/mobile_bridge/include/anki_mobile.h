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

int anki_mobile_open_collection(
    AnkiMobileBackend *backend,
    const char *collection_path,
    const char *media_folder_path,
    const char *media_db_path,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_backend_command(
    AnkiMobileBackend *backend,
    uint32_t service,
    uint32_t method,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_dashboard_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_progress_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_practice_bootstrap_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_record_attempt_json(
    AnkiMobileBackend *backend,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_explain_answer_json(
    AnkiMobileBackend *backend,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_practice_scores_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_study_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_study_review_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_gre_study_answer_json(
    AnkiMobileBackend *backend,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_prepare_demo_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_brainlift_sync_status_json(
    AnkiMobileBackend *backend,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_brainlift_sync_pull_json(
    AnkiMobileBackend *backend,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_brainlift_sync_push_json(
    AnkiMobileBackend *backend,
    const uint8_t *input,
    size_t input_len,
    uint8_t **out_bytes,
    size_t *out_len);

int anki_mobile_brainlift_sync_perform_json(
    AnkiMobileBackend *backend,
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
