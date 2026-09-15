use crate::GeneratorData;

#[derive(Clone)]
pub struct Gene {
    pub id: u8,
    pub x: u8,
    pub y: u8,
    pub rotated: bool,
    pub placed: bool,
}

impl From<GeneratorData> for Gene {
    fn from(value: GeneratorData) -> Self {
        Self {
            id: value.id,
            x: 0,
            y: 0,
            rotated: false,
            placed: false,
        }
    }
}
