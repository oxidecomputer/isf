// Copyright 2024 Oxide Computer Company

macro_rules! gen_bit {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_bit_ $width >](reg: $width, offset: usize) -> bool {
                (reg & (1 << (offset as $width))) != 0
            }
            pub fn [< set_bit_ $width >](reg: $width, offset: usize, value: bool) -> $width {
                let mask = !(0b1 << (offset as $width));
                let v = (value as $width) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_bit!(u32);
gen_bit!(u64);

macro_rules! gen_u8 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u8_ $width >](reg: $width, offset: usize) -> u8 {
                let v = reg >> (offset as $width);
                v as u8
            }
            pub fn [< set_u8_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0xff << (offset as $width));
                let v = (value as $width) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u8!(u32);
gen_u8!(u64);

macro_rules! gen_u16 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u16_ $width >](reg: $width, offset: usize) -> u16 {
                let v = reg >> (offset as $width);
                v as u16
            }
            pub fn [< set_u16_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0xffff << (offset as $width));
                let v = (value as $width) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u16!(u32);
gen_u16!(u64);

macro_rules! gen_u32 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u32_ $width >](reg: $width, offset: usize) -> u32 {
                let v = reg >> (offset as $width);
                v as u32
            }
            pub fn [< set_u32_ $width >](reg: $width, offset: usize, value: u32) -> $width {
                let mask = !(0xffffffff << (offset as $width));
                let v = (value as $width) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u32!(u32);
gen_u32!(u64);

macro_rules! gen_u2 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u2_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b11
            }
        }
        paste::item! {
            pub fn [< set_u2_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b11 << (offset as $width));
                let v = ((value as $width) & 0b11) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u2!(u32);
gen_u2!(u64);

macro_rules! gen_u3 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u3_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b1111
            }
            pub fn [< set_u3_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b111 << (offset as $width));
                let v = ((value as $width) & 0b111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u3!(u32);
gen_u3!(u64);

macro_rules! gen_u4 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u4_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b1111
            }
            pub fn [< set_u4_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b1111 << (offset as $width));
                let v = ((value as $width) & 0b1111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u4!(u32);
gen_u4!(u64);

macro_rules! gen_u5 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u5_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b11111
            }
            pub fn [< set_u5_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b11111 << (offset as $width));
                let v = ((value as $width) & 0b11111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u5!(u32);
gen_u5!(u64);

macro_rules! gen_u6 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u6_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b111111
            }
            pub fn [< set_u6_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b111111 << (offset as $width));
                let v = ((value as $width) & 0b111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u6!(u32);
gen_u6!(u64);

macro_rules! gen_u7 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u7_ $width >](reg: $width, offset: usize) -> u8 {
                let v = [< get_u8_ $width >](reg, offset);
                v & 0b1111111
            }
            pub fn [< set_u7_ $width >](reg: $width, offset: usize, value: u8) -> $width {
                let mask = !(0b1111111 << (offset as $width));
                let v = ((value as $width) & 0b1111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u7!(u32);
gen_u7!(u64);

macro_rules! gen_u9 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u9_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b111111111
            }
            pub fn [< set_u9_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b111111111 << (offset as $width));
                let v = ((value as $width) & 0b111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u9!(u32);
gen_u9!(u64);

macro_rules! gen_u10 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u10_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b1111111111
            }
            pub fn [< set_u10_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b1111111111 << (offset as $width));
                let v = ((value as $width) & 0b1111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u10!(u32);
gen_u10!(u64);

macro_rules! gen_u11 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u11_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b11111111111
            }
            pub fn [< set_u11_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b11111111111 << (offset as $width));
                let v = ((value as $width) & 0b11111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u11!(u32);
gen_u11!(u64);

macro_rules! gen_u12 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u12_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b111111111111
            }
            pub fn [< set_u12_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b111111111111 << (offset as $width));
                let v = ((value as $width) & 0b111111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u12!(u32);
gen_u12!(u64);

macro_rules! gen_u13 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u13_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b1111111111111
            }
            pub fn [< set_u13_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b1111111111111 << (offset as $width));
                let v = ((value as $width) & 0b1111111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u13!(u32);
gen_u13!(u64);

macro_rules! gen_u14 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u14_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b11111111111111
            }
            pub fn [< set_u14_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b11111111111111 << (offset as $width));
                let v = ((value as $width) & 0b11111111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u14!(u32);
gen_u14!(u64);

macro_rules! gen_u15 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u15_ $width >](reg: $width, offset: usize) -> u16 {
                let v = [< get_u16_ $width >](reg, offset);
                v & 0b111111111111111
            }
            pub fn [< set_u15_ $width >](reg: $width, offset: usize, value: u16) -> $width {
                let mask = !(0b111111111111111 << (offset as $width));
                let v = ((value as $width) & 0b111111111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u15!(u32);
gen_u15!(u64);

macro_rules! gen_u19 {
    ($width:ident) => {
        paste::item! {
            pub fn [< get_u19_ $width >](reg: $width, offset: usize) -> u32 {
                let v = [< get_u32_ $width >](reg, offset);
                v & 0b1111111111111111111
            }
            pub fn [< set_u19_ $width >](reg: $width, offset: usize, value: u32) -> $width {
                let mask = !(0b1111111111111111111 << (offset as $width));
                let v = ((value as $width) & 0b1111111111111111111) << (offset as $width);
                (reg & mask) | v
            }
        }
    };
}
gen_u19!(u32);
gen_u19!(u64);
