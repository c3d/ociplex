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

Note that the test suite is sensitive to the environment, and getting it to
pass fully is difficult. Current results of this test suite on my machine:

* `crun`:
  ```
  Suites:   ​25 failed​, ​25 passed​, ​8 skip​, ​58 of 58 completed
  Asserts:   ​ ​​​70 failed​, ​3357 passed​, ​466 skip​, ​of 3893
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
