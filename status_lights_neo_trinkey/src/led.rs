use smart_leds::RGB8;
use status_lights_messages::{LedColor, LedColorTimed};

#[derive(Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn default() -> Color {
        Color {
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    pub fn to_rgb(&self) -> Option<RGB8> {
        if self.red == 0 && self.green == 0 && self.blue == 0 {
            None
        } else {
            Some(RGB8::new(self.red, self.green, self.blue))
        }
    }
}

impl From<LedColor> for Color {
    fn from(led_color: LedColor) -> Self {
        Self {
            red: led_color.red,
            green: led_color.green,
            blue: led_color.blue,
        }
    }
}

#[derive(Debug)]
pub struct ColorTimed {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub ms_remaining: u32,
}

impl ColorTimed {
    pub const fn default() -> ColorTimed {
        ColorTimed {
            red: 0,
            green: 0,
            blue: 0,
            ms_remaining: 0,
        }
    }

    pub fn to_rgb(&self) -> Option<RGB8> {
        if self.red == 0 && self.green == 0 && self.blue == 0 {
            None
        } else {
            Some(RGB8::new(self.red, self.green, self.blue))
        }
    }

    pub fn reduce_time(&mut self, ms: u32) {
        if self.ms_remaining == 0 {
            return; // 0 times do not reduce
        }
        self.ms_remaining = self.ms_remaining.saturating_sub(ms);
        if self.ms_remaining == 0 {
            self.red = 0;
            self.green = 0;
            self.blue = 0;
        }
    }
}

impl From<LedColorTimed> for ColorTimed {
    fn from(led_color_timed: LedColorTimed) -> Self {
        Self {
            red: led_color_timed.red,
            green: led_color_timed.green,
            blue: led_color_timed.blue,
            ms_remaining: (led_color_timed.seconds as u32) * 1000,
        }
    }
}
