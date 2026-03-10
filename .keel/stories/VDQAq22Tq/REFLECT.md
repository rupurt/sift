# Reflection - Provide Static Linux Executable

We have enhanced the portability of `sift` on Linux by providing a fully static executable using the `musl` libc implementation. By adding the `x86_64-unknown-linux-musl` target to our `cargo-dist` configuration and updating the release workflow to install `musl-tools`, we now offer a binary that can run on any Linux distribution without depending on the system's version of `glibc`.
