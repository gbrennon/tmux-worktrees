.PHONY: test test-verbose test-filter test-tap

BATS := $(if $(shell command -v bats 2>/dev/null),bats,./test/bats/bin/bats)
TEST_DIR := test
FILTER ?=

test:
	$(BATS) $(TEST_DIR)/

test-verbose:
	$(BATS) --verbose-run $(TEST_DIR)/

test-tap:
	$(BATS) --formatter tap $(TEST_DIR)/

test-filter:
ifndef FILTER
	$(error FILTER is required — usage: make test-filter FILTER=resolve)
endif
	$(BATS) --filter "$(FILTER)" $(TEST_DIR)/
