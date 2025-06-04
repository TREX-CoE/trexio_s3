#ifndef __TREXIO_S3__
#define __TREXIO_S3__

#include <stdlib.h>
#include <stdint.h>

typedef void trexio_s3_client;

// Connect/disconnect
// Returns NULL if connection failed
trexio_s3_client* s3_connect();

void s3_disconnect(trexio_s3_client*);


// Check existance on server
// Returns 0 if file exists
int32_t s3_file_exists(const trexio_s3_client*,
                       const char* file_name,
                       size_t str_len);

// Returns the size of the file, or -1 upon failure
int64_t s3_size(const trexio_s3_client*,
                const char* file_name,
                size_t str_len);


// Get and put
// Both return 0 upon success, and -1 upon failure
int32_t s3_get(const trexio_s3_client*,
               const char* file_name,
               size_t str_len,
               char* buffer,
               size_t buffer_size);


int32_t s3_put(const trexio_s3_client*,
               const char* file_name,
               size_t str_len,
               const char* buffer,
               size_t buffer_size);


#endif

