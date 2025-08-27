#include "dlt_parser.h"

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers.hpp>
#include <catch2/matchers/catch_matchers_quantifiers.hpp>
#include <catch2/matchers/catch_matchers_range_equals.hpp>
#include <catch2/matchers/catch_matchers_templated.hpp>

#include <filesystem>
#include <ranges>
#include <string>
#include <string_view>
#include <vector>

using namespace std::string_view_literals;
using namespace Catch::Matchers;


struct EqualsStringViewMatcher : Catch::Matchers::MatcherGenericBase {
  explicit EqualsStringViewMatcher(std::string_view value) : value{value} {}

  bool match(std::string_view const &other) const { return value == other; }

  std::string describe() const override { return "Equals: " + std::string{value}; }

private:
  std::string_view value;
};

auto Equals(const std::string_view &value) -> EqualsStringViewMatcher { return EqualsStringViewMatcher{value}; }


// NOLINTNEXTLINE(readability-function-cognitive-complexity)
SCENARIO("DLT constructor", "[dlt]") {
  std::filesystem::path const DATA_PATH = DATA_DIR;

  GIVEN("A DLT file with control messages") {
    std::filesystem::path const path{DATA_PATH / "testfile_control_messages.dlt"};

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        REQUIRE_THAT(dlt.app_ids(), AllMatch(Equals("APP\x00"sv)));
        REQUIRE_THAT(dlt.ctx_ids(), AllMatch(Equals("CON\x00"sv)));
        REQUIRE_THAT(dlt.payloads(),
          RangeEquals({"set_default_log_level 04 72 65 6d 6f",
            "set_default_trace_status 00 72 65 6d 6f",
            "set_verbose_mode 01",
            "set_timing_packets 00"}));
        REQUIRE(dlt.size() == 4);
      }
    }
  }

  GIVEN("A DLT file with messages with empty, a number and text") {
    std::filesystem::path const path{DATA_PATH / "testfile_empty_number_text.dlt"};

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        REQUIRE_THAT(dlt.app_ids(), AllMatch(Equals("LOG\x00"sv)));
        REQUIRE_THAT(dlt.ctx_ids(), AllMatch(Equals("TES1"sv)));
        REQUIRE_THAT(dlt.payloads(), RangeEquals({""sv, "1011"sv, "Hello BMW\x00"sv}));
        REQUIRE(dlt.size() == 3);
      }
    }
  }

  GIVEN("A DLT file of messages with types int, string, cstring and float") {
    std::filesystem::path const path{DATA_PATH / "testfile_single_payloads.dlt"};
    std::vector<std::string_view> expected_payloads{"101"sv,
      "102"sv,
      "103"sv,
      "104"sv,
      "105"sv,
      "106"sv,
      "107"sv,
      "108"sv,
      "109"sv,
      "110"sv,
      "true"sv,
      "STRING 112 message\x00"sv,
      "CSTRING 113 message\x00"sv,
      "1.1"sv,
      "1.2"sv,
      "48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv};

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        REQUIRE_THAT(dlt.app_ids(), AllMatch(Equals("LOG\x00"sv)));
        REQUIRE_THAT(dlt.ctx_ids(), AllMatch(Equals("TES2"sv)));
        for (auto const [payload, expected] : std::views::zip(dlt.payloads(), expected_payloads)) {
          REQUIRE(payload == expected);
        }
      }
    }
  }

  GIVEN("A DLT file of messages with zero, one and more number of arguments") {
    std::filesystem::path const path{DATA_PATH / "testfile_multiple_number_of_arguments.dlt"};
    std::vector<std::string_view> expected_payloads{
      ""sv,
      "21"sv,
      "31 32"sv,
      "41 42 43"sv,
      "51 52 53 54"sv,
      "61 62 63 64 65"sv,
      "71 72 73 74 75 76"sv,
    };

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        REQUIRE_THAT(dlt.app_ids(), AllMatch(Equals("LOG\x00"sv)));
        REQUIRE_THAT(dlt.ctx_ids(), AllMatch(Equals("TES3"sv)));
        for (auto const [payload, expected] : std::views::zip(dlt.payloads(), expected_payloads)) {
          REQUIRE(payload == expected);
        }
      }
    }
  }

  GIVEN("A DLT file of number and text messages") {
    std::filesystem::path const path{DATA_PATH / "testfile_number_and_text.dlt"};
    std::vector<std::string_view> expected_payloads{
      "0 Hello world\x00"sv,
      "1 Hello world\x00"sv,
      "2 Hello world\x00"sv,
      "3 Hello world\x00"sv,
      "4 Hello world\x00"sv,
      "5 Hello world\x00"sv,
      "6 Hello world\x00"sv,
      "7 Hello world\x00"sv,
      "8 Hello world\x00"sv,
      "9 Hello world\x00"sv,
      "10 Hello world\x00"sv,
      "11 Hello world\x00"sv,
      "12 Hello world\x00"sv,
      "13 Hello world\x00"sv,
      "14 Hello world\x00"sv,
      "15 Hello world\x00"sv,
      "16 Hello world\x00"sv,
      "17 Hello world\x00"sv,
    };

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        REQUIRE_THAT(dlt.app_ids(), AllMatch(Equals("LOG\x00"sv)));
        REQUIRE_THAT(dlt.ctx_ids(), AllMatch(Equals("TES4"sv)));
        for (auto const [payload, expected] : std::views::zip(dlt.payloads(), expected_payloads)) {
          REQUIRE(payload == expected);
        }
      }
    }
  }

  GIVEN("A DLT file of ") {
    std::filesystem::path const path{DATA_PATH / "testfile_type_id_and_text.dlt"};
    std::vector<std::string_view> expected_payloads{
      "set_default_log_level 04 72 65 6d 6f"sv,
      "set_default_trace_status 00 72 65 6d 6f"sv,
      "set_verbose_mode 01"sv,
      "set_timing_packets 00"sv,
      "101 "sv,
      "102 f3 03"sv,
      "103 0a 00 48 65 6c 6c 6f 20 42 4d 57 00"sv,
      "201 65"sv,
      "202 66 00"sv,
      "203 67 00 00 00"sv,
      "204 68 00 00 00 00 00 00 00"sv,
      "205 69"sv,
      "206 6a 00"sv,
      "207 6b 00 00 00"sv,
      "208 6c 00 00 00 00 00 00 00"sv,
      "209 6d 00 00 00"sv,
      "210 6e 00 00 00"sv,
      "211 6f"sv,
      "212 13 00 53 54 52 49 4e 47 20 31 31 32 20 6d 65 73 73 61 67 65 00"sv,
      "213 14 00 43 53 54 52 49 4e 47 20 31 31 33 20 6d 65 73 73 61 67 65 00"sv,
      "214 cd cc 8c 3f"sv,
      "215 33 33 33 33 33 33 f3 3f"sv,
      "216 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "301 "sv,
      "302 15 00 00 00"sv,
      "303 1f 00 00 00 20 00 00 00"sv,
      "304 29 00 00 00 2a 00 00 00 2b 00 00 00"sv,
      "305 33 00 00 00 34 00 00 00 35 00 00 00 36 00 00 00"sv,
      "305 3d 00 00 00 3e 00 00 00 3f 00 00 00 40 00 00 00 41 00 00 00"sv,
      "305 47 00 00 00 48 00 00 00 49 00 00 00 4a 00 00 00 4b 00 00 00 4c 00 00 00"sv,
      "401 00 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 01 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 02 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 03 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 04 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 05 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 06 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 07 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 08 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 09 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0a 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0b 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0c 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0d 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0e 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 0f 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 10 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 11 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 12 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 13 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 14 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 15 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 16 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 17 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 18 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 19 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
      "401 1a 00 00 00 0c 00 48 65 6c 6c 6f 20 77 6f 72 6c 64 00"sv,
    };

    WHEN("constuctor is called") {
      DLT const dlt{path};

      THEN("data is correct") {
        REQUIRE_THAT(dlt.patterns(), AllMatch(Equals("DLT\001"sv)));
        for (auto const [payload, expected] : std::views::zip(dlt.payloads(), expected_payloads)) {
          REQUIRE(payload == expected);
        }
      }
    }
  }
}
