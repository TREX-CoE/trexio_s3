#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

#include <trexio_s3.h>

int main() {
  int rc;

  trexio_s3_client* client = s3_connect();

  char* file_name = "data/test_c";
  char* data = "Hello World in C source file!\n";
  rc = s3_put(client, file_name, strlen(file_name), data, strlen(data));
  assert (rc == 0);

  rc = s3_file_exists(client, file_name, strlen(file_name));
  assert (rc == 0);

  size_t size = s3_size(client, file_name, strlen(file_name));
  char* data_read = malloc(size * sizeof(char));
  rc = s3_get(client, file_name, strlen(file_name), data_read, size);
  assert(rc == 0);

  printf("'%s'\n", data_read);


  free(data_read);

}
