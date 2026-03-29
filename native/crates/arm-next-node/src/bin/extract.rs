use arm_next_core::convert_file_parallel;
use std::env;
use std::path::PathBuf;

fn main() {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".next/server/app"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("public/ai-md"));

    let max_tokens: Option<u32> = env::var("ARM_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse().ok());

    match convert_file_parallel(&input, &output, None, max_tokens, None) {
        Ok(n) => println!("arm-next-extract: wrote {n} markdown files under {}", output.display()),
        Err(e) => {
            eprintln!("arm-next-extract: {e}");
            std::process::exit(1);
        }
    }
}
