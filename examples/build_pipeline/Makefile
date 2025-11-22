# Build Pipeline Example - Multi-Program Memory Profiling
#
# This Makefile orchestrates a realistic build pipeline that spawns multiple
# programs across different languages:
# - Python code generation
# - C library compilation (cc)
# - Rust compilation (spawns rustc, cc, ld)
# - Node.js testing
# - Perl data validation
# - Python packaging
#
# Usage:
#   make                    # Run full pipeline (sequential)
#   make SIZE=large         # Run with large problem size
#   make all-parallel       # Run with parallel builds
#   make clean              # Clean generated files
#
# Environment Variables:
#   SIZE      Problem size: small (default), medium, large
#   JOBS      Parallel job count for compilation (default: 4)
#   PROFILE   Rust build profile: debug (default), release

.PHONY: all all-parallel codegen build-c build test validate package clean help

# Default configuration
SIZE ?= small
JOBS ?= 4
PROFILE ?= debug

# Directory structure
SCRIPT_DIR := scripts
SRC_DIR := src
TARGET_DIR := target
DIST_DIR := dist

# Detect platform
UNAME := $(shell uname)

# Binary names
ifeq ($(PROFILE),release)
	BUILD_FLAGS := --release
	BINARY := $(TARGET_DIR)/release/build_example
else
	BUILD_FLAGS :=
	BINARY := $(TARGET_DIR)/debug/build_example
endif

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

# Default target: sequential pipeline
all: banner codegen build-c build test validate package success

# Parallel target: run with make -j
all-parallel: banner
	@echo "$(CYAN)Running parallel build pipeline with $(JOBS) jobs...$(RESET)"
	@$(MAKE) -j $(JOBS) codegen build-c build test validate package
	@$(MAKE) success

banner:
	@echo "$(CYAN)========================================$(RESET)"
	@echo "$(CYAN)  Build Pipeline Example for memwatch  $(RESET)"
	@echo "$(CYAN)========================================$(RESET)"
	@echo ""
	@echo "Configuration:"
	@echo "  SIZE:    $(SIZE)"
	@echo "  PROFILE: $(PROFILE)"
	@echo "  JOBS:    $(JOBS)"
	@echo "  Platform: $(UNAME)"
	@echo ""

# Phase 1: Code Generation (Python)
codegen:
	@echo "$(YELLOW)Phase 1: Code Generation$(RESET)"
	@echo "Generating Rust source files with SIZE=$(SIZE)..."
	@python3 $(SCRIPT_DIR)/codegen.py --size $(SIZE) --output $(SRC_DIR)
	@echo "$(GREEN)✓ Code generation complete$(RESET)"
	@echo ""

# Phase 2: C Library Compilation (cc)
build-c: codegen
	@echo "$(YELLOW)Phase 2: C Library Compilation$(RESET)"
	@echo "Compiling C helper library..."
	@mkdir -p $(TARGET_DIR)/c_lib
	@cc -c $(SRC_DIR)/lib_helper.c -o $(TARGET_DIR)/c_lib/lib_helper.o -I$(SRC_DIR) -O2
	@ar rcs $(TARGET_DIR)/c_lib/libhelper.a $(TARGET_DIR)/c_lib/lib_helper.o
	@echo "$(GREEN)✓ C library compilation complete$(RESET)"
	@echo ""

# Phase 3: Rust Compilation (cargo → rustc → cc → ld)
build: build-c
	@echo "$(YELLOW)Phase 3: Rust Compilation$(RESET)"
	@echo "Building Rust project ($(PROFILE) mode)..."
	@echo "This will spawn: cargo → rustc → cc → ld"
	@cargo build $(BUILD_FLAGS) --manifest-path Cargo.toml
	@echo "$(GREEN)✓ Rust compilation complete$(RESET)"
	@echo ""

# Phase 4: Testing (Node.js)
test: build
	@echo "$(YELLOW)Phase 4: Testing$(RESET)"
	@echo "Running tests with Node.js (SIZE=$(SIZE))..."
	@node $(SCRIPT_DIR)/run_tests.js --size $(SIZE) --binary $(BINARY)
	@echo "$(GREEN)✓ Tests complete$(RESET)"
	@echo ""

# Phase 5: Data Validation (Perl)
validate: test
	@echo "$(YELLOW)Phase 5: Data Validation$(RESET)"
	@echo "Running data validation with Perl..."
	@perl $(SCRIPT_DIR)/validate_data.pl --size $(SIZE) --output $(DIST_DIR)
	@echo "$(GREEN)✓ Validation complete$(RESET)"
	@echo ""

# Phase 6: Packaging (Python)
package: validate
	@echo "$(YELLOW)Phase 6: Packaging$(RESET)"
	@echo "Creating distribution package..."
	@mkdir -p $(DIST_DIR)
	@python3 $(SCRIPT_DIR)/package.py --size $(SIZE) --binary $(BINARY) --output $(DIST_DIR)
	@echo "$(GREEN)✓ Packaging complete$(RESET)"
	@echo ""

success:
	@echo "$(GREEN)========================================$(RESET)"
	@echo "$(GREEN)  Pipeline complete!$(RESET)"
	@echo "$(GREEN)========================================$(RESET)"
	@echo ""
	@echo "To profile this pipeline with memwatch:"
	@echo "  memwatch run -- make SIZE=$(SIZE)"
	@echo "  memwatch run --timeline timeline.csv -- make SIZE=large"
	@echo "  memwatch run --exclude 'sh|make' -- make"
	@echo ""

# Clean generated files
clean:
	@echo "Cleaning generated files..."
	@rm -rf $(SRC_DIR)/*.rs
	@rm -rf $(TARGET_DIR)
	@rm -rf $(DIST_DIR)
	@echo "$(GREEN)✓ Clean complete$(RESET)"

# Help target
help:
	@echo "Build Pipeline Example - Makefile Targets"
	@echo ""
	@echo "Targets:"
	@echo "  all           Run full pipeline (sequential, 6 phases)"
	@echo "  all-parallel  Run full pipeline (parallel)"
	@echo "  codegen       Run code generation only (Python)"
	@echo "  build-c       Run C library compilation (cc)"
	@echo "  build         Run Rust compilation (requires build-c)"
	@echo "  test          Run tests (Node.js, requires build)"
	@echo "  validate      Run data validation (Perl, requires test)"
	@echo "  package       Run packaging (Python, requires validate)"
	@echo "  clean         Remove generated files"
	@echo "  help          Show this help message"
	@echo ""
	@echo "Languages Used:"
	@echo "  Phase 1: Python (code generation)"
	@echo "  Phase 2: C (library compilation with cc)"
	@echo "  Phase 3: Rust (cargo → rustc → cc → ld)"
	@echo "  Phase 4: Node.js (testing)"
	@echo "  Phase 5: Perl (data validation)"
	@echo "  Phase 6: Python (packaging)"
	@echo ""
	@echo "Environment Variables:"
	@echo "  SIZE          Problem size: small, medium, large (default: small)"
	@echo "  PROFILE       Build profile: debug, release (default: debug)"
	@echo "  JOBS          Parallel jobs for compilation (default: 4)"
	@echo ""
	@echo "Examples:"
	@echo "  make                              # Sequential, small size"
	@echo "  make SIZE=large                   # Large problem size"
	@echo "  make all-parallel JOBS=8          # Parallel with 8 jobs"
	@echo "  make PROFILE=release              # Release build"
	@echo "  memwatch run -- make SIZE=medium  # Profile with memwatch"
	@echo ""
