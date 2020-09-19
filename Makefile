# A Makefile to bundle the project into an executable
# Completely unecessary, but I used this as an excuse to finally learn how to use Makefiles properly

ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SILENT := 2> /dev/null
M := @make -s
C := @cargo

# Build a release edition of the project
all: clean make_dir
	$(C) build --release $(SILENT)
	@mv target/release/anvil.exe dist

	@printf "Built release version successfully"

# Build a version for testing purposes
debug: clean make_dir
	$(C) build $(SILENT)
	@mv target/debug/anvil.exe dist

	@printf "Built debugging version successfully"

# Clean the dist/ folder
clean:
	-@rm dist/*.* $(SILENT)

# Make the required dist directory if not present
make_dir:
	@if [ ! -d "./dist" ]; then mkdir ./dist; fi