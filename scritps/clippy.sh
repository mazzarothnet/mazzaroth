#!/usr/bin/env bash
cargo clippy --all-targets --all-features "$@" -- \
        -D warnings \
        -D clippy::panic \
        -D clippy::expect_used \
        -D clippy::unwrap_used \
        -D clippy::branches_sharing_code \
        -D clippy::cast_lossless \
        -D clippy::exit \
        -D clippy::implicit_clone \
        -D clippy::index_refutable_slice \
        -D clippy::map_err_ignore \
        -D clippy::maybe_infinite_iter \
        -D clippy::mem_forget \
        -D clippy::mismatching_type_param_order \
        -D clippy::mutex_integer \
        -D clippy::needless_pass_by_value \
        -D clippy::option_option \
        -D clippy::wildcard_imports