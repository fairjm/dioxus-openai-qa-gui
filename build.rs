use std::io;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("favicon-32x32.ico")
            .compile()?;
    }
    Ok(())
}
