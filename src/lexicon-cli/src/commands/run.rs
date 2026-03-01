use anyhow::Result;

pub fn run_command(args: Vec<String>) -> Result<()> {
    println!("Running project with args: {:?}", args);
    Ok(())
}
