.PHONY: test test-verbose test-filter test-tap coverage

BATS := $(if $(shell command -v bats 2>/dev/null),bats,./test/bats/bin/bats)
KCOV := $(shell command -v kcov 2>/dev/null)
TEST_DIR := test
KCOV_DIR := /tmp/kcov-output
FILTER ?=

test:
ifdef KCOV
	rm -rf $(KCOV_DIR) $(KCOV_DIR)-direct
	kcov ... $(BATS) --formatter tap $(TEST_DIR)/ 2>&1 | tee /tmp/bats-out.txt; EXIT_BATS=$${PIPESTATUS[0]}; \
	kcov ... $(TEST_DIR)/direct-coverage.sh 2>/dev/null; EXIT_DIRECT=$$?; \
	python3 test/coverage-report.py $(KCOV_DIR) $(KCOV_DIR)-direct; \
	grep '^not ok' /tmp/bats-out.txt | sed 's/^not ok /  FAIL: /' ; \
	exit $$(( EXIT_BATS || EXIT_DIRECT ))
endif

coverage:
ifdef KCOV
	rm -rf $(KCOV_DIR)
	kcov --include-path="$(CURDIR)/scripts" --bash-parse-files-in-dir="$(CURDIR)/scripts" --clean $(KCOV_DIR) $(BATS) $(TEST_DIR)/; \
	EXIT=$$?; \
	python3 test/coverage-report.py $(KCOV_DIR); \
	exit $$EXIT
else
	@echo "  kcov not found — install kcov for coverage metrics (https://github.com/SimonKagstrom/kcov)"
	@exit 1
endif

test-verbose:
	$(BATS) --verbose-run $(TEST_DIR)/

test-tap:
	$(BATS) --formatter tap $(TEST_DIR)/

test-filter:
ifndef FILTER
	$(error FILTER is required — usage: make test-filter FILTER=resolve)
endif
	$(BATS) --filter "$(FILTER)" $(TEST_DIR)/
