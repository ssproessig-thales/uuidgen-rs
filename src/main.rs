//! A Rust clone of the GNU/util-linux `uuidgen` command-line tool.
//!
//! Supports all switches provided by util-linux `uuidgen`:
//!   -r, --random        generate random-based uuid (v4)
//!   -t, --time          generate time-based uuid (v1)
//!   -n, --namespace ns  generate hash-based uuid in this namespace
//!   -N, --name name     generate hash-based uuid from this name
//!   -m, --md5           generate md5 hash (v3)
//!   -s, --sha1          generate sha1 hash (v5)
//!   -x, --hex           interpret name as hex string
//!   -h, --help          display this help
//!   -V, --version       display version

use clap::{ArgAction, ArgGroup, Parser};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "uuidgen",
    version,
    about = "Create a new UUID value.",
    disable_help_flag = false
)]
#[command(group(
    ArgGroup::new("mode")
        .args(["random", "time", "md5", "sha1"])
        .multiple(false)
))]
struct Args {
    /// generate random-based uuid
    #[arg(short = 'r', long = "random", action = ArgAction::SetTrue)]
    random: bool,

    /// generate time-based uuid
    #[arg(short = 't', long = "time", action = ArgAction::SetTrue)]
    time: bool,

    /// generate hash-based uuid in this namespace
    #[arg(short = 'n', long = "namespace", value_name = "ns")]
    namespace: Option<String>,

    /// generate hash-based uuid from this name
    #[arg(short = 'N', long = "name", value_name = "name")]
    name: Option<String>,

    /// generate md5 hash
    #[arg(short = 'm', long = "md5", action = ArgAction::SetTrue)]
    md5: bool,

    /// generate sha1 hash
    #[arg(short = 's', long = "sha1", action = ArgAction::SetTrue)]
    sha1: bool,

    /// interpret name as hex string
    #[arg(short = 'x', long = "hex", action = ArgAction::SetTrue)]
    hex: bool,
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(uuid) => println!("{uuid}"),
        Err(e) => {
            eprintln!("uuidgen: {e}");
            std::process::exit(1);
        }
    }
}

fn run(args: &Args) -> Result<Uuid, String> {
    // md5/sha1 require both --namespace and --name.
    if args.md5 || args.sha1 {
        let namespace = resolve_namespace(
            args.namespace
                .as_deref()
                .ok_or("--namespace and --name arguments are required for --md5 or --sha1")?,
        )?;
        let name = args
            .name
            .as_deref()
            .ok_or("--namespace and --name arguments are required for --md5 or --sha1")?;

        // -x/--hex: interpret the name as a hex string of raw bytes.
        let bytes = if args.hex {
            decode_hex(name)?
        } else {
            name.as_bytes().to_vec()
        };

        let uuid = if args.md5 {
            Uuid::new_v3(&namespace, &bytes)
        } else {
            Uuid::new_v5(&namespace, &bytes)
        };
        return Ok(uuid);
    }

    if args.namespace.is_some() || args.name.is_some() {
        return Err("--namespace and --name arguments require --md5 or --sha1".into());
    }

    if args.time {
        let node_id = get_node_id();
        return Ok(Uuid::now_v1(&node_id));
    }

    // -r/--random is the default when nothing else is specified.
    Ok(Uuid::new_v4())
}

/// Resolve a namespace argument. Accepts the well-known aliases used by
/// util-linux (@dns, @url, @oid, @x500) as well as an arbitrary UUID string.
fn resolve_namespace(ns: &str) -> Result<Uuid, String> {
    match ns {
        "@dns" => Ok(Uuid::NAMESPACE_DNS),
        "@url" => Ok(Uuid::NAMESPACE_URL),
        "@oid" => Ok(Uuid::NAMESPACE_OID),
        "@x500" => Ok(Uuid::NAMESPACE_X500),
        other => Uuid::parse_str(other)
            .map_err(|_| format!("invalid namespace: '{other}' (expected @dns/@url/@oid/@x500 or a UUID)")),
    }
}

