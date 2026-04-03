/// HART engineering unit codes from HCF_SPEC-183 (HART Common Tables), Table 2.
/// Expansion range codes (170-219) have classification-dependent meanings and
/// are represented as `ExpansionRange(u8)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCode {
    // --- Pressure (codes 1-14, 145, 237-239) ---
    /// inches of water column (inWC) at 68 deg F
    InchesWaterColumn68F,
    /// inches of mercury (inHg) at 0 deg C
    InchesOfMercury,
    /// feet of water (ftH2O) at 68 deg F
    FeetOfWater68F,
    /// millimeters of water (mmH2O) at 68 deg F
    MillimetersOfWater68F,
    /// millimeters of mercury (mmHg) at 0 deg C
    MillimetersOfMercury,
    /// pounds per square inch (psi)
    Psi,
    /// bar
    Bar,
    /// millibar (mbar)
    Millibar,
    /// grams per square centimeter (g/cm2)
    GramsPerSquareCentimeter,
    /// kilograms per square centimeter (kg/cm2)
    KilogramsPerSquareCentimeter,
    /// pascal (Pa)
    Pascals,
    /// kilopascal (kPa)
    KiloPascals,
    /// torr
    Torr,
    /// atmospheres (atm)
    Atmospheres,
    /// inches of water at 60 deg F
    InchesWaterColumn60F,
    /// megapascal (MPa)
    MegaPascals,
    /// inches of water at 4 deg C
    InchesWaterColumn4C,
    /// millimeters of water at 4 deg C
    MillimetersOfWater4C,

    // --- Volumetric Flow (codes 15-31, 130-138, 235) ---
    /// cubic feet per minute
    CubicFeetPerMinute,
    /// US gallons per minute
    UsGallonsPerMinute,
    /// liters per minute
    LitersPerMinute,
    /// imperial gallons per minute
    ImperialGallonsPerMinute,
    /// cubic meters per hour
    CubicMetersPerHour,
    /// US gallons per second
    UsGallonsPerSecond,
    /// million US gallons per day
    MillionUsGallonsPerDay,
    /// liters per second
    LitersPerSecond,
    /// million liters per day
    MillionLitersPerDay,
    /// cubic feet per second
    CubicFeetPerSecond,
    /// cubic feet per day
    CubicFeetPerDay,
    /// cubic meters per second
    CubicMetersPerSecond,
    /// cubic meters per day
    CubicMetersPerDay,
    /// imperial gallons per hour
    ImperialGallonsPerHour,
    /// imperial gallons per day
    ImperialGallonsPerDay,
    /// cubic feet per hour
    CubicFeetPerHour,
    /// cubic meters per minute
    CubicMetersPerMinute,
    /// barrels (42 US gallons) per second
    BarrelsPerSecond,
    /// barrels (42 US gallons) per minute
    BarrelsPerMinute,
    /// barrels (42 US gallons) per hour
    BarrelsPerHour,
    /// barrels (42 US gallons) per day
    BarrelsPerDay,
    /// US gallons per hour
    UsGallonsPerHour,
    /// imperial gallons per second
    ImperialGallonsPerSecond,
    /// liters per hour
    LitersPerHour,
    /// US gallons per day
    UsGallonsPerDay,

    // --- Temperature (codes 32-35) ---
    /// degrees Celsius
    DegreesCelsius,
    /// degrees Fahrenheit
    DegreesFahrenheit,
    /// degrees Rankine
    DegreesRankine,
    /// Kelvin
    Kelvin,

    // --- Electrical (codes 36-39) ---
    /// millivolts (mV)
    Millivolts,
    /// volts (V)
    Volts,
    /// ohms
    Ohms,
    /// milliamperes (mA)
    Milliamperes,

    // --- Volume (codes 40-43, 46, 110-113, 124, 236) ---
    /// US gallons
    UsGallons,
    /// liters
    Liters,
    /// imperial gallons
    ImperialGallons,
    /// cubic meters
    CubicMeters,
    /// barrels (42 US gallons)
    Barrels,
    /// bushel
    Bushel,
    /// cubic yards
    CubicYards,
    /// cubic feet
    CubicFeet,
    /// cubic inches
    CubicInches,
    /// liquid barrels (31.5 US gallons)
    LiquidBarrels,
    /// hectoliters
    Hectoliters,

    // --- Length (codes 44-45, 47-49) ---
    /// feet
    Feet,
    /// meters
    Meters,
    /// inches
    Inches,
    /// centimeters
    Centimeters,
    /// millimeters
    Millimeters,

    // --- Time (codes 50-53) ---
    /// minutes
    Minutes,
    /// seconds
    Seconds,
    /// hours
    Hours,
    /// days
    Days,

    // --- Percent / Dimensionless ---
    /// percent
    Percent,

    // --- Mass (codes 60-63) ---
    /// grams (g)
    Grams,
    /// kilograms (kg)
    Kilograms,
    /// metric tons (t)
    MetricTons,
    /// pounds (lb)
    Pounds,

    // --- Mass Flow (codes 70-85) ---
    /// grams per second (g/s)
    GramsPerSecond,
    /// grams per minute (g/min)
    GramsPerMinute,
    /// grams per hour (g/h)
    GramsPerHour,
    /// kilograms per second (kg/s)
    KilogramsPerSecond,
    /// kilograms per minute (kg/min)
    KilogramsPerMinute,
    /// kilograms per hour (kg/h)
    KilogramsPerHour,
    /// kilograms per day (kg/d)
    KilogramsPerDay,
    /// metric tons per minute (t/min)
    MetricTonsPerMinute,
    /// metric tons per hour (t/h)
    MetricTonsPerHour,
    /// metric tons per day (t/d)
    MetricTonsPerDay,
    /// pounds per second (lb/s)
    PoundsPerSecond,
    /// pounds per minute (lb/min)
    PoundsPerMinute,
    /// pounds per hour (lb/h)
    PoundsPerHour,
    /// pounds per day (lb/d)
    PoundsPerDay,
    /// short tons per minute (ton/min)
    ShortTonsPerMinute,
    /// short tons per hour (ton/h)
    ShortTonsPerHour,

    // --- Density (codes 91-97) ---
    /// grams per cubic centimeter (g/cm3)
    GramsPerCubicCentimeter,
    /// kilograms per cubic meter (kg/m3)
    KilogramsPerCubicMeter,
    /// pounds per US gallon (lb/ugl)
    PoundsPerUsGallon,
    /// pounds per cubic foot (lb/ft3)
    PoundsPerCubicFoot,
    /// grams per milliliter (g/ml)
    GramsPerMilliliter,
    /// kilograms per liter (kg/l)
    KilogramsPerLiter,
    /// grams per liter (g/l)
    GramsPerLiter,

    // --- Expansion range (codes 170-219, classification-dependent) ---
    /// Expansion range code (170-219), meaning depends on Device Variable Classification
    ExpansionRange(u8),

    // --- Programmable/manufacturer units (codes 240-249) ---
    /// programmable unit/s (240)
    ProgrammableUnitPerSecond,
    /// programmable unit/min (241)
    ProgrammableUnitPerMinute,
    /// programmable unit/h (242)
    ProgrammableUnitPerHour,
    /// programmable unit/d (243)
    ProgrammableUnitPerDay,
    /// programmable unit (244)
    ProgrammableUnit244,
    /// programmable unit (249)
    ProgrammableUnit249,

    // --- Special codes ---
    /// not used (250)
    NotUsed,
    /// none / dimensionless (251)
    None,
    /// unknown (252)
    UnknownUnit,
    /// custom unit, device-specific (253)
    Custom,

    /// Any code not explicitly listed above
    Unknown(u8),
}

