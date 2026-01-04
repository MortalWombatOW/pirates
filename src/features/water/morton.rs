/// Morton Codes (Z-Order Curve) utilities for Linear Quadtree.
/// 
/// Interleaves bits of x and y coordinates to produce a single integer key.
/// Ideally suited for quadtree storage and traversal.
/// 
/// Supports 16-bit coordinates (u16), resulting in 32-bit Morton codes (u32),
/// allowing for a grid size of up to 65536 x 65536.

/// Spreads the lower 16 bits of x into the odd bits of a 32-bit integer.
/// 0b_x1_x0 -> 0b_0_x1_0_x0
fn spread_bits_16(x: u16) -> u32 {
    let mut x = x as u32;
    x = (x | (x << 8)) & 0x00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F;
    x = (x | (x << 2)) & 0x33333333;
    x = (x | (x << 1)) & 0x55555555;
    x
}

/// Compacts the odd-positioned bits of a 32-bit integer back to the lower 16 bits.
/// Inverse of spread_bits_16.
fn compact_bits_32(mut x: u32) -> u16 {
    x = x & 0x55555555;
    x = (x | (x >> 1)) & 0x33333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF;
    x as u16
}

/// Encodes 2D coordinates into a Morton Code.
/// y is in the odd bits, x is in the even bits.
pub fn morton_encode(x: u16, y: u16) -> u32 {
    spread_bits_16(x) | (spread_bits_16(y) << 1)
}

/// Decodes a Morton Code back into 2D coordinates.
pub fn morton_decode(code: u32) -> (u16, u16) {
    let x = compact_bits_32(code);
    let y = compact_bits_32(code >> 1);
    (x, y)
}

/// Calculates the Morton code for the neighbor to the East (+x).
/// Handles carry propagation correctly by decoding/encoding.
pub fn neighbor_east(code: u32) -> u32 {
    let (x, y) = morton_decode(code);
    morton_encode(x.wrapping_add(1), y)
}

/// Calculates the Morton code for the neighbor to the West (-x).
pub fn neighbor_west(code: u32) -> u32 {
    let (x, y) = morton_decode(code);
    morton_encode(x.wrapping_sub(1), y)
}

/// Calculates the Morton code for the neighbor to the South (+y).
/// Note: In WGPU/Bevy/Many game grids, +Y is UP, but in some it's DOWN.
/// Assuming +Y is UP (North) for World Space, but if this is array indexing, +Y might be down.
/// Consistently using +Y as "South" or "North" depends on convention.
/// Given standard grid iteration often goes 0..H, let's just expose directional adds.
pub fn neighbor_south(code: u32) -> u32 {
    let (x, y) = morton_decode(code);
    morton_encode(x, y.wrapping_add(1))
}

/// Calculates the Morton code for the neighbor to the North (-y).
pub fn neighbor_north(code: u32) -> u32 {
    let (x, y) = morton_decode(code);
    morton_encode(x, y.wrapping_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_bits() {
        assert_eq!(spread_bits_16(0b0001), 0b0001);
        assert_eq!(spread_bits_16(0b0010), 0b0100);
        assert_eq!(spread_bits_16(0b0011), 0b0101);
        assert_eq!(spread_bits_16(0xFFFF), 0x55555555);
    }

    #[test]
    fn test_compact_bits() {
        assert_eq!(compact_bits_32(0b0001), 0b0001);
        assert_eq!(compact_bits_32(0b0100), 0b0010);
        assert_eq!(compact_bits_32(0b0101), 0b0011);
        assert_eq!(compact_bits_32(0x55555555), 0xFFFF);
    }

    #[test]
    fn test_encode_decode() {
        let x = 123;
        let y = 456;
        let code = morton_encode(x, y);
        let (dx, dy) = morton_decode(code);
        assert_eq!(x, dx);
        assert_eq!(y, dy);
    }

    #[test]
    fn test_neighbors() {
        let x = 10;
        let y = 10;
        let center = morton_encode(x, y);
        
        let east = neighbor_east(center);
        assert_eq!(morton_decode(east), (x + 1, y));

        let west = neighbor_west(center);
        assert_eq!(morton_decode(west), (x - 1, y));
        
        let south = neighbor_south(center);
        assert_eq!(morton_decode(south), (x, y + 1));
        
        let north = neighbor_north(center);
        assert_eq!(morton_decode(north), (x, y - 1));
    }
    
    #[test]
    fn test_wrapping() {
        let x = 0xFFFF;
        let y = 10;
        let code = morton_encode(x, y);
        let east = neighbor_east(code);
        assert_eq!(morton_decode(east), (0, y)); // Wrapped
    }
}
