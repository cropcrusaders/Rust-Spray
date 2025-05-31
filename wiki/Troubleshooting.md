# Troubleshooting

This page collects common build and runtime issues.

## Cross compilation linker error

When building with `cross` you might encounter an error like:

```text
/usr/aarch64-linux-gnu/lib/../lib/Scrt1.o: Relocations in generic ELF (EM: 183)
```

This usually means the build fell back to the host toolchain. Ensure the `cross` container started correctly (watch for the message `Falling back to cargo on the host`) and that the appropriate `*-linux-gnu` cross compiler packages are installed. Rebuild the Docker image if necessary and run `cross build` again.
