use std::hint;
use std::io;
use std::io::Write;
use std::ptr;
use std::simd::cmp::SimdPartialEq;

#[cfg(debug_assertions)]
pub const capacity: usize = 1 << 15;

#[cfg(not(debug_assertions))]
pub const capacity: usize = 1 << 13;

#[cfg(not(debug_assertions))]
const capacity_bits: u32 = 13;

#[cfg(not(debug_assertions))]
const index_multiplier: u64 = 0x72aff84272951c0d;

pub const newl: u8 = b'\n';
pub const semi: u8 = b';';

pub const u8x32_semi: u8x32 = u8x32::splat(semi);
pub const u8x32_newl: u8x32 = u8x32::splat(newl);

pub const u8x32_lanes: usize = 32;
pub type u8x32 = std::simd::Simd<u8, u8x32_lanes>;

pub type Temperature = i16;
pub type TemperatureCount = i64;

const short_u64_masks: [u64; 8] = [
    0x0000000000000000,
    0x00000000000000ff,
    0x000000000000ffff,
    0x0000000000ffffff,
    0x00000000ffffffff,
    0x000000ffffffffff,
    0x0000ffffffffffff,
    0x00ffffffffffffff,
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct StationKey(u64);

impl StationKey {
    #[inline(always)]
    fn index(self) -> usize {
        #[cfg(debug_assertions)]
        {
            self.0 as usize & (capacity - 1)
        }

        #[cfg(not(debug_assertions))]
        {
            (self.0.wrapping_mul(index_multiplier) >> (64 - capacity_bits)) as usize
        }
    }
}

struct StationSlot<'a> {
    #[cfg(debug_assertions)]
    key: StationKey,
    station: &'a [u8],
    aggregate: Aggregate,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Aggregate {
    pub min: Temperature,
    pub max: Temperature,
    pub sum: TemperatureCount,
    pub count: TemperatureCount,
}

impl Aggregate {
    pub fn new(temperature: Temperature) -> Self {
        Self {
            max: temperature,
            min: temperature,
            sum: temperature as TemperatureCount,
            count: 1,
        }
    }

    pub fn update(&mut self, temperature: Temperature) {
        self.sum += temperature as TemperatureCount;
        self.count += 1;
        if temperature > self.max {
            self.max = temperature
        }
        if temperature < self.min {
            self.min = temperature
        }
    }
}

impl Extend<Aggregate> for Aggregate {
    fn extend<T: IntoIterator<Item = Aggregate>>(&mut self, iter: T) {
        for item in iter {
            self.extend_one(item);
        }
    }

    fn extend_one(&mut self, item: Aggregate) {
        self.sum += item.sum;
        self.count += item.count;
        if item.max > self.max {
            self.max = item.max
        }
        if item.min < self.min {
            self.min = item.min
        }
    }
}

pub struct Metrics<'a> {
    table: Vec<Option<StationSlot<'a>>>,
    len: usize,
}

impl<'a> Metrics<'a> {
    pub fn new() -> Self {
        let mut table = Vec::with_capacity(capacity);
        table.resize_with(capacity, || None);

        Self { table, len: 0 }
    }

    #[cfg(not(debug_assertions))]
    pub fn compute(&mut self, slice: &'a [u8]) {
        unsafe { self.compute_fast(slice) }
    }

    #[cfg(debug_assertions)]
    pub fn compute(&mut self, slice: &'a [u8]) {
        self.compute_generic(slice)
    }

    #[cfg(not(debug_assertions))]
    unsafe fn compute_fast(&mut self, slice: &'a [u8]) {
        let mut cursor = 0;
        let ptr = slice.as_ptr();

        while cursor + 40 <= slice.len() {
            let line_ptr = unsafe { ptr.add(cursor) };
            let chunk =
                unsafe { u8x32::from_array(ptr::read_unaligned(line_ptr as *const [u8; 32])) };
            let semicolon_bitmask = chunk.simd_eq(u8x32_semi).to_bitmask();

            if semicolon_bitmask == 0 {
                self.compute_generic(unsafe { slice.get_unchecked(cursor..) });
                return;
            }

            let semicolon_cursor = semicolon_bitmask.trailing_zeros() as usize;
            let station = unsafe { slice.get_unchecked(cursor..cursor + semicolon_cursor) };
            let temperature_word =
                unsafe { ptr::read_unaligned(line_ptr.add(semicolon_cursor + 1) as *const u64) };
            let dot_pos = dot_pos(temperature_word);
            let temperature = parse_temperature_word(temperature_word, dot_pos);

            self.upsert(station, temperature);

            cursor += semicolon_cursor + (dot_pos >> 3) + 4;
        }

        if cursor < slice.len() {
            self.compute_generic(unsafe { slice.get_unchecked(cursor..) });
        }
    }

