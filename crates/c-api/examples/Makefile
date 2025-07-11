# Makefile for vexy_json C++ examples

CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -O2
LDFLAGS = -L../../../target/release -lvexy_json
INCLUDES = -I../include

# For macOS, add rpath to find the library
ifeq ($(shell uname),Darwin)
    LDFLAGS += -Wl,-rpath,@executable_path/../../../target/release
endif

all: cpp_example

cpp_example: cpp_example.cpp
	$(CXX) $(CXXFLAGS) $(INCLUDES) $< $(LDFLAGS) -o $@

run: cpp_example
	./cpp_example

clean:
	rm -f cpp_example

.PHONY: all run clean