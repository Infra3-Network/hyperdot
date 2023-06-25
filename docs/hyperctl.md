# Hyperctl Design

## Overview

hyperctl is a command line tool for hyperdot that helps users, system administrators, and developers achieve better ergonomics when using hyperdot. Specifically, hyperctl provides the following capabilities
1. codegen for runtime metadata, such as polkadot runtime
2. Initialize the data engine
3. View the status of streaming
4. View the status of storage
5. And more to incoming....

## How it works!

hyperctl has many subcommands, so first, it is necessary to implement subcommands efficiently in rust. We use [clap](https://docs.rs/clap/4.3.8/clap/index.html) to do this, and here is an example of how the [wasmer cli](https://github.com/wasmerio/wasmer/blob/master/lib/cli/src/commands/run.rs#L59) uses clap.

