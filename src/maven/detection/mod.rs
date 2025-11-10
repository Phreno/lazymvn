//! Spring Boot detection and launch strategy

mod command_builder;
mod spring_boot;
#[cfg(test)]
mod spring_boot_tests;
mod strategy;
mod xml_parser;

pub use command_builder::build_launch_command;
pub use spring_boot::{detect_spring_boot_capabilities, SpringBootDetection};
pub use strategy::{decide_launch_strategy, LaunchStrategy};
