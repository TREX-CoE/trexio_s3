#ifndef __TREXIO_S3__
#define __TREXIO_S3__

#include <stdint.h>

typedef void trexio_s3_client;

// Connect/disconnect
trexio_s3_client* s3_connect();

void s3_disconnect(trexio_s3_client*);


// Check existance on server

int32_t s3_file_exists(const trexio_s3_client*,
                       const char* file_name,
                       size_t str_len);

int64_t s3_size(const trexio_s3_client*,
                const char* file_name,
                size_t str_len);


// Get and put

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

