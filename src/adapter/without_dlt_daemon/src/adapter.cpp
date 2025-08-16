#include "adapter.h"


#include <filesystem>
#include <stdexcept>


void parse_dlt_daemon(std::filesystem::path const & /*path*/)
{
  throw std::runtime_error("not linked with dlt-daemon");
}
