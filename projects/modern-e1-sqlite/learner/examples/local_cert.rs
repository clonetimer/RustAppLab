//! Generate a localhost-only teaching certificate in a NEW private directory.
use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args_os().nth(1).ok_or("provide a new private directory")?);
    fs::create_dir(&dir)?;
    let pair = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    fs::write(dir.join("cert.der"), pair.cert.der().as_ref())?;
    fs::write(dir.join("cert.pem"), pair.cert.pem())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(dir.join("key.pkcs8.der"))?.write_all(&pair.signing_key.serialize_der())?;
    println!("Local teaching certificate created. Keep the private key private; never install it system-wide.");
    Ok(())
}
