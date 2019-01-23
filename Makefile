LIB = core http-client
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
	cd components/$1 && cargo test && cargo test --release
.PHONY: unit-$1
endef
$(foreach component,$(ALL),$(eval $(call UNIT,$(component))))

# Lints we need to work through and decide as a team whether to allow or fix
UNEXAMINED_LINTS = clippy::const_static_lifetime \
				   clippy::cyclomatic_complexity \
				   clippy::deref_addrof \
				   clippy::expect_fun_call \
				   clippy::for_kv_map \
				   clippy::get_unwrap \
				   clippy::identity_conversion \
				   clippy::if_let_some_result \
				   clippy::large_enum_variant \
				   clippy::len_without_is_empty \
				   clippy::len_zero \
				   clippy::let_and_return \
				   clippy::let_unit_value \
				   clippy::map_clone \
				   clippy::match_bool \
				   clippy::match_ref_pats \
				   clippy::module_inception \
				   clippy::needless_bool \
				   clippy::needless_collect \
				   clippy::needless_pass_by_value \
				   clippy::needless_range_loop \
				   clippy::needless_return \
				   clippy::new_ret_no_self \
				   clippy::new_without_default \
				   clippy::new_without_default_derive \
				   clippy::ok_expect \
				   clippy::op_ref \
				   clippy::option_map_unit_fn \
				   clippy::or_fun_call \
				   clippy::println_empty_string \
				   clippy::ptr_arg \
				   clippy::question_mark \
				   clippy::redundant_closure \
				   clippy::redundant_field_names \
				   clippy::redundant_pattern_matching \
				   clippy::single_char_pattern \
				   clippy::single_match \
				   clippy::string_lit_as_bytes \
				   clippy::too_many_arguments \
				   clippy::toplevel_ref_arg \
				   clippy::trivially_copy_pass_by_ref \
				   clippy::unit_arg \
				   clippy::unnecessary_operation \
				   clippy::unreadable_literal \
				   clippy::unused_label \
				   clippy::unused_unit \
				   clippy::useless_asref \
				   clippy::useless_format \
				   clippy::useless_let_if_seq \
				   clippy::useless_vec \
				   clippy::write_with_newline \
				   clippy::wrong_self_convention \
				   renamed_and_removed_lints

# Lints we disagree with and choose to keep in our code with no warning
ALLOWED_LINTS =

# Known failing lints we want to receive warnings for, but not fail the build
LINTS_TO_FIX =

# Lints we don't expect to have in our code at all and want to avoid adding
# even at the cost of failing the build
DENIED_LINTS = clippy::assign_op_pattern \
			   clippy::blacklisted_name \
			   clippy::block_in_if_condition_stmt \
			   clippy::bool_comparison \
			   clippy::cast_lossless \
			   clippy::clone_on_copy \
			   clippy::cmp_owned \
			   clippy::collapsible_if \
			   clippy::correctness \

define LINT
lint-$1: ## executes the $1 component's linter checks
	$(run) sh -c 'cd components/$1 && cargo clippy --all-targets --tests $(CARGO_FLAGS) -- \
												   $(addprefix -A ,$(UNEXAMINED_LINTS)) \
												   $(addprefix -A ,$(ALLOWED_LINTS)) \
												   $(addprefix -W ,$(LINTS_TO_FIX)) \
												   $(addprefix -D ,$(DENIED_LINTS))'
.PHONY: lint-$1
endef
$(foreach component,$(ALL),$(eval $(call LINT,$(component))))

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
