use containerd_shim_protos as shim;

use protobuf::{
    well_known_types::{
        any::Any,
        struct_::{value::Kind, Struct, Value},
    },
    MessageField,
};
use shim::ttrpc::context::Context;
use shim::{api, api::ConnectResponse, Client, TaskClient};

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use liboci_cli::GlobalOpts;

use super::Backend;

const TTRPC_ADDRESS: &str = "TTPRC_ADDRESS";

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    shim: PathBuf,
    socket: PathBuf,
    events: PathBuf,
    bundle_dir: PathBuf,
    debug_shim: bool,
}

impl Config {
    pub fn instantiate(self, opts: GlobalOpts) -> Box<dyn Backend> {
        Box::new(ShimV2Backend::new(
            self.shim,
            self.socket,
            self.events,
            self.bundle_dir,
            self.debug_shim,
            opts,
        ))
    }
}

#[derive(Debug)]
struct ShimV2Backend {
    shim: PathBuf,
    socket: PathBuf,
    events: PathBuf,
    bundle_dir: PathBuf,
    debug_shim: bool,
    global_opts: GlobalOpts,
    container_id: String,
}

fn path_buf_to_str<'a>(kind: &str, path: &'a Path) -> Result<&'a str> {
    path.to_str().ok_or_else(|| {
        anyhow!(
            "ShimV2 {} path {} contains invalid characters",
            kind,
            path.display(),
        )
    })
}

fn path_buf_to_value(kind: &str, path: &Path) -> Result<Value> {
    let string = path_buf_to_str(kind, path)?;
    Ok(Value {
        kind: Some(Kind::StringValue(string.to_string())),
        ..Default::default()
    })
}

fn string_value(value: &str) -> Value {
    Value {
        kind: Some(Kind::StringValue(value.into())),
        ..Default::default()
    }
}

fn bool_value(value: bool) -> Value {
    Value {
        kind: Some(Kind::BoolValue(value)),
        ..Default::default()
    }
}

fn u32_value(value: u32) -> Value {
    Value {
        kind: Some(Kind::NumberValue(f64::from(value))),
        ..Default::default()
    }
}

fn add_option(opts: &mut Struct, kind: &str, value: Value) {
    opts.fields.insert(kind.to_string(), value);
}

fn add_u32_option(opts: &mut Struct, kind: &str, value: u32) {
    add_option(opts, kind, u32_value(value));
}

fn add_string_option(opts: &mut Struct, kind: &str, value: &str) {
    add_option(opts, kind, string_value(value));
}

fn add_path_option(opts: &mut Struct, kind: &str, path: &Path) -> Result<()> {
    let path = path_buf_to_value(kind, path)?;
    add_option(opts, kind, path);
    Ok(())
}

fn add_bool_option(opts: &mut Struct, kind: &str, value: bool) {
    add_option(opts, kind, bool_value(value));
}

impl ShimV2Backend {
    fn new(
        shim: PathBuf,
        socket: PathBuf,
        events: PathBuf,
        bundle_dir: PathBuf,
        debug_shim: bool,
        global_opts: GlobalOpts,
    ) -> Self {
        ShimV2Backend {
            shim,
            socket,
            events,
            bundle_dir,
            debug_shim,
            global_opts,
            container_id: "".to_string(),
        }
    }

