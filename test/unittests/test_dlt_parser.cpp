#include "dlt_parser.h"

#include <catch2/catch_test_macros.hpp>

#include <filesystem>

SCENARIO("DLT constructor") {
  GIVEN("A DLT file with control messages") {
    std::filesystem::path const path{"testfile.dlt"};

    WHEN("constuctor is called") {
      DLT const buffer{path};

      THEN("size and capacity change") {
        // REQUIRE(buffer.size() == 0);
        // REQUIRE(buffer.capacity() == 1);
      }
    }
  }
}
