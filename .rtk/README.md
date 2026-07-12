# Project-local rtk configuration for lexFlex

This directory allows custom output filters for commands used in this project.

## Example usage

1. Create a `.toml` file in `filters/` describing rules for a command.
2. Run `rtk trust` in this directory (once).
3. Use `rtk <your-command>` — it will apply the filter.

See global rtk docs or `rtk verify` for details.

## Suggested filters for lexFlex (future)

- Cargo output cleaner (keep only errors + summary, drop lots of warnings we already know about).
- RON validation or data loading logs.
- Benchmark script output.

For now the global config + built-in rtk rules are sufficient.