    fn compute_generic(&mut self, slice: &'a [u8]) {
        let mut cursor = 0;
        let mut line_start_cursor = 0;
        let mut maybe_semicolon_cursor = None;

        while cursor + u8x32_lanes <= slice.len() {
            let chunk = u8x32::from_slice(&slice[cursor..cursor + u8x32_lanes]);

            let semicolon_bitmask = chunk.simd_eq(u8x32_semi).to_bitmask();
            let newline_bitmask = chunk.simd_eq(u8x32_newl).to_bitmask();

            let mut bitmask = semicolon_bitmask | newline_bitmask;

            while bitmask != 0 {
                let relative_index = bitmask.trailing_zeros() as usize;
                let absolute_index = cursor + relative_index;

                if ((semicolon_bitmask >> relative_index) & 1) != 0 {
                    maybe_semicolon_cursor = Some(absolute_index);
                } else {
                    let semicolon_cursor = maybe_semicolon_cursor
                        .take()
                        .expect("newline must be before semicolon");

                    let station =
                        unsafe { slice.get_unchecked(line_start_cursor..semicolon_cursor) };
                    let temperature = unsafe {
                        parse_temperature_ptr(
                            slice.as_ptr().add(semicolon_cursor + 1),
                            absolute_index - semicolon_cursor - 1,
                        )
                    };

                    self.upsert(station, temperature);

                    line_start_cursor = absolute_index + 1;
                    maybe_semicolon_cursor = None;
                }

                bitmask &= bitmask - 1;
            }

            cursor += u8x32_lanes;
        }

        while cursor < slice.len() {
            match slice[cursor] {
                semi => maybe_semicolon_cursor = Some(cursor),
                newl => {
                    let semicolon_cursor = maybe_semicolon_cursor
                        .take()
                        .expect("newline must be before semicolon");

                    let station =
                        unsafe { slice.get_unchecked(line_start_cursor..semicolon_cursor) };
                    let temperature = unsafe {
                        parse_temperature_ptr(
                            slice.as_ptr().add(semicolon_cursor + 1),
                            cursor - semicolon_cursor - 1,
                        )
                    };

                    self.upsert(station, temperature);

                    line_start_cursor = cursor + 1;
                    maybe_semicolon_cursor = None;
                }
                _ => (),
            };

            cursor += 1;
        }
    }

    pub fn render(self, mut writer: impl Write) -> io::Result<()> {
        let mut stations: Vec<_> = self.table.iter().filter_map(|slot| slot.as_ref()).collect();
        stations.sort_unstable_by(|left, right| left.station.cmp(right.station));
        let mut stations = stations.into_iter().peekable();

        write!(&mut writer, "{{")?;

        while let Some(slot) = stations.next() {
            let aggregate = &slot.aggregate;

            let station = unsafe { str::from_utf8_unchecked(slot.station) };
            let min = aggregate.min as f64 / 10.0;
            let avg = (aggregate.sum as f64 / aggregate.count as f64).round() / 10.0;
            let max = aggregate.max as f64 / 10.0;

            write!(&mut writer, "{}={:.1}/{:.1}/{:.1}", station, min, avg, max)?;

            if stations.peek().is_some() {
                write!(&mut writer, ", ")?;
            }
        }

        writeln!(&mut writer, "}}")?;

        writer.flush()?;

        Ok(())
    }
}

trait Upsert<K, T> {
    fn upsert(&mut self, key: K, value: T);
}

impl<'a> Upsert<&'a [u8], Temperature> for Metrics<'a> {
    fn upsert(&mut self, key: &'a [u8], value: Temperature) {
        let station_key = station_key(key);

        #[cfg(not(debug_assertions))]
        {
            let index = station_key.index();

            match unsafe { self.table.get_unchecked_mut(index) } {
                Some(slot) => {
                    slot.aggregate.update(value);
                }
                empty => {
                    *empty = Some(StationSlot {
                        #[cfg(debug_assertions)]
                        key: station_key,
                        station: key,
                        aggregate: Aggregate::new(value),
                    });
                    self.len += 1;
                }
            }

            return;
        }

        #[cfg(debug_assertions)]
        {
            let mut index = station_key.index();

            loop {
                match unsafe { self.table.get_unchecked_mut(index) } {
                    Some(slot) => {
                        if slot.key == station_key {
                            debug_assert_eq!(slot.station, key, "station fingerprint collision");
                            slot.aggregate.update(value);
                            return;
                        }
                    }
                    empty => {
                        *empty = Some(StationSlot {
                            #[cfg(debug_assertions)]
                            key: station_key,
                            station: key,
                            aggregate: Aggregate::new(value),
                        });
                        self.len += 1;
                        return;
                    }
                }

                index = (index + 1) & (capacity - 1);
            }
        }
    }
}

