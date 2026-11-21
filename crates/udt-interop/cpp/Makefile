# Build thin C++ UDT stream peers against vendored libudt.
# From repo root:
#   ./crates/udt-interop/scripts/build_vendor.sh
#   make -C crates/udt-interop/cpp

UDT_ROOT ?= ../vendor/udt/udt4
SRC_DIR  := $(UDT_ROOT)/src

UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
  OS ?= OSX
  # Avoid -DAMD64 (rdtsc) on Apple Silicon; OSX path uses mach_absolute_time.
  ARCH ?= ARM_BASE
else
  OS ?= LINUX
  ifeq ($(UNAME_M),x86_64)
    ARCH ?= AMD64
  else
    ARCH ?= ARM_BASE
  endif
endif

# Prefer static libudt.a so peers run without DYLD/LD_LIBRARY_PATH.
LIBUDT_A := $(SRC_DIR)/libudt.a

CXX ?= g++
CXXFLAGS := -Wall -D$(OS) -I$(SRC_DIR) -O2
LDFLAGS := $(LIBUDT_A) -lpthread -lm

.PHONY: all clean

all: stream_server stream_client

stream_server: stream_server.cpp $(LIBUDT_A)
	$(CXX) $(CXXFLAGS) $< -o $@ $(LDFLAGS)

stream_client: stream_client.cpp $(LIBUDT_A)
	$(CXX) $(CXXFLAGS) $< -o $@ $(LDFLAGS)

$(LIBUDT_A):
	$(MAKE) -C $(SRC_DIR) os=$(OS) arch=$(ARCH)

clean:
	rm -f stream_server stream_client stream_server_static stream_client_static
