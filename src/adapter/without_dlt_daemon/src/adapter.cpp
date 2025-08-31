#include "adapter.h"

#include <filesystem>
#include <stdexcept>


void parse_dlt_daemon(std::filesystem::path const & /*path*/) {
  throw std::runtime_error("v1 not possible because it was not linked with dlt-daemon");
}
