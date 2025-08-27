#include "buffer.h"

#include <catch2/catch_test_macros.hpp>
#include <fmt/base.h>
#include <fmt/format.h>

#include <cstddef>
#include <string_view>
#include <vector>


using namespace std::string_view_literals;


SCENARIO("Format Hex type") {

  GIVEN("A Hex<int>") {
    Hex value{1};

    WHEN("fmt::format is called") {
      auto result = fmt::format("{}", value);

      THEN("return the hex representation of the underlying type") { REQUIRE(result == "0x1"); }
    }
  }

  GIVEN("A Hex<std::string_view>") {
    Hex value{"abc"sv};

    WHEN("fmt::format is called") {
      auto result = fmt::format("{}", value);

      THEN("return the hex representation of the underlying type") { REQUIRE(result == "61 62 63"); }
    }
  }
}

SCENARIO("Buffer constructor") {
  GIVEN("A capacity of 1") {
    size_t const capacity{1};

    WHEN("constuctor is called") {
      Buffer const buffer{capacity};

      THEN("size and capacity change") {
        REQUIRE(buffer.size() == 0);
        REQUIRE(buffer.capacity() == 1);
      }
    }
  }

  GIVEN("A capacity of 64") {
    size_t const capacity{64};

    WHEN("constuctor is called") {
      Buffer const buffer{capacity};

      THEN("size and capacity change") {
        REQUIRE(buffer.size() == 0);
        REQUIRE(buffer.capacity() == 64);
      }
    }
  }
}


SCENARIO("Buffer can store data") {

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};
    auto begin = buffer.begin();

    WHEN("the integer 12345 is stored") {
      int const value = 12345;
      auto end = buffer.store(value);
      std::string_view result{begin, end};

      THEN("characters '12345' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "12345");
      }
    }

    WHEN("the unsigned integer 12345 is stored") {
      unsigned int const value = 12345;
      auto end = buffer.store(value);
      std::string_view result{begin, end};

      THEN("characters '12345' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "12345");
      }
    }

    WHEN("the float 1.2345 is stored") {
      float const value = 1.2345F;
      auto end = buffer.store(value);
      std::string_view result{begin, end};

      THEN("characters '1.2345' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "1.2345");
      }
    }

    WHEN("the double 1.2345 is stored") {
      double const value = 1.2345;
      auto end = buffer.store(value);
      std::string_view result{begin, end};

      THEN("characters '1.2345' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "1.2345");
      }
    }

    WHEN("the hexadecimal 0x1a2b is stored") {
      int const value = 0x1a2b;
      auto end = buffer.store(Hex(value));
      std::string_view result{begin, end};

      THEN("characters '0x1a2b' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "0x1a2b");
      }
    }

    WHEN("the binary 0b0101 is stored") {
      int const value = 0b0101;
      auto end = buffer.store(Bin(value));
      std::string_view result{begin, end};

      THEN("characters '0b101' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "0b101");
      }
    }

    WHEN("the std::string_view 'abc' is stored") {
      std::string_view const value{"abc"};
      auto end = buffer.store(Hex(value));
      std::string_view result{begin, end};

      THEN("characters '61 62 63' are stored in Buffer") {
        REQUIRE(buffer.size() == 8);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "61 62 63");
      }
    }

    WHEN("the std::vector '{1,2,3}' is stored") {
      std::vector<int> const value{1, 2, 3};
      auto end = buffer.store(value);
      std::string_view result{begin, end};

      THEN("characters '1 2 3' are stored in Buffer") {
        REQUIRE(buffer.size() == 9);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "[1, 2, 3]");
      }
    }

    WHEN("two integers are stored") {
      int const value1 = 123;
      int const value2 = 456;
      auto end = buffer.store(value1, value2);
      std::string_view result{begin, end};

      THEN("characters '123 456' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "123 456");
      }
    }
  }
}
