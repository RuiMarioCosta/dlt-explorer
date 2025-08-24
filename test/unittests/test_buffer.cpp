#include "buffer.h"

#include <catch2/catch_test_macros.hpp>
#include <fmt/base.h>

#include <cstddef>
#include <string_view>
#include <utility>


using namespace std::string_view_literals;


struct TestType {
  TestType() { fmt::println("------ctor"); }
  TestType(TestType const & /*t*/) { fmt::println("------copy ctor"); }
  TestType &operator=(TestType const &rhs) {
    fmt::println("------copy assign");
    TestType temp = rhs;
    *this = std::move(temp);
    return *this;
  }
  TestType(TestType && /*t*/) noexcept { fmt::println("------move ctor"); }
  TestType &operator=(TestType && /*t*/) noexcept {
    fmt::println("------move assign");
    return *this;
  }
  ~TestType() { fmt::println("------dtor"); }

  static std::string_view print() { return "test type"; }
};

template<>
struct fmt::formatter<TestType> : formatter<string_view> {
  static auto format(const TestType &testType, format_context &ctx) {
    return fmt::format_to(ctx.out(), "{}", testType.print());
  }
};


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

    WHEN("the integer 12345 is stored") {
      int const value = 12345;
      auto result = buffer.store(value);

      THEN("characters '12345' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "12345");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the unsigned integer 12345 is stored") {
      unsigned int const value = 12345;
      auto result = buffer.store(value);

      THEN("characters '12345' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "12345");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the float 1.2345 is stored") {
      float const value = 1.2345F;
      auto result = buffer.store(value);

      THEN("characters '1.2345' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "1.2345");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the double 1.2345 is stored") {
      double const value = 1.2345;
      auto result = buffer.store(value);

      THEN("characters '1.2345' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "1.2345");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the hexadecimal 0x1a2b is stored") {
      int const value = 0x1a2b;
      auto result = buffer.store(Hex(value));

      THEN("characters '0x1a2b' are stored in Buffer") {
        REQUIRE(buffer.size() == 6);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "0x1a2b");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the binary 0b0101 is stored") {
      int const value = 0b0101;
      auto result = buffer.store(Bin(value));

      THEN("characters '0b101' are stored in Buffer") {
        REQUIRE(buffer.size() == 5);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "0b101");
      }
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the TestType is stored") {
      buffer.store(TestType{});

      THEN("no ctor are called") {}
    }
  }

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the std::string_view 'abc' is stored") {
      std::string_view const value{"abc"};
      auto result = buffer.store(Hex(value));

      THEN("characters '61 62 63' are stored in Buffer") {
        REQUIRE(buffer.size() == 8);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "61 62 63");
      }
    }
  }
}


SCENARIO("Buffer can store vector with multiple data") {

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the vector {123, 456} is stored") {
      std::vector<int> value{123, 456};
      auto result = buffer.store(value);

      THEN("characters '123 456' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "123 456");
      }
    }

    WHEN("the array {123, 456} is stored") {
      std::array<int, 2> value{123, 456};
      auto result = buffer.store(value);

      THEN("characters '123 456' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "123 456");
      }
    }

    WHEN("the array {'abc'sv, 'def'sv} is stored") {
      std::array<std::string_view, 2> value{"abc"sv, "def"sv};
      auto result = buffer.store(value);

      THEN("characters 'abc def' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "abc def");
      }
    }
  }
}


SCENARIO("Buffer can store multiple data") {

  GIVEN("A Buffer with big enough space") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};

    WHEN("the integers 123 and 456 are stored") {
      int const value1 = 123;
      int const value2 = 456;
      auto result = buffer.store(value1, value2);

      THEN("characters '123 456' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "123 456");
      }
    }

    WHEN("the string_views 'abc' and 'def' are stored") {
      std::string_view const value1 = "abc";
      std::string_view const value2 = "def";
      auto result = buffer.store(value1, value2);

      THEN("characters 'abc def' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "abc def");
      }
    }
  }
}


SCENARIO("Clearing vector between storing") {

  GIVEN("A Buffer with big enough space and a vector with reserved size 64") {
    size_t const capacity = 1024;
    Buffer buffer{capacity};
    std::vector<int> value;
    value.reserve(64);

    WHEN("the values 123 and 456 are stored") {
      value.emplace_back(123);
      value.emplace_back(456);
      auto result = buffer.store(value);

      THEN("characters '123 456' are stored in Buffer") {
        REQUIRE(buffer.size() == 7);
        REQUIRE(buffer.capacity() == 1024);
        REQUIRE(result == "123 456");
      }

      AND_WHEN("the vector is cleared and new data is added") {
        value.clear();
        value.emplace_back(111);
        value.emplace_back(222);
        auto result2 = buffer.store(value);

        THEN("characters '111 222' are stored in Buffer") {
          REQUIRE(buffer.size() == 14);
          REQUIRE(buffer.capacity() == 1024);
          REQUIRE(result2 == "111 222");
        }
      }
    }
  }
}