impl<'a> Upsert<&'a [u8], Aggregate> for Metrics<'a> {
    fn upsert(&mut self, key: &'a [u8], value: Aggregate) {
        let station_key = station_key(key);

        #[cfg(not(debug_assertions))]
        {
            let index = station_key.index();

            match unsafe { self.table.get_unchecked_mut(index) } {
                Some(slot) => {
                    slot.aggregate.extend_one(value);
                }
                empty => {
                    *empty = Some(StationSlot {
                        #[cfg(debug_assertions)]
                        key: station_key,
                        station: key,
                        aggregate: value,
                    });
                    self.len += 1;
                }
            }

            return;
        }

        #[cfg(debug_assertions)]
        {
            let mut index = station_key.index();

            loop {
                match unsafe { self.table.get_unchecked_mut(index) } {
                    Some(slot) => {
                        if slot.key == station_key {
                            debug_assert_eq!(slot.station, key, "station fingerprint collision");
                            slot.aggregate.extend_one(value);
                            return;
                        }
                    }
                    empty => {
                        *empty = Some(StationSlot {
                            #[cfg(debug_assertions)]
                            key: station_key,
                            station: key,
                            aggregate: value,
                        });
                        self.len += 1;
                        return;
                    }
                }

                index = (index + 1) & (capacity - 1);
            }
        }
    }
}

impl<'a> Extend<Metrics<'a>> for Metrics<'a> {
    fn extend<T: IntoIterator<Item = Metrics<'a>>>(&mut self, iter: T) {
        for item in iter {
            self.extend_one(item);
        }
    }

    fn extend_one(&mut self, item: Metrics<'a>) {
        for slot in item.table.into_iter().flatten() {
            self.upsert(slot.station, slot.aggregate);
        }
    }
}

#[inline(always)]
fn station_key(slice: &[u8]) -> StationKey {
    #[cfg(not(debug_assertions))]
    {
        unsafe { station_key_ptr(slice.as_ptr(), slice.len()) }
    }

    #[cfg(debug_assertions)]
    {
        let len = slice.len();

        unsafe { hint::assert_unchecked(len <= u16::MAX as usize) };

        let head = if len >= 8 {
            unsafe { ptr::read_unaligned(slice.as_ptr() as *const u64) }
        } else {
            read_short_u64(slice.as_ptr(), len)
        };

        let extra: u64 = if len >= 8 {
            (unsafe { ptr::read_unaligned(slice.as_ptr().add(len - 8) as *const u64) })
                ^ read_u64_middle(slice).rotate_left(17)
        } else {
            head
        };

        let mut hash = head ^ extra.rotate_left(32) ^ ((len as u64) << 48) ^ len as u64;

        hash ^= hash >> 32;
        hash ^= hash >> 16;

        StationKey(hash)
    }
}

#[inline(always)]
#[cfg(not(debug_assertions))]
unsafe fn station_key_ptr(ptr: *const u8, len: usize) -> StationKey {
    unsafe { hint::assert_unchecked(len <= u16::MAX as usize) };

    let head = if len >= 8 {
        unsafe { ptr::read_unaligned(ptr as *const u64) }
    } else {
        read_short_u64(ptr, len)
    };

    StationKey(head ^ ((len as u64) << 48) ^ len as u64)
}

#[inline(always)]
#[cfg(debug_assertions)]
fn read_u64_middle(slice: &[u8]) -> u64 {
    if slice.len() > 16 {
        let offset = slice.len() / 2;

        unsafe { ptr::read_unaligned(slice.as_ptr().add(offset) as *const u64) }
    } else {
        0
    }
}

#[inline(always)]
fn read_short_u64(ptr: *const u8, len: usize) -> u64 {
    unsafe {
        if len >= 3 {
            return ptr::read_unaligned(ptr as *const u64) & *short_u64_masks.get_unchecked(len);
        }

        match len {
            0 => 0,
            1 => *ptr as u64,
            2 => ptr::read_unaligned(ptr as *const u16) as u64,
            _ => ptr::read_unaligned(ptr as *const u64),
        }
    }
}

