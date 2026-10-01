//! Bundled instructions always travel with the executable; never install automatically.
pub const MAIN: &str = include_str!("../skills/gpui-visual/SKILL.md");
pub const CONSUMER: &str = include_str!("../skills/gpui-visual/references/consumer.md");
pub fn get(reference: Option<&str>) -> anyhow::Result<&'static str> {
    match reference {
        None => Ok(MAIN),
        Some("consumer") => Ok(CONSUMER),
        Some(_) => anyhow::bail!("Unknown skill reference; available: consumer"),
    }
}
