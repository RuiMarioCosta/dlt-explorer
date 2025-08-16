#include "adapter.h"
#include "dlt_parser.h"
#include "options.h"

int main(int argc, char *argv[]) {
  auto options = parse_cmd_options(argc, argv);

  if (options.v1) {
    parse_dlt_daemon(options.dlt_path);
  } else {
    parse_dlt_explorer(options.dlt_path);
  }
}
