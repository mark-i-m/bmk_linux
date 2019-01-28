//! Routines for parsing /proc files on Linux 4.4.0 efficiently for use in benchmarking.
//!
//! Each submodule parses a particular file in the procfs. A struct is defined in each module
//! containing a field for each field in the appropriate procfs file. These are parsed into the
//! nearest rust type. To parse a file, call the `read` method defined on the struct, providing any
//! arguements as necessary. Each `read` method uses a constant-sized buffer of 1 page (4KB) so as
//! to avoid disturbing memory too much.
//!
//! Read the man page for procfs (`man procfs`) for more info about the meaning of a file's
//! contents or a struct's fields.

#[macro_use]
mod parsing;

pub mod meminfo;
pub mod pid;
