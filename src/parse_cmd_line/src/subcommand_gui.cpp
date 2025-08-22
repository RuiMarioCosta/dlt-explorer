#include "parse_cmd_line.h"

#include <CLI/CLI.hpp>
#include <fmt/base.h>

#include <memory>


void setup_subcommand_gui(CLI::App &app) {
  auto options = std::make_shared<Options>();

  auto *sub = app.add_subcommand("gui", "Start GUI");
  sub->add_option("-p,--path", options->dlt_path, "DLT file path")->required()->check(CLI::ExistingFile);
  sub->add_option("-f,--filter", options->dlt_filter, "DLT filter path");

  sub->callback([options]() { fmt::println("Start GUI (not implemented yet)"); });
}

// void run_subcommand_gui() {}

// namespace po = boost::program_options;
//
// Options parse_cmd_options(int argc, char **argv) {
//   std::filesystem::path path;
//   std::filesystem::path filter;
//   bool version1{false};
//
//   po::options_description desc("Allowed options");
//   desc.add_options()("help", "Produce help message");
//   desc.add_options()("path", po::value<std::filesystem::path>(&path), "Path to DLT file");
//   desc.add_options()("filter", po::value<std::filesystem::path>(&filter), "Path to DLT Filter file");
//   desc.add_options()("v1", po::bool_switch(&version1), "Enable DLT v1 support");
//
//   po::variables_map var_map;
//   po::store(po::parse_command_line(argc, argv, desc), var_map);
//   po::notify(var_map);
//
//   if (var_map.contains("help")) {
//     std::cout << desc << "\n";
//     return {};
//   }
//
//   return Options{.dlt_path = path, .dlt_filter = filter, .v1 = version1};
// }