    fn launch(&self, socket_path: &str, pid: &str) -> Result<Client> {
        // Need to switch to the correct directory for the shim
        let bundle_dir = self.bundle_dir.as_os_str();
        let bundle_str = bundle_dir.to_str().ok_or(anyhow!(
            "The bundle_dir option {:?} contains invalid characters",
            bundle_dir
        ))?;
        let offset = bundle_str
            .find("{container-id}")
            .ok_or(anyhow!("The bundle_dir option is missing container-id"))?;
        let bundle_dir = bundle_str.replace("{container-id}", pid);
        if self.debug_shim {
            println!("bundle dir after replacement is {:?}", bundle_dir);
        }
        env::set_current_dir(bundle_dir)?;

        // Need to create a `log` file to log output of target task
        let mut file = File::create("log")?;
        file.write_all(b"")?;

        let mut cmdargs = Vec::<OsString>::new();

        if self.debug_shim {
            cmdargs.push("-debug".into());
        }
        cmdargs.push("-namespace".into());
        cmdargs.push("default".into());
        cmdargs.push("-address".into());
        cmdargs.push(self.socket.clone().into());
        cmdargs.push("-id".into());
        cmdargs.push(pid.into());
        cmdargs.push("-publish-binary".into());
        cmdargs.push(self.events.clone().into());
        cmdargs.push("start".into());

        let status = Command::new(&self.shim).args(cmdargs).status()?;

        if status.success() {
            return shim::Client::connect(socket_path).map_err(anyhow::Error::from);
        }

        let path = &self.shim;
        Err(if let Some(sig) = status.signal() {
            anyhow!("ShimV2 backend {:?} terminated with signal {:?}", path, sig)
        } else if let Some(code) = status.code() {
            anyhow!("ShimV2 backend {:?} failed with status code {}", path, code)
        } else {
            anyhow!("Unidentified failure in ShimV2 backend")
        })
    }

    fn invoke(&self, pid: &str) -> Result<(TaskClient, Context, ConnectResponse)> {
        let socket_path = path_buf_to_str("socket", &self.socket)?;
        let client =
            shim::Client::connect(socket_path).or_else(|_| self.launch(socket_path, pid))?;
        let task_client = shim::TaskClient::new(client);
        let context = Context::default();
        let req = api::ConnectRequest {
            id: pid.to_string(),
            ..Default::default()
        };
        let resp = task_client.connect(context.clone(), &req)?;
        Ok((task_client, context, resp))
    }
}

impl Backend for ShimV2Backend {
    // Standard commands (from liboci_cli::StandardCmd)
    fn create(&self, args: liboci_cli::Create) -> Result<()> {
        if self.debug_shim {
            println!("Bundle argument is {:?}", args.bundle);
        }
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let bundle = path_buf_to_str("bundle", &args.bundle)?;

        if let Some(socket) = args.console_socket {
            println!(
                "Console socket {} option not implemented, ignored",
                socket.display()
            );
        }
        if let Some(pid_file) = args.pid_file {
            println!(
                "pid_file option {} not implemented, ignored",
                pid_file.display()
            );
        }
        if args.no_pivot {
            eprintln!("no-pivot option not implemented, ignored");
        }
        if args.no_new_keyring {
            eprintln!("no-new-keyring option not implemented, ignored");
        }
        if args.preserve_fds > 0 {
            eprintln!("preserve-fds option not implemented, ignored");
        }
        let req = api::CreateTaskRequest {
            id: args.container_id,
            bundle: bundle.to_owned(),
            ..Default::default()
        };
        let resp = task.create(context, &req)?;
        if self.global_opts.debug {
            println!("Create connect response {:?}", connect_response);
            println!("Create response {:?}", resp);
        }
        Ok(())
    }

    fn start(&self, args: liboci_cli::Start) -> Result<()> {
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let req = api::StartRequest {
            id: args.container_id,
            ..Default::default()
        };
        let resp = task.start(context, &req)?;
        if self.global_opts.debug {
            println!("Start connect response {:?}", connect_response);
            println!("Start response {:?}", resp);
        }

        Ok(())
    }

    fn kill(&self, args: liboci_cli::Kill) -> Result<()> {
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let signal = args.signal.parse::<u32>()?;
        let req = api::KillRequest {
            id: args.container_id,
            signal: signal,
            all: args.all,
            ..Default::default()
        };
        let resp = task.kill(context, &req)?;
        if self.global_opts.debug {
            println!(
                "Kill connect response {:?} task response {:?}",
                connect_response, resp
            );
        }
        Ok(())
    }

