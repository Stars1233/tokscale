use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Green,
    Halloween,
    Teal,
    Blue,
    Pink,
    Purple,
    Orange,
    Monochrome,
    YlGnBu,
}

impl ThemeName {
    pub fn all() -> &'static [ThemeName] {
        &[
            ThemeName::Green,
            ThemeName::Halloween,
            ThemeName::Teal,
            ThemeName::Blue,
            ThemeName::Pink,
            ThemeName::Purple,
            ThemeName::Orange,
            ThemeName::Monochrome,
            ThemeName::YlGnBu,
        ]
    }

    pub fn next(self) -> ThemeName {
        let themes = Self::all();
        let idx = themes.iter().position(|&t| t == self).unwrap_or(0);
        themes[(idx + 1) % themes.len()]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeName::Green => "green",
            ThemeName::Halloween => "halloween",
            ThemeName::Teal => "teal",
            ThemeName::Blue => "blue",
            ThemeName::Pink => "pink",
            ThemeName::Purple => "purple",
            ThemeName::Orange => "orange",
            ThemeName::Monochrome => "monochrome",
            ThemeName::YlGnBu => "ylgnbu",
        }
    }
}

impl std::str::FromStr for ThemeName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "green" => Ok(ThemeName::Green),
            "halloween" => Ok(ThemeName::Halloween),
            "teal" => Ok(ThemeName::Teal),
            "blue" => Ok(ThemeName::Blue),
            "pink" => Ok(ThemeName::Pink),
            "purple" => Ok(ThemeName::Purple),
            "orange" => Ok(ThemeName::Orange),
            "monochrome" => Ok(ThemeName::Monochrome),
            "ylgnbu" => Ok(ThemeName::YlGnBu),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: ThemeName,
    pub colors: [Color; 5],
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub highlight: Color,
    pub muted: Color,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        let colors = match name {
            ThemeName::Green => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(14, 68, 41),
                Color::Rgb(0, 109, 50),
                Color::Rgb(38, 166, 65),
                Color::Rgb(57, 211, 83),
            ],
            ThemeName::Halloween => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(99, 29, 0),
                Color::Rgb(153, 68, 0),
                Color::Rgb(255, 123, 0),
                Color::Rgb(255, 166, 39),
            ],
            ThemeName::Teal => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(0, 68, 68),
                Color::Rgb(0, 109, 109),
                Color::Rgb(38, 166, 154),
                Color::Rgb(57, 211, 196),
            ],
            ThemeName::Blue => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(14, 41, 68),
                Color::Rgb(0, 50, 109),
                Color::Rgb(38, 65, 166),
                Color::Rgb(57, 83, 211),
            ],
            ThemeName::Pink => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(68, 14, 41),
                Color::Rgb(109, 0, 50),
                Color::Rgb(166, 38, 65),
                Color::Rgb(211, 57, 83),
            ],
            ThemeName::Purple => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(41, 14, 68),
                Color::Rgb(50, 0, 109),
                Color::Rgb(65, 38, 166),
                Color::Rgb(83, 57, 211),
            ],
            ThemeName::Orange => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(68, 41, 14),
                Color::Rgb(109, 50, 0),
                Color::Rgb(166, 65, 38),
                Color::Rgb(211, 83, 57),
            ],
            ThemeName::Monochrome => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(50, 55, 62),
                Color::Rgb(80, 85, 92),
                Color::Rgb(140, 145, 152),
                Color::Rgb(200, 205, 212),
            ],
            ThemeName::YlGnBu => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(34, 94, 168),
                Color::Rgb(29, 145, 192),
                Color::Rgb(65, 182, 196),
                Color::Rgb(127, 205, 187),
            ],
        };

        Self {
            name,
            colors,
            background: Color::Rgb(13, 17, 23),
            foreground: Color::Rgb(201, 209, 217),
            border: Color::Rgb(48, 54, 61),
            highlight: colors[4],
            muted: Color::Rgb(139, 148, 158),
        }
    }
}
