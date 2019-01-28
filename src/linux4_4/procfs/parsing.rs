//! Generate a parser

/// Generates a parser for files that just contain a big list of numbers (e.g. /proc/[pid]/stat).
macro_rules! list_parser {
    (struct $struct:ident; $path_fn:ident($($args:ident : $pty:ty),*); $($field:ident : $ty:ty),+,) => {
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

                // split over ASCII ' ' character
                let mut reader = buf.split(b' ');

                $(
                    let $field: $ty = {
                        let v = reader.next().unwrap()?;
                        let s = ::std::str::from_utf8(&v).unwrap().trim();
                        s.parse().unwrap()
                    };
                )*

                Ok($struct {
                    $(
                        $field
                    ),*
                })
            }
        }
    }
}
