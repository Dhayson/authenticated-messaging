#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Level
{
    Debug,
    Info,
    Normal,
}

pub fn log(level: Level, log: &str) -> Option<()>
{
    if level >= GLOBAL_LEVEL
    {
        eprintln!("{}", log);
    }
    Some(())
}

const GLOBAL_LEVEL: Level = Level::Debug;
