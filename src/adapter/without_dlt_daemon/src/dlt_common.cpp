/*
 * Adaptation of dlt-daemon's dlt_common.c file to C++ with only some of the functions
 */
#include "dlt_common.h"
#include "dlt_types.h"


DltReturnValue dlt_check_storageheader(DltStorageHeader *storageheader) {
  if (storageheader == nullptr) { return DLT_RETURN_WRONG_PARAMETER; }

  return ((storageheader->pattern[0] == 'D') && (storageheader->pattern[1] == 'L') && (storageheader->pattern[2] == 'T')
           && (storageheader->pattern[3] == 1))
           ? DLT_RETURN_TRUE
           : DLT_RETURN_OK;
}
