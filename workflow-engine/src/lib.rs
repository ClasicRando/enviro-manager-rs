#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![warn(clippy::cloned_instead_of_copied)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::create_dir)]
#![warn(clippy::empty_structs_with_brackets)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::expect_used)]
#![warn(clippy::fn_params_excessive_bools)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_string_new)]
#![warn(clippy::match_on_vec_items)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::missing_assert_message)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_continue)]
#![warn(clippy::needless_for_each)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::panic)]
#![warn(clippy::partial_pub_fields)]
#![warn(clippy::print_stdout)]
#![warn(clippy::pub_use)]
#![warn(clippy::string_to_string)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_slice)]
#![warn(clippy::too_many_arguments)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::unnecessary_box_returns)]
#![warn(clippy::unused_async)]
#![warn(clippy::unused_self)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::use_self)]
#![warn(clippy::wildcard_imports)]

//! Workflow Engine component of the EnivroManager application suite

pub mod api;
pub mod database;
pub mod executor;
pub mod job;
pub mod job_worker;
pub mod services;
pub mod workflow_run;
