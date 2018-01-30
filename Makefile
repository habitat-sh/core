LIB = core
ALL = $(LIB)

.DEFAULT_GOAL := build-lib

build: build-lib
build-all: build
.PHONY: build build-all

build-lib: $(addprefix build-,$(LIB)) ## builds the library components
.PHONY: build-lib

unit: unit-lib
unit-all: unit
.PHONY: unit unit-all

unit-lib: $(addprefix unit-,$(LIB)) ## executes the library components' unit test suites
.PHONY: unit-lib

lint: lint-lib
lint-all: lint
.PHONY: lint lint-all

lint-lib: $(addprefix lint-,$(LIB))
.PHONY: lint-lib

clean: clean-lib
clean-all: clean
.PHONY: clean clean-all

clean-lib: $(addprefix clean-,$(LIB)) ## cleans the library components' project trees
.PHONY: clean-lib

fmt: fmt-lib
fmt-all: fmt
.PHONY: fmt fmt-all

fmt-lib: $(addprefix fmt-,$(LIB)) ## formats the library components' codebases
.PHONY: clean-lib

help:
	@perl -nle'print $& if m{^[a-zA-Z_-]+:.*?## .*$$}' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
.PHONY: help

define BUILD
build-$1: ## builds the $1 component
	cd components/$1 && cargo build
.PHONY: build-$1

endef
$(foreach component,$(ALL),$(eval $(call BUILD,$(component))))

define UNIT
unit-$1: ## executes the $1 component's unit test suite
	cd components/$1 && cargo test
.PHONY: unit-$1
endef
$(foreach component,$(ALL),$(eval $(call UNIT,$(component))))

define CLEAN
clean-$1: ## cleans the $1 component's project tree
	cd components/$1 && cargo clean
.PHONY: clean-$1

endef
$(foreach component,$(ALL),$(eval $(call CLEAN,$(component))))

define FMT
fmt-$1: ## formats the $1 component
	cd components/$1 && cargo fmt
.PHONY: fmt-$1

endef
$(foreach component,$(ALL),$(eval $(call FMT,$(component))))
