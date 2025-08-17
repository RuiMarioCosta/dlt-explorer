#pragma once

#include "parse_cmd_line_export.h"

#include <CLI/CLI.hpp>
#include <fmt/base.h>
#include <fmt/format.h>

#include <string>


struct Options {
  std::string dlt_path;
  std::string dlt_filter;
  bool version1{false};
};

template<>
struct fmt::formatter<Options> : fmt::formatter<std::string> {
  template<typename FormatContext>
  auto format(const Options &opt, FormatContext &ctx) const {
    return fmt::formatter<std::string>::format(fmt::format("Options(dlt_path='{}', dlt_filter='{}', version1={})",
                                                 opt.dlt_path,
                                                 opt.dlt_filter,
                                                 opt.version1 ? "true" : "false"),
      ctx);
  }
};


void PARSE_CMD_LINE_EXPORT setup_subcommand_gui(CLI::App &app);
void PARSE_CMD_LINE_EXPORT setup_subcommand_term(CLI::App &app);
