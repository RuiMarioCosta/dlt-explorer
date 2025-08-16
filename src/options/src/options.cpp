#include "options.h"

#include <boost/program_options/options_description.hpp>
#include <boost/program_options/parsers.hpp>
#include <boost/program_options/value_semantic.hpp>
#include <boost/program_options/variables_map.hpp>

#include <filesystem>
#include <iostream>

namespace po = boost::program_options;

Options parse_cmd_options(int argc, char **argv)
{
  std::filesystem::path path;
  std::filesystem::path filter;
  bool version1{ false };

  po::options_description desc("Allowed options");
  desc.add_options()("help", "Produce help message");
  desc.add_options()("path", po::value<std::filesystem::path>(&path), "Path to DLT file");
  desc.add_options()("filter", po::value<std::filesystem::path>(&filter), "Path to DLT Filter file");
  desc.add_options()("v1", po::bool_switch(&version1), "Enable DLT v1 support");

  po::variables_map var_map;
  po::store(po::parse_command_line(argc, argv, desc), var_map);
  po::notify(var_map);

  if (var_map.contains("help")) {
    std::cout << desc << "\n";
    return {};
  }

  return Options{ .dlt_path = path, .dlt_filter = filter, .v1 = version1 };
}