    fn delete(&self, args: liboci_cli::Delete) -> Result<()> {
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let req = api::DeleteRequest {
            id: args.container_id,
            ..Default::default()
        };
        let resp = task.delete(context, &req)?;
        if self.global_opts.debug {
            println!("Delete connect response {:?}", connect_response);
            println!("Delete response {:?}", resp);
        }

        Ok(())
    }

    fn state(&self, args: liboci_cli::State) -> Result<()> {
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let req = api::StateRequest {
            id: args.container_id,
            ..Default::default()
        };
        let resp = task.state(context, &req)?;
        if self.global_opts.debug {
            println!("State connect response {:?}", connect_response);
            println!("State response {:?}", resp);
        }

        Ok(())
    }

    // Common non-standard commands (from liboci_cli::CommonCmd)
    fn checkpoint(&self, args: liboci_cli::Checkpoint) -> Result<()> {
        let (task, context, connect_response) = self.invoke(&args.container_id)?;
        let image_path = path_buf_to_str("image_path", &args.image_path)?;
        let mut opts = Struct::new();

        if let Some(work_path) = args.work_path {
            add_path_option(&mut opts, "work-path", &work_path)?;
        }
        if let Some(parent_path) = args.parent_path {
            add_path_option(&mut opts, "parent-path", &parent_path)?;
        }
        if args.leave_running {
            add_bool_option(&mut opts, "leave-running", args.leave_running);
        }
        if args.tcp_established {
            add_bool_option(&mut opts, "tcp-established", args.tcp_established);
        }
        if args.ext_unix_sk {
            add_bool_option(&mut opts, "ext-unix-sk", args.ext_unix_sk);
        }
        if args.shell_job {
            add_bool_option(&mut opts, "shell-job", args.shell_job);
        }
        if args.lazy_pages {
            add_bool_option(&mut opts, "lazy-pages", args.lazy_pages);
        }
        if let Some(status_fd) = args.status_fd {
            add_u32_option(&mut opts, "status-fd", status_fd);
        }
        if let Some(page_server) = args.page_server {
            add_string_option(&mut opts, "page-server", &page_server);
        }
        if args.file_locks {
            add_bool_option(&mut opts, "file-locks", args.file_locks);
        }
        if args.pre_dump {
            add_bool_option(&mut opts, "pre-dump", args.pre_dump);
        }
        if let Some(manage_cgroups_mode) = args.manage_cgroups_mode {
            add_string_option(&mut opts, "manage-cgroups-mode", &manage_cgroups_mode);
        }
        if args.empty_ns {
            add_bool_option(&mut opts, "empty-ns", args.empty_ns);
        }
        if args.auto_dedup {
            add_bool_option(&mut opts, "auto-dedup", args.auto_dedup);
        }

        let options = Any::pack(&opts)?;
        let req = api::CheckpointTaskRequest {
            id: args.container_id,
            path: image_path.to_owned(),
            options: MessageField::some(options),
            ..Default::default()
        };

        let resp = task.checkpoint(context, &req)?;
        if self.global_opts.debug {
            println!("Checkpoint connect response {:?}", connect_response);
            println!("Checkpoint response {:?}", resp);
        }

        Ok(())
    }

    fn events(&self, args: liboci_cli::Events) -> Result<()> {
        Ok(())
    }

    fn exec(&self, args: liboci_cli::Exec) -> Result<()> {
        Ok(())
    }

    fn features(&self, _args: liboci_cli::Features) -> Result<()> {
        Ok(())
    }

    fn list(&self, args: liboci_cli::List) -> Result<()> {
        Ok(())
    }

    fn pause(&self, args: liboci_cli::Pause) -> Result<()> {
        Ok(())
    }

    fn ps(&self, args: liboci_cli::Ps) -> Result<()> {
        Ok(())
    }

    fn resume(&self, args: liboci_cli::Resume) -> Result<()> {
        Ok(())
    }

    fn run(&self, args: liboci_cli::Run) -> Result<()> {
        Ok(())
    }

    fn update(&self, args: liboci_cli::Update) -> Result<()> {
        Ok(())
    }

    fn spec(&self, args: liboci_cli::Spec) -> Result<()> {
        Ok(())
    }
}
