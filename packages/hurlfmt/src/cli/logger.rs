/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2024 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use std::path::PathBuf;

use crate::linter;
use colored::*;
use hurl_core::error::DisplaySourceError;
use hurl_core::parser;
use hurl_core::text::Format;

pub fn make_logger_verbose(verbose: bool) -> impl Fn(&str) {
    move |message| log_verbose(verbose, message)
}

pub fn make_logger_error_message(color: bool) -> impl Fn(bool, &str) {
    move |warning, message| log_error_message(color, warning, message)
}

pub fn make_logger_parser_error(
    lines: Vec<String>,
    color: bool,
    filename: Option<PathBuf>,
) -> impl Fn(&parser::ParseError, bool) {
    move |error: &parser::ParseError, warning: bool| {
        log_error(lines.clone(), color, filename.clone(), error, warning);
    }
}

pub fn make_logger_linter_error(
    lines: Vec<String>,
    color: bool,
    filename: Option<PathBuf>,
) -> impl Fn(&linter::Error, bool) {
    move |error: &linter::Error, warning: bool| {
        log_error(lines.clone(), color, filename.clone(), error, warning);
    }
}

pub fn log_info(message: &str) {
    eprintln!("{message}");
}

fn log_error_message(color: bool, warning: bool, message: &str) {
    let log_type = match (color, warning) {
        (false, true) => "warning".to_string(),
        (false, false) => "error".to_string(),
        (true, true) => "warning".yellow().bold().to_string(),
        (true, false) => "error".red().bold().to_string(),
    };
    eprintln!("{log_type}: {message}");
}

fn log_verbose(verbose: bool, message: &str) {
    if verbose {
        if message.is_empty() {
            eprintln!("*");
        } else {
            eprintln!("* {message}");
        }
    }
}

fn log_error(
    lines: Vec<String>,
    color: bool,
    filename: Option<PathBuf>,
    error: &dyn DisplaySourceError,
    warning: bool,
) {
    let line_number_size = if lines.len() < 100 {
        2
    } else if lines.len() < 1000 {
        3
    } else {
        4
    };

    let error_type = if warning {
        String::from("warning")
    } else {
        String::from("error")
    };
    let error_type = if !color {
        error_type
    } else if warning {
        error_type.yellow().bold().to_string()
    } else {
        error_type.red().bold().to_string()
    };
    let format = if color { Format::Ansi } else { Format::Plain };
    eprintln!("{}: {}", error_type, error.description());

    if let Some(filename) = filename {
        eprintln!(
            "{}--> {}:{}:{}",
            " ".repeat(line_number_size).as_str(),
            filename.display(),
            error.source_info().start.line,
            error.source_info().start.column,
        );
    }
    eprintln!("{} |", " ".repeat(line_number_size));

    let line = lines.get(error.source_info().start.line - 1).unwrap();
    let line = str::replace(line, "\t", "    "); // replace all your tabs with 4 characters
    eprintln!(
        "{line_number:>width$} |{line}",
        line_number = error.source_info().start.line,
        width = line_number_size,
        line = if line.is_empty() {
            line
        } else {
            format!(" {line}")
        }
    );

    // TODO: to clean/Refacto
    // specific case for assert errors
    let lines = lines.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    if error.source_info().start.column == 0 {
        let fix_me = &error.fixme(&lines);
        let fixme_lines = fix_me.split('\n');

        // edd an empty line at the end?
        for line in fixme_lines {
            eprintln!(
                "{} |   {fixme}",
                " ".repeat(line_number_size).as_str(),
                fixme = line.to_string(format),
            );
        }
    } else {
        let line = lines.get(error.source_info().start.line - 1).unwrap();
        let width = error.source_info().end.column - error.source_info().start.column;

        let mut tab_shift = 0;
        for (i, c) in line.chars().enumerate() {
            if i >= error.source_info().start.column - 1 {
                break;
            };
            if c == '\t' {
                tab_shift += 1;
            }
        }
        eprintln!(
            "{} | {}{} {fixme}",
            " ".repeat(line_number_size).as_str(),
            " ".repeat(error.source_info().start.column - 1 + tab_shift * 3),
            "^".repeat(if width > 1 { width } else { 1 }),
            fixme = error.fixme(&lines).to_string(format),
        );
    }

    eprintln!("{} |\n", " ".repeat(line_number_size));
}
