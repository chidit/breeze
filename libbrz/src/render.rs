/// Renderer `Coord`-inate
///
/// This is logically different from the text `Coord`-inate,

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: u32,
    pub bg: u32,
    pub style: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

pub struct ColorMap {
    pub default_bg: u32,
    pub default_fg: u32,
}

impl ColorMap {
    pub fn default_style(&self) -> Style {
        Style {
            fg: self.default_fg,
            bg: self.default_bg,
            style: 0,
        }
    }
}

impl std::ops::Add<Coord> for Coord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Coord {
            y: self.y + other.y,
            x: self.x + other.x,
        }
    }
}

impl Coord {
    pub fn add_x(mut self, x: usize) -> Coord {
        self.x = self.x.saturating_add(x);
        self
    }

    pub fn add_y(mut self, y: usize) -> Coord {
        self.y = self.y.saturating_add(y);
        self
    }

    pub fn center(self) -> Self {
        Self {
            x: self.x / 2,
            y: self.y / 2,
        }
    }

    fn is_inside_dimensions(self, other: Coord) -> bool {
        self.y < other.y && self.x < other.x
    }
    fn is_inside(self, other: Rect) -> bool {
        self.is_inside_dimensions(other.dimensions)
    }
}

pub trait Renderer {
    fn color_map(&self) -> &ColorMap;
    fn dimensions(&self) -> Coord;
    /// The whole dimensions as a `Rect` that starts at (0, 0)
    fn dimensions_rect(&self) -> Rect {
        Rect {
            offset: Coord { x: 0, y: 0 },
            dimensions: self.dimensions(),
        }
    }
    fn put(&mut self, coord: Coord, ch: char, style: Style);

    fn print(&mut self, coord: Coord, text: &str, style: Style) {
        let dims = self.dimensions();
        for (i, ch) in text.chars().enumerate() {
            let coord = coord.add_x(i);
            if !coord.is_inside_dimensions(dims) {
                break;
            }
            self.put(coord, ch, style);
        }
    }

    fn set_cursor(&mut self, coord: Option<Coord>);
}

impl<T> Renderer for &mut T
where
    T: Renderer + ?Sized,
{
    fn color_map(&self) -> &ColorMap {
        (**self).color_map()
    }
    fn dimensions(&self) -> Coord {
        (**self).dimensions()
    }
    fn put(&mut self, coord: Coord, ch: char, style: Style) {
        (**self).put(coord, ch, style)
    }
    fn set_cursor(&mut self, coord: Option<Coord>) {
        (**self).set_cursor(coord)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub offset: Coord,
    pub dimensions: Coord,
}

impl Rect {
    pub fn split_verticaly_at(self, x: isize) -> (Rect, Rect) {
        let x = if x < 0 {
            (self.dimensions.x as isize + x) as usize
        } else if x > 0 {
            x as usize
        } else {
            panic!("Can't split at 0")
        };

        assert!(x < self.dimensions.x);
        (
            Rect {
                offset: self.offset,
                dimensions: Coord {
                    x,
                    y: self.dimensions.y,
                },
            },
            Rect {
                offset: Coord {
                    x: self.offset.x + x,
                    y: self.offset.y,
                },
                dimensions: Coord {
                    x: self.dimensions.x - x,
                    y: self.dimensions.y,
                },
            },
        )
    }

    pub fn split_horizontaly_at(self, y: isize) -> (Rect, Rect) {
        let y = if y < 0 {
            (self.dimensions.y as isize + y) as usize
        } else if y > 0 {
            y as usize
        } else {
            panic!("Can't split at 0")
        };

        assert!(y < self.dimensions.y);

        (
            Rect {
                offset: self.offset,
                dimensions: Coord {
                    x: self.dimensions.x,
                    y,
                },
            },
            Rect {
                offset: Coord {
                    x: self.offset.x,
                    y: self.offset.y + y,
                },
                dimensions: Coord {
                    x: self.dimensions.x,
                    y: self.dimensions.y - y,
                },
            },
        )
    }

    pub fn to_renderer<'r, R>(self, r: &'r mut R) -> View<'r, R>
    where
        R: Renderer,
    {
        View {
            rect: self,
            backend: r,
        }
    }
}

/// A rectangual view over another `Renderer`
pub struct View<'r, R> {
    rect: Rect,
    backend: &'r mut R,
}

impl<'r, R> Renderer for View<'r, R>
where
    R: Renderer,
{
    fn color_map(&self) -> &ColorMap {
        self.backend.color_map()
    }

    fn dimensions(&self) -> Coord {
        self.rect.dimensions
    }

    fn put(&mut self, coord: Coord, ch: char, style: Style) {
        if coord.is_inside(self.rect) {
            self.backend.put(coord + self.rect.offset, ch, style)
        }
    }
    fn set_cursor(&mut self, coord: Option<Coord>) {
        self.backend.set_cursor(coord.map(|c| c + self.rect.offset))
    }
}