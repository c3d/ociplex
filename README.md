# ociplex - An OCI multiplexer

The `ociplex` tool is an OCI runtime multiplexer.

The primary usage for `ociplex` is to act as a gateway between OCI-only runtime
engines like `podman` and semi-compliant runtimes like Kata Containers (which,
after version 2.0, only supports the `shimv2` interface).

However, `ociplex` features several back-ends, which makes it possible to target
another OCI runtime, for example to add additional logging or debugging
capability.

## Building

At the moment, this requires a custom version of the [`liboci-cli` crate][oci]

[oci]: https://github.com/c3d/youki/tree/liboci-ociplex


## Testing

Testing is done through the standard OCI runtime test suite:

```
$ git clone git@github.com:opencontainers/runtime-tools.git
$ cd runtime-tools
$ make runtimetest validation-executables
```

You can check that the test suite works for you as follows:

```
$ sudo -E make RUNTIME=/usr/local/bin/runc localvalidation
```

Then you can use it to run your tests with `ociplex` as the runtime.
