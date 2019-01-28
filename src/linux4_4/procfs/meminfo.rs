//! Routines for parsing /proc/meminfo.

use std::path::PathBuf;

pub fn get_path() -> PathBuf {
    PathBuf::from("/proc/meminfo")
}

/// Some number of Kilobytes
#[derive(Debug, Clone, Copy)]
pub struct KiloBytes(usize);

impl KiloBytes {
    pub fn kilobytes(self) -> usize {
        self.0
    }
}

impl std::str::FromStr for KiloBytes {
    type Err = std::num::ParseIntError;

    // Parse a string like "93 kB"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ');
        let value: usize = parts.next().expect("expected value").parse().unwrap();
        let as_kb = match parts
            .next()
            .expect("expected units")
            .to_lowercase()
            .as_ref()
        {
            "kb" => value,
            "mb" => 1024 * value,
            "gb" => 1024 * 1024 * value,
            "tb" => 1024 * 1024 * 1024 * value,
            "pb" => 1024 * 1024 * 1024 * 1024 * value,
            other => panic!("unexpected units: `{}`", other),
        };

        Ok(KiloBytes(as_kb))
    }
}

/// Generates a parser for files that contains "maps" of values (e.g. /proc/meminfo).
macro_rules! map_parser {
    (struct $struct:ident; $path_fn:ident($($args:ident : $pty:ty),*); $($lit:literal $field:ident : $ty:ty),+,) => {
        #[derive(Clone, Debug)]
        pub struct $struct {
            $(
                pub $field: $ty
            ),*
        }

        impl $struct {
            pub fn read($($args: $pty),*) -> Result<$struct, ::std::io::Error> {
                use std::io::{BufRead, BufReader};

                const BUFFER_CAP: usize = 4096; // Make all buffers 1 page

                let path = $path_fn($($args),*);
                let file = std::fs::File::open(&path)?;

                // Use a bounded-size buffer to limit disruption of the measurement
                let buf = BufReader::with_capacity(BUFFER_CAP, file);

                // split into different lines.
                let reader = buf.lines();

                // compute the length needed.
                #[allow(clippy::let_and_return)]
                const NUM_ENTRIES: usize = {
                    0
                    $(+ {
                        let $field = 1;
                        $field
                    })+
                };

                let mut map = std::collections::HashMap::with_capacity(NUM_ENTRIES);

                for line in reader {
                    // Split the line at the `:`
                    let line = line.expect("unable to read procfs");
                    let mut split = line.split(":");

                    let name = split.next().expect("Malformed input").trim().to_owned();
                    let value = split.next().expect("Malformed input").trim().to_owned();

                    map.insert(name, value);
                }

                Ok($struct {
                    $(
                        $field: {
                            map.get($lit).map(|s| s.parse::<$ty>().unwrap()).expect("field not found")
                        }
                    ),*
                })
            }
        }
    }
}

map_parser! {
    struct ProcMeminfo;

    get_path();

    "MemTotal"          mem_total: KiloBytes,
    "MemFree"           mem_free: KiloBytes,
    "MemAvailable"      mem_available: KiloBytes,
    "Buffers"           buffers: KiloBytes,
    "Cached"            cached: KiloBytes,
    "SwapCached"        swap_cached: KiloBytes,
    "Active"            active: KiloBytes,
    "Inactive"          inactive: KiloBytes,
    "Active(anon)"      active_anon: KiloBytes,
    "Inactive(anon)"    inactive_anon: KiloBytes,
    "Active(file)"      active_file: KiloBytes,
    "Inactive(file)"    inactive_file: KiloBytes,
    "Unevictable"       unevictable: KiloBytes,
    "Mlocked"           mlocked: KiloBytes,
    "SwapTotal"         swap_total: KiloBytes,
    "SwapFree"          swap_free: KiloBytes,
    "Dirty"             dirty: KiloBytes,
    "Writeback"         writeback: KiloBytes,
    "AnonPages"         anon_pages: KiloBytes,
    "Mapped"            mapped: KiloBytes,
    "Shmem"             shmem: KiloBytes,
    "Slab"              slab: KiloBytes,
    "SReclaimable"      sreclaimable: KiloBytes,
    "SUnreclaim"        sunreclaimable: KiloBytes,
    "KernelStack"       kernel_stack: KiloBytes,
    "PageTables"        page_tables: KiloBytes,
    "NFS_Unstable"      nfs_unstable: KiloBytes,
    "Bounce"            bounce: KiloBytes,
    "WritebackTmp"      writeback_tmp: KiloBytes,
    "CommitLimit"       commit_limit: KiloBytes,
    "Committed_AS"      committed_as: KiloBytes,
    "VmallocTotal"      vmalloc_total: KiloBytes,
    "VmallocUsed"       vmalloc_used: KiloBytes,
    "VmallocChunk"      vmalloc_chunk: KiloBytes,
    "HardwareCorrupted" hardware_corrupted: KiloBytes,
    "AnonHugePages"     anon_huge_pages: KiloBytes,
    "CmaTotal"          cma_total: KiloBytes,
    "CmaFree"           cma_free: KiloBytes,
    "HugePages_Total"   huge_pages_total: usize,
    "HugePages_Free"    huge_pages_free: usize,
    "HugePages_Rsvd"    huge_pages_rsvd: usize,
    "HugePages_Surp"    huge_pages_surp: usize,
    "Hugepagesize"      huge_page_size: KiloBytes,
    "DirectMap4k"       direct_map_4k: KiloBytes,
    "DirectMap2M"       direct_map_2m: KiloBytes,
    "DirectMap1G"       direct_map_1g: KiloBytes,
}
