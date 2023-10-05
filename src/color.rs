use serde::{Serialize, Deserialize, Serializer, Deserializer, de::Error};
use crossterm::style::Color;

#[derive(Debug, Copy, Clone)]
pub struct RGBColor{
    r: u8,
    g: u8,
    b: u8,
}

impl<'de> Deserialize<'de> for RGBColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let error = || D::Error::custom("Color must be 6 hex digits");

        let text: String = Deserialize::deserialize(deserializer)?;
        if text.len() != 6 { return Err(error()); }
        let r = u8::from_str_radix(text.get(0..2).ok_or_else(|| error())?, 16).map_err(|_| error())?;
        let g = u8::from_str_radix(text.get(2..4).ok_or_else(|| error())?, 16).map_err(|_| error())?;
        let b = u8::from_str_radix(text.get(4..6).ok_or_else(|| error())?, 16).map_err(|_| error())?;

        Ok(Self{r, g, b})
    }
}

impl Serialize for RGBColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b);
        serializer.serialize_str(&text)
    }
}

impl From<RGBColor> for Color {
    fn from(rgb: RGBColor) -> Color {
        Color::Rgb{r: rgb.r, g: rgb.g, b: rgb.b}
    }
}
