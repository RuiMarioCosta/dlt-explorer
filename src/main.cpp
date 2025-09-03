#include "adapter.h"
#include "dlt_parser.h"

#include <internal_use_only/config.hpp>

#include <CLI/CLI.hpp>
#include <fmt/base.h>
#include <fmt/format.h>

#include <exception>
#include <memory>
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


void setup_subcommand_term(CLI::App &app) {
  auto options = std::make_shared<Options>();

  auto *sub = app.add_subcommand("term", "Run on terminal");
  sub->add_option("-p,--path", options->dlt_path, "DLT file path")->required()->check(CLI::ExistingFile);
  sub->add_option("-f,--filter", options->dlt_filter, "DLT filter path");
  sub->add_flag("--v1", options->version1, "Parse using dlt-daemon lib");

  sub->callback([options]() {
    fmt::println("{:#>80}", '#');
    fmt::println("{}", *options);
    fmt::println("{:#>80}", '#');

    if (options->version1) {
      parse_dlt_daemon(options->dlt_path);
    } else {
      DLT const dlt{options->dlt_path};
      fmt::println("{}", dlt.payloads());
    }
  });
}

void setup_subcommand_gui(CLI::App &app) {
  auto options = std::make_shared<Options>();

  auto *sub = app.add_subcommand("gui", "Start GUI");
  sub->add_option("-p,--path", options->dlt_path, "DLT file path");
  sub->add_option("-f,--filter", options->dlt_filter, "DLT filter path");

  sub->callback([options]() {
    fmt::println("----------------");
    fmt::println("Start GUI (not implemented yet)");
  });
}


int main(int argc, char *argv[]) {
  try {
    CLI::App app{fmt::format("App for visualizing and exploring DLT files.\n\n{} version {}",
      dlt_explorer::cmake::project_name,
      dlt_explorer::cmake::project_version)};

    setup_subcommand_term(app);
    setup_subcommand_gui(app);

    app.require_subcommand(1);

    CLI11_PARSE(app, argc, argv);

  } catch (const std::exception &e) { fmt::println("Unhandled exception in main: {}", e.what()); }
}
