use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ArgsParser {
    #[arg(short, long, value_enum, default_value_t = GridSize::Small)]
    pub grid_size: GridSize,
    #[arg(short, long, value_enum, default_value_t = Speed::High)]
    pub speed: Speed,
    #[arg(short, long, value_enum, default_value_t = MovementKeyScheme::Arrows)]
    pub movement_key_scheme: MovementKeyScheme,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum GridSize {
    Small,
    Medium,
    Large,
}

impl GridSize {
    pub fn value(&self) -> f64 {
        match self {
            GridSize::Small => 0.7,
            GridSize::Medium => 0.85,
            GridSize::Large => 1.0,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Speed {
    Slow,
    Moderate,
    High,
}

impl Speed {
    pub fn value(&self) -> u64 {
        match self {
            Speed::Slow => 120,
            Speed::Moderate => 90,
            Speed::High => 60,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum MovementKeyScheme {
    Wsad,
    Arrows,
}
