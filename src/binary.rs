


pub fn get_bits_msb(number: u16, idx1: u8, idx2: u8) -> u16 {
    let msb_low = idx1.min(idx2);
    let msb_high = idx1.max(idx2);

    let lsb_low  = 15 - msb_high;
    let lsb_high = 15 - msb_low;

    let width = lsb_high - lsb_low + 1;

    (number >> lsb_low) & ((1 << width) - 1)
}

pub fn get_bits_lsb(number: u16, idx1: u8, idx2: u8) -> u16 {
    let lsb_low  = idx1.min(idx2);
    let lsb_high = idx1.max(idx2);

    let width = lsb_high - lsb_low + 1;

    (number >> lsb_low) & ((1 << width) - 1)
}