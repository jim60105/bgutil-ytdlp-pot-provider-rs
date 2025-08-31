# CLI Migration Guide

This document provides a migration guide from the old dual-binary CLI to the new unified CLI.

## New Unified CLI Structure

The project now uses a single binary `bgutil-pot` with subcommands instead of two separate binaries.

### Command Comparison

#### Server Mode
| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `bgutil-pot-server` | `bgutil-pot server` | Default port and host unchanged |
| `bgutil-pot-server --port 8080` | `bgutil-pot server --port 8080` | All options preserved |
| `bgutil-pot-server --host 0.0.0.0` | `bgutil-pot server --host 0.0.0.0` | All options preserved |
| `bgutil-pot-server --verbose` | `bgutil-pot server --verbose` | All options preserved |

#### Generate Mode
| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `bgutil-pot-generate --content-binding "test"` | `bgutil-pot --content-binding "test"` | Default mode when no subcommand |
| `bgutil-pot-generate --proxy "http://proxy:8080"` | `bgutil-pot --proxy "http://proxy:8080"` | All options preserved |
| `bgutil-pot-generate --bypass-cache` | `bgutil-pot --bypass-cache` | All options preserved |
| `bgutil-pot-generate --verbose` | `bgutil-pot --verbose` | All options preserved |

#### Global Options
| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `bgutil-pot-server --version` | `bgutil-pot --version` | Works from any context |
| `bgutil-pot-generate --version` | `bgutil-pot --version` | Unified version command |
| `bgutil-pot-server --help` | `bgutil-pot server --help` | Context-specific help |
| `bgutil-pot-generate --help` | `bgutil-pot --help` | Global help shows all options |

## Key Improvements

1. **Unified Interface**: Single binary reduces complexity
2. **Intuitive Design**: Follows modern CLI patterns with subcommands
3. **Backward Compatibility**: All parameters and behaviors preserved
4. **Better Help**: Contextual help for each mode
5. **Conflict Prevention**: Clap prevents invalid parameter combinations

## Migration Notes

- All existing scripts and integrations continue to work by changing the binary name
- Output formats remain identical
- Error messages and exit codes are preserved
- All deprecated parameter handling remains the same