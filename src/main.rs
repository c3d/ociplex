// ****************************************************************************
//  main.rs                                                     ociplex project
// ****************************************************************************
//
//   File Description:
//
//     Main entry point for the ociplex utility
//
//     `ociplex` is an OCI runtime adapter, which can convert from the legacy
//     command-line interface, as implemented by `runc`, `crun` and similar,
//     and a more modern "shim-v2" RPC-based implementation, as exposed for
//     example by `kata`.
//
//
//
// ****************************************************************************
//   (C) 2023 Christophe de Dinechin <dinechin@redhat.com>
//   This software is licensed under the terms outlined in LICENSE.txt
// ****************************************************************************
//   This file is part of ociplex.
//
//   ociplex is free software: you can redistribute it and/or modify
//   it under the terms outlined in the LICENSE.txt file
//
//   ociplex is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// ****************************************************************************

use std::path::PathBuf;
use std::{fs, str::FromStr};

use anyhow::{Context, Result};
use clap::{crate_version, Parser};
use liboci_cli::{CommonCmd, GlobalOpts, StandardCmd};
use tracing::{debug, error, info, instrument, Level};
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};

mod backend;

#[derive(Parser, Debug)]
enum Subcommand
// ----------------------------------------------------------------------------
//   Subcommands for the runtime
// ----------------------------------------------------------------------------
{
    // Standard commands (create, start, state, kill, delete) are defined
    // directly by the OCI specification
    #[clap(flatten)]
    Standard(StandardCmd),

    // Common commands (e.g. checkpoint or ps) are implemented by many runtimes,
    // but not considered mandatory to be OCI-compliant.
    #[clap(flatten)]
    CommonCmd(CommonCmd),
}

#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Opts
// ----------------------------------------------------------------------------
//  Options for ociplex are provided by the configuration file
// ----------------------------------------------------------------------------
//  Since the command-line options are actually used for OCI command-line
//  arguments, additional configuration of ociplex has to come from some other
//  place. We use a toml configuration file, which can be overriden with the
//  -backend uption.
//  See examples/*.toml for examples of configuration files.
{
    // Configuration file
    #[clap(long, default_value = "/etc/ociplex.toml")]
    backend: PathBuf,

    // Shared (global) options, from the OCI specification for runtimes
    #[clap(flatten)]
    global: GlobalOpts,

    /// Set the log level (info, debug, trace, error or none)
    #[clap(long)]
    log_level: Option<Level>,

    /// Log to the system logger (syslog)
    #[clap(long)]
    syslog: bool,

    // Subcommand and its associated options if any
    #[clap(subcommand)]
    subcmd: Subcommand,
}

// Default tracing level depends on configuration (debug or not)
#[cfg(debug_assertions)]
const DEFAULT_TRACING_LEVEL: Level = Level::DEBUG;

#[cfg(not(debug_assertions))]
const DEFAULT_TRACING_LEVEL: Level = Level::ERROR;

fn set_tracing_level() -> Result<()>
// ----------------------------------------------------------------------------
//  Select tracing options and install a global tracing collector
// ----------------------------------------------------------------------------
//  Tracing options are set by:
//  1. The OCIPLEX_LOG environment variable
//  2. The --debug option
//  3. The --log-level option, which overrides --debug
//  4. The --log, --log-format and --log-to-syslog
{
    // We need options to define where the tracing goes
    let opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(e) => e.exit(),
    };

    // Select actual tracing level
    let env_filter = EnvFilter::try_from_env("OCIPLEX_LOG");
    let tracing_level = if let Some(level) = opts.log_level {
        Some(level)
    } else if opts.global.debug {
        Some(Level::DEBUG)
    } else if env_filter.is_ok() {
        None
    } else {
        Some(DEFAULT_TRACING_LEVEL)
    };

    let tracing_filter = if let Some(level) = tracing_level {
        EnvFilter::from(level.as_str())
    } else {
        env_filter.unwrap()
    };

    tracing_subscriber::fmt()
        .with_env_filter(tracing_filter)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .init();

    Ok(())
}

#[instrument(level = "debug")]
fn instrumented(x: i32)
// ----------------------------------------------------------------------------
//   This is just for testing
// ----------------------------------------------------------------------------
{
    info!("Inside instrumented x={}", x);
    debug!("Inside instrumented x={}", x)
}

fn main() -> Result<()>
// ----------------------------------------------------------------------------
//  Main entry point for the tool
// ----------------------------------------------------------------------------
{
    // Setup global tracing
    set_tracing_level()?;

    // Parse options with clap
    info!("Parsing options");
    let opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(e) => e.exit(),
    };

    instrumented(42);

    // Read backend configuration from file specified with --backend option
    let config = fs::read_to_string(&opts.backend).context("Reading backend config")?;
    let config: backend::Config = toml::from_str(&config).context("Parsing backend config")?;

    // Instantiate the backend and delegate the rest of the work to it
    let backend = config.instantiate(opts.global);
    match opts.subcmd {
        Subcommand::Standard(std) => backend.standard_command(std)?,
        Subcommand::CommonCmd(common) => backend.common_command(common)?,
    }

    Ok(())
}
