use image::Rgb;
pub const BACKGROUND: Rgb<u8> = Rgb([0, 0, 0]);
pub const MAIN_LINES: Rgb<u8> = Rgb([127, 127, 127]);
pub const GRID_LINES: Rgb<u8> = Rgb([63, 63, 63]);
pub const BRIGHTER_GRID_LINES: Rgb<u8> = Rgb([95, 95, 95]);
pub const TEXT: Rgb<u8> = Rgb([127, 127, 127]);

pub const TEMP_MIN: Rgb<u8> = Rgb([0, 148, 255]);
pub const TEMP_MAX: Rgb<u8> = Rgb([255, 0, 0]);
pub const TEMP: Rgb<u8> = Rgb([255, 0, 0]);
pub const TEMP_FEELS_LIKE: Rgb<u8> = Rgb([0, 255, 33]);
pub const TEMP_WET_BULB: Rgb<u8> = Rgb([0, 148, 255]);

pub const RAIN: Rgb<u8> = Rgb([0, 148, 255]);
pub const SNOW: Rgb<u8> = Rgb([216, 239, 255]);
pub const POP: Rgb<u8> = Rgb([0, 148, 255]);

pub const UVI_LOW: [u8; 3] = [0, 255, 33];
pub const UVI_MEDIUM: [u8; 3] = [255, 255, 33];
pub const UVI_HIGH: [u8; 3] = [255, 0, 33];

pub const GUST_LOW: [u8; 3] = [70, 119, 67];
pub const GUST_MEDIUM: [u8; 3] = [118, 118, 62];
pub const GUST_HIGH: [u8; 3] = [122, 67, 62];
pub const GUST_VERY_HIGH: [u8; 3] = [103, 78, 122];
pub const WIND_LOW: [u8; 3] = [0, 255, 33];
pub const WIND_MEDIUM: [u8; 3] = [255, 255, 33];
pub const WIND_HIGH: [u8; 3] = [255, 0, 33];
pub const WIND_VERY_HIGH: [u8; 3] = [188, 66, 255];
