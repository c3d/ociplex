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

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{crate_version, Parser};

use liboci_cli::{CommonCmd, GlobalOpts, StandardCmd};

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

    // Subcommand and its associated options if any
    #[clap(subcommand)]
    subcmd: Subcommand,
}

fn main() -> Result<()>
// ----------------------------------------------------------------------------
//  Main entry point for the tool
// ----------------------------------------------------------------------------
{
    // Pars options with clap
    let opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(e) => e.exit(),
    };

    // Read backend configuration from file specified with -backend option
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
