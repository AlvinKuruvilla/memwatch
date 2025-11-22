/**
 * C Helper Library Header
 */

#ifndef LIB_HELPER_H
#define LIB_HELPER_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Validate data checksum
 */
int validate_checksum(const unsigned char* data, size_t len);

/**
 * Process large dataset
 */
void process_large_dataset(void);

/**
 * Library initialization
 */
void lib_helper_init(void);

#ifdef __cplusplus
}
#endif

#endif /* LIB_HELPER_H */
