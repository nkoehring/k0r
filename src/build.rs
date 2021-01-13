use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env()?;
    ructe.statics()?.add_files("src/static")?;
    ructe.compile_templates("src/templates")
}