impl UnitCode {
    pub fn from_u8(code: u8) -> Self {
        match code {
            // Pressure
            1 => UnitCode::InchesWaterColumn68F,
            2 => UnitCode::InchesOfMercury,
            3 => UnitCode::FeetOfWater68F,
            4 => UnitCode::MillimetersOfWater68F,
            5 => UnitCode::MillimetersOfMercury,
            6 => UnitCode::Psi,
            7 => UnitCode::Bar,
            8 => UnitCode::Millibar,
            9 => UnitCode::GramsPerSquareCentimeter,
            10 => UnitCode::KilogramsPerSquareCentimeter,
            11 => UnitCode::Pascals,
            12 => UnitCode::KiloPascals,
            13 => UnitCode::Torr,
            14 => UnitCode::Atmospheres,
            // Volumetric Flow
            15 => UnitCode::CubicFeetPerMinute,
            16 => UnitCode::UsGallonsPerMinute,
            17 => UnitCode::LitersPerMinute,
            18 => UnitCode::ImperialGallonsPerMinute,
            19 => UnitCode::CubicMetersPerHour,
            22 => UnitCode::UsGallonsPerSecond,
            23 => UnitCode::MillionUsGallonsPerDay,
            24 => UnitCode::LitersPerSecond,
            25 => UnitCode::MillionLitersPerDay,
            26 => UnitCode::CubicFeetPerSecond,
            27 => UnitCode::CubicFeetPerDay,
            28 => UnitCode::CubicMetersPerSecond,
            29 => UnitCode::CubicMetersPerDay,
            30 => UnitCode::ImperialGallonsPerHour,
            31 => UnitCode::ImperialGallonsPerDay,
            // Temperature
            32 => UnitCode::DegreesCelsius,
            33 => UnitCode::DegreesFahrenheit,
            34 => UnitCode::DegreesRankine,
            35 => UnitCode::Kelvin,
            // Electrical
            36 => UnitCode::Millivolts,
            37 => UnitCode::Volts,
            38 => UnitCode::Ohms,
            39 => UnitCode::Milliamperes,
            // Volume
            40 => UnitCode::UsGallons,
            41 => UnitCode::Liters,
            42 => UnitCode::ImperialGallons,
            43 => UnitCode::CubicMeters,
            // Length
            44 => UnitCode::Feet,
            45 => UnitCode::Meters,
            // Volume (continued)
            46 => UnitCode::Barrels,
            // Length (continued)
            47 => UnitCode::Inches,
            48 => UnitCode::Centimeters,
            49 => UnitCode::Millimeters,
            // Time
            50 => UnitCode::Minutes,
            51 => UnitCode::Seconds,
            52 => UnitCode::Hours,
            53 => UnitCode::Days,
            // Percent
            57 => UnitCode::Percent,
            // Mass
            60 => UnitCode::Grams,
            61 => UnitCode::Kilograms,
            62 => UnitCode::MetricTons,
            63 => UnitCode::Pounds,
            // Mass Flow
            70 => UnitCode::GramsPerSecond,
            71 => UnitCode::GramsPerMinute,
            72 => UnitCode::GramsPerHour,
            73 => UnitCode::KilogramsPerSecond,
            74 => UnitCode::KilogramsPerMinute,
            75 => UnitCode::KilogramsPerHour,
            76 => UnitCode::KilogramsPerDay,
            77 => UnitCode::MetricTonsPerMinute,
            78 => UnitCode::MetricTonsPerHour,
            79 => UnitCode::MetricTonsPerDay,
            80 => UnitCode::PoundsPerSecond,
            81 => UnitCode::PoundsPerMinute,
            82 => UnitCode::PoundsPerHour,
            83 => UnitCode::PoundsPerDay,
            84 => UnitCode::ShortTonsPerMinute,
            85 => UnitCode::ShortTonsPerHour,
            // Density
            91 => UnitCode::GramsPerCubicCentimeter,
            92 => UnitCode::KilogramsPerCubicMeter,
            93 => UnitCode::PoundsPerUsGallon,
            94 => UnitCode::PoundsPerCubicFoot,
            95 => UnitCode::GramsPerMilliliter,
            96 => UnitCode::KilogramsPerLiter,
            97 => UnitCode::GramsPerLiter,
            // Volume (continued)
            110 => UnitCode::Bushel,
            111 => UnitCode::CubicYards,
            112 => UnitCode::CubicFeet,
            113 => UnitCode::CubicInches,
            // Volume Flow (continued)
            124 => UnitCode::LiquidBarrels,
            130 => UnitCode::CubicFeetPerHour,
            131 => UnitCode::CubicMetersPerMinute,
            132 => UnitCode::BarrelsPerSecond,
            133 => UnitCode::BarrelsPerMinute,
            134 => UnitCode::BarrelsPerHour,
            135 => UnitCode::BarrelsPerDay,
            136 => UnitCode::UsGallonsPerHour,
            137 => UnitCode::ImperialGallonsPerSecond,
            138 => UnitCode::LitersPerHour,
            // Pressure (continued)
            145 => UnitCode::InchesWaterColumn60F,
            // Expansion range (170-219)
            170..=219 => UnitCode::ExpansionRange(code),
            // Programmable units (240-249)
            240 => UnitCode::ProgrammableUnitPerSecond,
            241 => UnitCode::ProgrammableUnitPerMinute,
            242 => UnitCode::ProgrammableUnitPerHour,
            243 => UnitCode::ProgrammableUnitPerDay,
            244 => UnitCode::ProgrammableUnit244,
            249 => UnitCode::ProgrammableUnit249,
            // Special codes
            250 => UnitCode::NotUsed,
            251 => UnitCode::None,
            252 => UnitCode::UnknownUnit,
            253 => UnitCode::Custom,
            // Volume Flow (continued)
            235 => UnitCode::UsGallonsPerDay,
            // Volume (continued)
            236 => UnitCode::Hectoliters,
            // Pressure (continued)
            237 => UnitCode::MegaPascals,
            238 => UnitCode::InchesWaterColumn4C,
            239 => UnitCode::MillimetersOfWater4C,
            // Fallback
            _ => UnitCode::Unknown(code),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            // Pressure
            UnitCode::InchesWaterColumn68F => 1,
            UnitCode::InchesOfMercury => 2,
            UnitCode::FeetOfWater68F => 3,
            UnitCode::MillimetersOfWater68F => 4,
            UnitCode::MillimetersOfMercury => 5,
            UnitCode::Psi => 6,
            UnitCode::Bar => 7,
            UnitCode::Millibar => 8,
            UnitCode::GramsPerSquareCentimeter => 9,
            UnitCode::KilogramsPerSquareCentimeter => 10,
            UnitCode::Pascals => 11,
            UnitCode::KiloPascals => 12,
            UnitCode::Torr => 13,
            UnitCode::Atmospheres => 14,
            // Volumetric Flow
            UnitCode::CubicFeetPerMinute => 15,
            UnitCode::UsGallonsPerMinute => 16,
            UnitCode::LitersPerMinute => 17,
            UnitCode::ImperialGallonsPerMinute => 18,
            UnitCode::CubicMetersPerHour => 19,
            UnitCode::UsGallonsPerSecond => 22,
            UnitCode::MillionUsGallonsPerDay => 23,
            UnitCode::LitersPerSecond => 24,
            UnitCode::MillionLitersPerDay => 25,
            UnitCode::CubicFeetPerSecond => 26,
            UnitCode::CubicFeetPerDay => 27,
            UnitCode::CubicMetersPerSecond => 28,
            UnitCode::CubicMetersPerDay => 29,
            UnitCode::ImperialGallonsPerHour => 30,
            UnitCode::ImperialGallonsPerDay => 31,
            // Temperature
            UnitCode::DegreesCelsius => 32,
            UnitCode::DegreesFahrenheit => 33,
            UnitCode::DegreesRankine => 34,
            UnitCode::Kelvin => 35,
            // Electrical
            UnitCode::Millivolts => 36,
            UnitCode::Volts => 37,
            UnitCode::Ohms => 38,
            UnitCode::Milliamperes => 39,
            // Volume
            UnitCode::UsGallons => 40,
            UnitCode::Liters => 41,
            UnitCode::ImperialGallons => 42,
            UnitCode::CubicMeters => 43,
            // Length
            UnitCode::Feet => 44,
            UnitCode::Meters => 45,
            // Volume (continued)
            UnitCode::Barrels => 46,
            // Length (continued)
            UnitCode::Inches => 47,
            UnitCode::Centimeters => 48,
            UnitCode::Millimeters => 49,
            // Time
            UnitCode::Minutes => 50,
            UnitCode::Seconds => 51,
            UnitCode::Hours => 52,
            UnitCode::Days => 53,
            // Percent
            UnitCode::Percent => 57,
            // Mass
            UnitCode::Grams => 60,
            UnitCode::Kilograms => 61,
            UnitCode::MetricTons => 62,
            UnitCode::Pounds => 63,
            // Mass Flow
            UnitCode::GramsPerSecond => 70,
            UnitCode::GramsPerMinute => 71,
            UnitCode::GramsPerHour => 72,
            UnitCode::KilogramsPerSecond => 73,
            UnitCode::KilogramsPerMinute => 74,
            UnitCode::KilogramsPerHour => 75,
            UnitCode::KilogramsPerDay => 76,
            UnitCode::MetricTonsPerMinute => 77,
            UnitCode::MetricTonsPerHour => 78,
            UnitCode::MetricTonsPerDay => 79,
            UnitCode::PoundsPerSecond => 80,
            UnitCode::PoundsPerMinute => 81,
            UnitCode::PoundsPerHour => 82,
            UnitCode::PoundsPerDay => 83,
            UnitCode::ShortTonsPerMinute => 84,
            UnitCode::ShortTonsPerHour => 85,
            // Density
            UnitCode::GramsPerCubicCentimeter => 91,
            UnitCode::KilogramsPerCubicMeter => 92,
            UnitCode::PoundsPerUsGallon => 93,
            UnitCode::PoundsPerCubicFoot => 94,
            UnitCode::GramsPerMilliliter => 95,
            UnitCode::KilogramsPerLiter => 96,
            UnitCode::GramsPerLiter => 97,
            // Volume (continued)
            UnitCode::Bushel => 110,
            UnitCode::CubicYards => 111,
            UnitCode::CubicFeet => 112,
            UnitCode::CubicInches => 113,
            UnitCode::LiquidBarrels => 124,
            // Volumetric Flow (continued)
            UnitCode::CubicFeetPerHour => 130,
            UnitCode::CubicMetersPerMinute => 131,
            UnitCode::BarrelsPerSecond => 132,
            UnitCode::BarrelsPerMinute => 133,
            UnitCode::BarrelsPerHour => 134,
            UnitCode::BarrelsPerDay => 135,
            UnitCode::UsGallonsPerHour => 136,
            UnitCode::ImperialGallonsPerSecond => 137,
            UnitCode::LitersPerHour => 138,
            // Pressure (continued)
            UnitCode::InchesWaterColumn60F => 145,
            // Expansion range
            UnitCode::ExpansionRange(c) => *c,
            // Programmable units
            UnitCode::ProgrammableUnitPerSecond => 240,
            UnitCode::ProgrammableUnitPerMinute => 241,
            UnitCode::ProgrammableUnitPerHour => 242,
            UnitCode::ProgrammableUnitPerDay => 243,
            UnitCode::ProgrammableUnit244 => 244,
            UnitCode::ProgrammableUnit249 => 249,
            // Special codes
            UnitCode::NotUsed => 250,
            UnitCode::None => 251,
            UnitCode::UnknownUnit => 252,
            UnitCode::Custom => 253,
            // Volumetric Flow (continued)
            UnitCode::UsGallonsPerDay => 235,
            // Volume (continued)
            UnitCode::Hectoliters => 236,
            // Pressure (continued)
            UnitCode::MegaPascals => 237,
            UnitCode::InchesWaterColumn4C => 238,
            UnitCode::MillimetersOfWater4C => 239,
            // Fallback
            UnitCode::Unknown(c) => *c,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_codes_roundtrip() {
        let cases: &[(u8, UnitCode)] = &[
            (1, UnitCode::InchesWaterColumn68F),
            (6, UnitCode::Psi),
            (7, UnitCode::Bar),
            (8, UnitCode::Millibar),
            (11, UnitCode::Pascals),
            (12, UnitCode::KiloPascals),
            (32, UnitCode::DegreesCelsius),
            (33, UnitCode::DegreesFahrenheit),
            (35, UnitCode::Kelvin),
            (39, UnitCode::Milliamperes),
            (44, UnitCode::Feet),
            (45, UnitCode::Meters),
            (49, UnitCode::Millimeters),
            (57, UnitCode::Percent),
        ];
        for &(code, ref expected) in cases {
            let decoded = UnitCode::from_u8(code);
            assert_eq!(&decoded, expected, "from_u8({}) mismatch", code);
            assert_eq!(
                decoded.as_u8(),
                code,
                "as_u8() roundtrip failed for code {}",
                code
            );
        }
    }

    #[test]
    fn test_unknown_code_preserved() {
        // Code 20 is not defined, should return Unknown
        let result = UnitCode::from_u8(20);
        assert_eq!(result, UnitCode::Unknown(20));
        assert_eq!(result.as_u8(), 20);

        // Code 100 is not defined
        let result = UnitCode::from_u8(100);
        assert_eq!(result, UnitCode::Unknown(100));
        assert_eq!(result.as_u8(), 100);
    }

    #[test]
    fn test_special_codes() {
        assert_eq!(UnitCode::from_u8(250), UnitCode::NotUsed);
        assert_eq!(UnitCode::NotUsed.as_u8(), 250);

        assert_eq!(UnitCode::from_u8(251), UnitCode::None);
        assert_eq!(UnitCode::None.as_u8(), 251);

        assert_eq!(UnitCode::from_u8(252), UnitCode::UnknownUnit);
        assert_eq!(UnitCode::UnknownUnit.as_u8(), 252);

        assert_eq!(UnitCode::from_u8(253), UnitCode::Custom);
        assert_eq!(UnitCode::Custom.as_u8(), 253);
    }

    #[test]
    fn test_expansion_range() {
        for code in 170u8..=219u8 {
            let unit = UnitCode::from_u8(code);
            assert_eq!(unit, UnitCode::ExpansionRange(code));
            assert_eq!(unit.as_u8(), code);
        }
    }

    #[test]
    fn test_all_known_codes_survive_roundtrip() {
        // All explicitly known codes (not Unknown) should survive as_u8 -> from_u8 -> as_u8
        let known_codes: &[u8] = &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
            49, 50, 51, 52, 53, 57, 60, 61, 62, 63, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81,
            82, 83, 84, 85, 91, 92, 93, 94, 95, 96, 97, 110, 111, 112, 113, 124, 130, 131, 132,
            133, 134, 135, 136, 137, 138, 145, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244,
            249, 250, 251, 252, 253,
        ];
        for &code in known_codes {
            let unit = UnitCode::from_u8(code);
            // None of these should be Unknown
            assert!(
                !matches!(unit, UnitCode::Unknown(_)),
                "code {} mapped to Unknown unexpectedly",
                code
            );
            // Roundtrip
            assert_eq!(unit.as_u8(), code, "roundtrip failed for code {}", code);
        }
    }

    #[test]
    fn test_all_u8_values_roundtrip() {
        // Every u8 value should survive from_u8 -> as_u8 -> from_u8
        for code in 0u8..=255u8 {
            let unit = UnitCode::from_u8(code);
            let back = unit.as_u8();
            assert_eq!(back, code, "as_u8 roundtrip failed for code {}", code);
            let unit2 = UnitCode::from_u8(back);
            assert_eq!(
                unit2.as_u8(),
                code,
                "double roundtrip failed for code {}",
                code
            );
        }
    }

    #[test]
    fn test_code_zero_is_unknown() {
        // Code 0 is not defined in the HART unit code table
        let unit = UnitCode::from_u8(0);
        assert_eq!(unit, UnitCode::Unknown(0));
        assert_eq!(unit.as_u8(), 0);
    }

    #[test]
    fn test_code_254_is_unknown() {
        let unit = UnitCode::from_u8(254);
        assert_eq!(unit, UnitCode::Unknown(254));
    }

    #[test]
    fn test_code_255_is_unknown() {
        let unit = UnitCode::from_u8(255);
        assert_eq!(unit, UnitCode::Unknown(255));
    }

    #[test]
    fn test_programmable_units_roundtrip() {
        let cases = [
            (240, UnitCode::ProgrammableUnitPerSecond),
            (241, UnitCode::ProgrammableUnitPerMinute),
            (242, UnitCode::ProgrammableUnitPerHour),
            (243, UnitCode::ProgrammableUnitPerDay),
            (244, UnitCode::ProgrammableUnit244),
            (249, UnitCode::ProgrammableUnit249),
        ];
        for (code, expected) in cases {
            assert_eq!(UnitCode::from_u8(code), expected);
            assert_eq!(expected.as_u8(), code);
        }
    }
}
