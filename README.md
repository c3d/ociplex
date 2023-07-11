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

The OCI runtimes must obey [the OCI specification][ocispec].

[ocispec]: https://github.com/opencontainers/runtime-tools/blob/master/docs/command-line-interface.md

Testing is done through the standard OCI runtime test suite:

```
$ git clone git@github.com:opencontainers/runtime-tools.git
$ cd runtime-tools
$ make runtimetest validation-executables
```

Some dependencies are necessary for the test suite to work well:

```
$ npm install tap
```

You can check that the test suite works for you as follows:

```
$ sudo -E make RUNTIME=/usr/local/bin/runc localvalidation
```

Then you can use it to run your tests with `ociplex` as the runtime.

Note that the test suite is sensitive to the environment, and getting it to
pass fully is difficult. Current results of this test suite on my machine:

* `runc`:
  ```
  Suites:   ​25 failed​, ​25 passed​, ​8 skip​, ​58 of 58 completed
  Asserts:   ​ ​​​80 failed​, ​3357 passed​, ​466 skip​, ​of 3903
  ```

* `crun`:
  ```
  Suites:   ​23 failed​, ​27 passed​, ​8 skip​, ​58 of 58 completed
  Asserts:   ​ ​​​45 failed​, ​3642 passed​, ​505 skip​, ​of 4192
  ```

* `youki`:
  ```
  Suites:   ​48 failed​, ​1 passed​, ​9 skip​, ​58 of 58 completed
  Asserts:   ​ ​​​73 failed​, ​23 passed​, ​9 skip​, ​of 105
  ```

### Running individual tests

You can run individual tests in `runtime-tools` as follows:

```
$ RUNTIME=/usr/bin/runc ./validation/start/start.t
```

### OCI bundle creation

You can use the tools to easily create an OCI bundle:

```
$ mkdir my-test
$ cd my-test
$ mkdir rootfs
$ ../oci-runtime-tool generate --output config.json
$ ../oci-runtime-tool validate
Bundle validation succeeded.
$ cd rootfs
$ tar xvfz ../../../../rootfs-amd64.tar.gz
```


### Example of high-level testing with `podman`

There are several examples of back-ends in the [examples/](examples) directory.
We will use the [configuration example][crun] to run the `cli` backend with
`/bin/crun` as the next OCI runtime.

[crun]: examples/cli-runc.toml

Since `'podman` expects a single argument for its `--runtime` option, which is a
path ot a runtime executable, we will need to create a small wrapper to act as
this binary, that runs the `debug` build of `ociplex` in the same directory:

```
#!/bin/bash
DIR=$(dirname $0)
BIN=$DIR/target/debug/ociplex
$BIN --backend $DIR/examples/cli-crun.toml --debug "$@"
```

This uses the `cli-crun.toml` example file as the backend, and passes the global
`--debug` option to `ociplex`. You can now run `podman` with it, knowing that
`podman` will expect a full path to your "runtime":

```
 podman --runtime $PWD/run-ociplex run -it fedora bash
[root@039750cf2130 /]# exit
exit
```
