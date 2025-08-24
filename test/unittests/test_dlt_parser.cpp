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


SCENARIO("DLT constructor") {
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
    std::vector<std::string_view> expected_payloads{""sv,
      "21"sv,
      "31 32"sv,
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
}
