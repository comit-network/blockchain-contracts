RUSTUP = rustup

# The CI should pass a DEFAULT_TOOLCHAIN env var, if not we default to stable
DEFAULT_TOOLCHAIN ?= stable
TOOLCHAIN = $(DEFAULT_TOOLCHAIN)
CARGO = $(RUSTUP) run --install $(TOOLCHAIN) cargo --color always

NIGHTLY_TOOLCHAIN = nightly-2019-12-30
CARGO_NIGHTLY = $(RUSTUP) run --install $(NIGHTLY_TOOLCHAIN) cargo --color always

GIT_HOOKS_PATH = ".githooks"
GIT_HOOKS = $(wildcard $(GIT_HOOKS_PATH)/*)

INSTALLED_TOOLCHAINS = $(shell $(RUSTUP) toolchain list)
INSTALLED_COMPONENTS = $(shell $(RUSTUP) component list --installed --toolchain $(TOOLCHAIN))
INSTALLED_NIGHTLY_COMPONENTS = $(shell $(RUSTUP) component list --installed --toolchain $(NIGHTLY_TOOLCHAIN))
AVAILABLE_CARGO_COMMANDS = $(shell $(CARGO) --list)

# All our targets go into .PHONY because none of them actually create files
.PHONY: init_git_hooks default install_rust install_rust_nightly install_clippy install_rustfmt install_tomlfmt install clean all ci build clippy test doc check_format format check_rust_format check_toml_format check_ts_format

default: init_git_hooks format build clippy

init_git_hooks:
	git config core.hooksPath $(GIT_HOOKS_PATH)

## Dev environment

install_rust:
ifeq (,$(findstring $(TOOLCHAIN),$(INSTALLED_TOOLCHAINS)))
	$(RUSTUP) install $(TOOLCHAIN)
endif

install_rust_nightly:
ifeq (,$(findstring $(NIGHTLY_TOOLCHAIN),$(INSTALLED_TOOLCHAINS)))
	$(RUSTUP) install $(NIGHTLY_TOOLCHAIN)
endif

install_clippy: install_rust
ifeq (,$(findstring clippy,$(INSTALLED_COMPONENTS)))
	$(RUSTUP) component add clippy --toolchain $(TOOLCHAIN)
endif

install_rustfmt: install_rust_nightly
ifeq (,$(findstring rustfmt,$(INSTALLED_NIGHTLY_COMPONENTS)))
	$(RUSTUP) component add rustfmt --toolchain $(NIGHTLY_TOOLCHAIN)
endif

install_tomlfmt: install_rust
ifeq (,$(findstring tomlfmt,$(AVAILABLE_CARGO_COMMANDS)))
	$(CARGO) install cargo-tomlfmt
endif

## Development tasks

clean:
	$(CARGO) clean

all: init_git_hooks format clippy test build doc

ci: check_format clippy test build doc

build:
	$(CARGO) build --all --all-targets $(BUILD_ARGS)

clippy: install_clippy
	$(CARGO) clippy \
	    --all-targets \
	    -- \
	    -W clippy::cast_possible_truncation \
	    -W clippy::cast_sign_loss \
	    -W clippy::fallible_impl_from \
	    -W clippy::cast_precision_loss \
	    -W clippy::cast_possible_wrap \
	    -W clippy::print_stdout \
	    -W clippy::dbg_macro \
	    -D warnings

test:
	$(CARGO) test --all

doc:
	$(CARGO) doc

check_format: check_rust_format check_toml_format

STAGED_FILES = $(shell git diff --staged --name-only)
STAGED_RUST_FILES = $(filter %.rs,$(STAGED_FILES))
STAGED_TOML_FILES = $(filter %Cargo.toml,$(STAGED_FILES))

format: install_rustfmt install_tomlfmt
ifneq (,$(STAGED_RUST_FILES))
	$(CARGO_NIGHTLY) fmt
endif
ifneq (,$(STAGED_TOML_FILES))
	$(CARGO) tomlfmt -p Cargo.toml
endif

check_rust_format: install_rustfmt
ifneq (,$(STAGED_RUST_FILES))
	$(CARGO_NIGHTLY) fmt -- --check
endif

check_toml_format: install_tomlfmt
ifneq (,$(STAGED_TOML_FILES))
	$(CARGO) tomlfmt -d -p Cargo.toml
endif
