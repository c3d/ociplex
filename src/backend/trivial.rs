// ****************************************************************************
//  trivial.rs                                                  ociplex project
// ****************************************************************************
//
//   File Description:
//
//    The trivial backedn is only recording what is happening
//    This is mostly for debugging purpose
//
//
//
//
//
//
//
// ****************************************************************************
//   (C) 2023 Christophe de Dinechin <christophe@dinechin.org>
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

use anyhow::{anyhow, Result};

use liboci_cli::GlobalOpts;

use super::Backend;

#[derive(Debug, serde::Deserialize)]
pub struct Config {}

impl Config
// ----------------------------------------------------------------------------
//    Implementation of the ociplex configuration interface
// ----------------------------------------------------------------------------
{
    pub fn instantiate(self, global: GlobalOpts) -> Box<dyn Backend>
    // ------------------------------------------------------------------------
    //   Instantiate a trivial backend from given OCI command-line options
    // ------------------------------------------------------------------------
    {
        dbg!(global);
        Box::new(TrivialBackend {})
    }
}

#[derive(Debug)]
struct TrivialBackend
// ----------------------------------------------------------------------------
//   Data specific to the trivial backend
// ----------------------------------------------------------------------------
//   Since the trivial backend does so little, there is not associated data
{}

impl Backend for TrivialBackend
// ----------------------------------------------------------------------------
//   Implement the backend interface (i.e. OCI runtime commands)
// ----------------------------------------------------------------------------
//   All commands simply return an error documenting the argument they received
{
    // ========================================================================
    //
    //   All standard commands (liboci_cli::StandardCmd)
    //
    // ========================================================================

    fn create(&self, args: liboci_cli::Create) -> Result<()>
    // ------------------------------------------------------------------------
    //    'create' subcommand: create a container
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial {:?}", args))
    }

    fn start(&self, args: liboci_cli::Start) -> Result<()>
    // ------------------------------------------------------------------------
    //    'start' subcommand: start a container
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial {:?}", args))
    }

    fn kill(&self, args: liboci_cli::Kill) -> Result<()>
    // ------------------------------------------------------------------------
    //   'kill' subcommand kills the container (i.e. the associated process)
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial {:?}", args))
    }

    fn delete(&self, args: liboci_cli::Delete) -> Result<()>
    // ------------------------------------------------------------------------
    //  'delete' subcommand removes the data associated with the container.
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial {:?}", args))
    }

    fn state(&self, args: liboci_cli::State) -> Result<()>
    // ------------------------------------------------------------------------
    //   'state' subcommand returns the state for a container
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial {:?}", args))
    }

    // ========================================================================
    //
    //   All common but non-standard commands (liboci_cli::CommonCmd)
    //
    // ========================================================================

    fn checkpoint(&self, args: liboci_cli::Checkpoint) -> Result<()>
    // ------------------------------------------------------------------------
    //   'checkpoint' subcommand saves the state of a container
    // ------------------------------------------------------------------------
    {
        Err(anyhow!("trivial: {:?}", args))
    }

    fn events(&self, args: liboci_cli::Events) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'events' subcommand gathers event list
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn exec(&self, args: liboci_cli::Exec) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'exec' subcommand executes a process inside a container
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn features(&self, args: liboci_cli::Features) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'features' subcommand lists supported features
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn list(&self, args: liboci_cli::List) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'list' subcommand lists containers
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn pause(&self, args: liboci_cli::Pause) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'pause' subcommand suspends execution of all processes in a container
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn ps(&self, args: liboci_cli::Ps) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'ps' subcommand lists processes belonging to a given container
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn resume(&self, args: liboci_cli::Resume) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'resume' subcommand resumes processes (see pause)
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn run(&self, args: liboci_cli::Run) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'run' subcommand creates an instance of a container and starts it
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn update(&self, args: liboci_cli::Update) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'update' subcommand changes resource constraints for a container
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
    fn spec(&self, args: liboci_cli::Spec) -> Result<()> {
    // ------------------------------------------------------------------------
    //   'spec' subcommand creates a new config.json file for the bundle
    // ------------------------------------------------------------------------

        Err(anyhow!("trivial: {:?}", args))
    }
}
