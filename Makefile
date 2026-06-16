.PHONY: test test-verbose test-filter test-tap coverage

SHELL := /bin/bash

BATS := $(if $(shell command -v bats 2>/dev/null),bats,./test/bats/bin/bats)
KCOV := $(shell command -v kcov 2>/dev/null)
TEST_DIR := test
KCOV_DIR := /tmp/kcov-output
FILTER ?=

test:
	@if command -v kcov >/dev/null 2>&1; then \
		rm -rf $(KCOV_DIR) $(KCOV_DIR)-direct; \
		kcov $(KCOV_DIR) $(BATS) --formatter tap $(TEST_DIR)/ 2>&1 | tee /tmp/bats-out.txt; EXIT_BATS=$${PIPESTATUS[0]}; \
		kcov $(KCOV_DIR)-direct $(TEST_DIR)/direct-coverage.sh 2>/dev/null; EXIT_DIRECT=$$?; \
		python3 test/coverage-report.py $(KCOV_DIR) $(KCOV_DIR)-direct; \
		grep '^not ok' /tmp/bats-out.txt | sed 's/^not ok /  FAIL: /' ; \
		exit $$(( EXIT_BATS || EXIT_DIRECT )); \
	else \
		$(BATS) --formatter tap $(TEST_DIR)/; \
	fi

coverage:
	@if command -v kcov >/dev/null 2>&1; then \
		rm -rf $(KCOV_DIR); \
		kcov --include-path="$(CURDIR)/scripts" --bash-parse-files-in-dir="$(CURDIR)/scripts" --clean $(KCOV_DIR) $(BATS) $(TEST_DIR)/; \
		EXIT=$$?; \
		python3 test/coverage-report.py $(KCOV_DIR); \
		exit $$EXIT; \
	else \
		echo "  kcov not found — install kcov for coverage metrics (https://github.com/SimonKagstrom/kcov)"; \
		exit 1; \
	fi

test-verbose:
	$(BATS) --verbose-run $(TEST_DIR)/

test-tap:
	$(BATS) --formatter tap $(TEST_DIR)/

test-filter:
	@if [ -z "$(FILTER)" ]; then echo "FILTER is required — usage: make test-filter FILTER=resolve" >&2; exit 1; fi
	$(BATS) --filter "$(FILTER)" $(TEST_DIR)/
