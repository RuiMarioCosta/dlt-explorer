#include "adapter.h"

#include "dlt_common.h"

#include <array>
#include <filesystem>

constexpr int DLT_DAEMON_TEXTSIZE = 10024;

void parse_dlt_daemon(std::filesystem::path const &path) {
  DltFile file;
  int const verbose{0};
  static std::array<char, DLT_DAEMON_TEXTSIZE> text;
  // static char text[DLT_DAEMON_TEXTSIZE];

  /* Normal Use-Case, expected 0 */
  dlt_log_init(0);
  dlt_file_init(&file, 0);
  dlt_file_open(&file, path.c_str(), 0);

  while (dlt_file_read(&file, 0) >= 0) {}

  for (int i = 0; i < file.counter; i++) {
    dlt_file_message(&file, i, 0);
    dlt_message_print_ascii(&file.msg, text.data(), DLT_DAEMON_TEXTSIZE, verbose);
  }

  dlt_file_free(&file, 0);
}