/// Decode a hex string into raw bytes, ignoring optional whitespace.
fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() % 2 != 0 {
        return Err("hex string must have an even number of digits".into());
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&cleaned[i..i + 2], 16)
                .map_err(|_| format!("invalid hex string: '{s}'"))
        })
        .collect()
}

/// Get the node id (MAC address) for time-based UUIDs. Falls back to a random
/// multicast MAC if no hardware address is available, matching util-linux.
fn get_node_id() -> [u8; 6] {
    if let Ok(Some(mac)) = mac_address::get_mac_address() {
        let b = mac.bytes();
        if b.iter().any(|&x| x != 0) {
            return b;
        }
    }
    let mut default: [u8; 6] = rand::random();
    default[0] |= 0x01; // set multicast bit
    default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_is_v4() {
        let args = Args {
            random: true,
            time: false,
            namespace: None,
            name: None,
            md5: false,
            sha1: false,
            hex: false,
        };
        let uuid = run(&args).unwrap();
        assert_eq!(uuid.get_version_num(), 4);
    }

    #[test]
    fn default_is_random_v4() {
        let args = Args {
            random: false,
            time: false,
            namespace: None,
            name: None,
            md5: false,
            sha1: false,
            hex: false,
        };
        assert_eq!(run(&args).unwrap().get_version_num(), 4);
    }

    #[test]
    fn time_is_v1() {
        let args = Args {
            random: false,
            time: true,
            namespace: None,
            name: None,
            md5: false,
            sha1: false,
            hex: false,
        };
        assert_eq!(run(&args).unwrap().get_version_num(), 1);
    }

    #[test]
    fn md5_matches_known_value() {
        // uuidgen -m -n @dns -N www.example.com
        let args = Args {
            random: false,
            time: false,
            namespace: Some("@dns".into()),
            name: Some("www.example.com".into()),
            md5: true,
            sha1: false,
            hex: false,
        };
        let uuid = run(&args).unwrap();
        assert_eq!(uuid.get_version_num(), 3);
        assert_eq!(uuid.to_string(), "5df41881-3aed-3515-88a7-2f4a814cf09e");
    }

    #[test]
    fn sha1_matches_known_value() {
        // uuidgen -s -n @dns -N www.example.com
        let args = Args {
            random: false,
            time: false,
            namespace: Some("@dns".into()),
            name: Some("www.example.com".into()),
            md5: false,
            sha1: true,
            hex: false,
        };
        let uuid = run(&args).unwrap();
        assert_eq!(uuid.get_version_num(), 5);
        assert_eq!(uuid.to_string(), "2ed6657d-e927-568b-95e1-2665a8aea6a2");
    }

    #[test]
    fn hex_name_decodes() {
        // "www.example.com" as hex.
        let hex_name = "7777772e6578616d706c652e636f6d";
        let args = Args {
            random: false,
            time: false,
            namespace: Some("@dns".into()),
            name: Some(hex_name.into()),
            md5: true,
            sha1: false,
            hex: true,
        };
        let uuid = run(&args).unwrap();
        // Must equal the non-hex md5 result for the same bytes.
        assert_eq!(uuid.to_string(), "5df41881-3aed-3515-88a7-2f4a814cf09e");
    }

    #[test]
    fn arbitrary_uuid_namespace() {
        let args = Args {
            random: false,
            time: false,
            namespace: Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8".into()),
            name: Some("test".into()),
            md5: false,
            sha1: true,
            hex: false,
        };
        assert_eq!(run(&args).unwrap().get_version_num(), 5);
    }

    #[test]
    fn md5_without_name_errors() {
        let args = Args {
            random: false,
            time: false,
            namespace: Some("@dns".into()),
            name: None,
            md5: true,
            sha1: false,
            hex: false,
        };
        assert!(run(&args).is_err());
    }
}
