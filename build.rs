use std::io::Result;
fn main() -> Result<()> {
    println!("building proto");
    prost_build::compile_protos(&["src/audio.proto"], &["src/"])?;
    Ok(())
}