#[inline(always)]
#[cfg(test)]
fn parse_temperature<'a>(slice: &'a [u8]) -> Temperature {
    unsafe { parse_temperature_ptr(slice.as_ptr(), slice.len()) }
}

#[inline(always)]
#[cfg(not(debug_assertions))]
fn dot_pos(word: u64) -> usize {
    ((!word & 0x10101000).trailing_zeros()) as usize
}

#[inline(always)]
#[cfg(not(debug_assertions))]
fn parse_temperature_word(word: u64, dot_pos: usize) -> Temperature {
    const MAGIC_MULTIPLIER: u64 = 100 * 0x1000000 + 10 * 0x10000 + 1;

    let signed = (((!word << 59) as i64) >> 63) as u64;
    let minus_filter = !(signed & 0xff);
    let digits = ((word & minus_filter) << (dot_pos ^ 0b11100)) & 0x0f000f0f00;
    let abs = ((digits.wrapping_mul(MAGIC_MULTIPLIER) >> 32) & 0x3ff) as u64;

    ((abs ^ signed).wrapping_sub(signed)) as i16
}

#[inline(always)]
unsafe fn parse_temperature_ptr(ptr: *const u8, len: usize) -> Temperature {
    unsafe { hint::assert_unchecked(len >= 3) };

    let negative = unsafe { (*ptr == b'-') as usize };

    let frac = unsafe { (*ptr.add(len - 1) - b'0') as Temperature };
    let ones = unsafe { (*ptr.add(len - 3) - b'0') as Temperature };
    let tens_index = if len >= 4 { len - 4 } else { 0 };
    let has_tens = (len >= 4 + negative) as Temperature;
    let tens = unsafe { has_tens * (*ptr.add(tens_index)).wrapping_sub(b'0') as Temperature };
    let value = tens * 100 + ones * 10 + frac;

    (1 - 2 * negative as Temperature) * value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    fn measure(filename: &str) {
        let input_path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("1brc/src/test/resources/samples")
            .join(filename);

        let assert_path = input_path.with_extension("out");

        let input = fs::read(&input_path).unwrap();
        let expected = fs::read(&assert_path).unwrap();

        let mut metrics = Metrics::new();
        metrics.compute(input.as_slice());

        let mut result = Vec::new();
        metrics.render(&mut result).unwrap();

        assert_eq!(String::from_utf8(result), String::from_utf8(expected))
    }

    #[test]
    fn measurements_1() {
        measure("measurements-1.txt");
    }

    #[test]
    fn measurements_2() {
        measure("measurements-2.txt");
    }

    #[test]
    fn measurements_3() {
        measure("measurements-3.txt");
    }

    #[test]
    fn measurements_10() {
        measure("measurements-10.txt");
    }

    #[test]
    fn measurements_20() {
        measure("measurements-20.txt");
    }

    #[test]
    fn measurements_10000_unique_keys() {
        measure("measurements-10000-unique-keys.txt");
    }

    #[test]
    fn measurements_boundaries() {
        measure("measurements-boundaries.txt");
    }

    #[test]
    fn measurements_complex_utf8() {
        measure("measurements-complex-utf8.txt");
    }

    #[test]
    fn measurements_dot() {
        measure("measurements-dot.txt");
    }

    #[test]
    fn measurements_rounding() {
        measure("measurements-rounding.txt");
    }

    #[test]
    fn measurements_short() {
        measure("measurements-short.txt");
    }

    #[test]
    fn measurements_shortest() {
        measure("measurements-shortest.txt");
    }

    #[test]
    fn parse_temperature_range() {
        assert_eq!(parse_temperature(b"0.0"), 0);

        assert_eq!(parse_temperature(b"-9.0"), -90);
        assert_eq!(parse_temperature(b"-9.5"), -95);
        assert_eq!(parse_temperature(b"-9.9"), -99);

        assert_eq!(parse_temperature(b"9.5"), 95);
        assert_eq!(parse_temperature(b"9.9"), 99);
        assert_eq!(parse_temperature(b"9.0"), 90);

        assert_eq!(parse_temperature(b"-99.0"), -990);
        assert_eq!(parse_temperature(b"-99.5"), -995);
        assert_eq!(parse_temperature(b"-99.9"), -999);

        assert_eq!(parse_temperature(b"99.0"), 990);
        assert_eq!(parse_temperature(b"99.5"), 995);
        assert_eq!(parse_temperature(b"99.9"), 999);
    }
}
