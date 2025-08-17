#include <filesystem>

struct Options {
  std::filesystem::path dlt_path;
  std::filesystem::path dlt_filter;
  bool v1;
};

Options parse_cmd_options(int argc, char *argv[]);
