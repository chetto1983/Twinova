#!/bin/sh
# Runs a test binary with at most 4 GiB of address space, for mutation testing: a mutant that
# makes a loop spin fills the tests' traffic log until the runner dies with no result. Under
# the cap the allocation fails, the test aborts, and the mutant counts as caught.
ulimit -v 4194304
exec "$@"
