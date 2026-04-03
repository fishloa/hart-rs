# HART Engineering Unit Codes

Compiled from publicly available HART device specifications. Unit codes are defined
in HCF_SPEC-183 (HART Common Tables), Table 2. The official spec is behind the
FieldComm Group paywall; these values are derived from published device specifications.

**Important**: Codes 170-219 are in the "expansion range" — their meaning depends on the
Device Variable Classification (see bottom of this document).

## Sources

- Siemens SITRANS LT500 HART Device Specification (A5E50170584A, Rev 1, Oct 2020)
- ABB TRIO-MASS Massmeter MC2 HART Protocol (D184B108U08)
- UE One Series 1XTX HART Command Details (1XTX_HARTCMD-01)
- Krohne IFC010 HART Command Specification (Rev 1.0, May 1997)
- jszumigaj/hart Go library (univrsl/units.go)
- FieldComm Group HART-IP Developer Kit flow device spec

---

## Fixed Unit Codes (0-169, 220+)

These codes have the same meaning regardless of Device Variable Classification.

### Pressure (Device Variable Classification 65, Table 2.65)

| Code | Unit |
|------|------|
| 1 | inches of water column (inWC) at 68 deg F |
| 2 | inches of mercury (inHg) at 0 deg C |
| 3 | feet of water (ftH2O) at 68 deg F |
| 4 | millimeters of water (mmH2O) at 68 deg F |
| 5 | millimeters of mercury (mmHg) at 0 deg C |
| 6 | pounds per square inch (psi) |
| 7 | bar |
| 8 | millibar (mbar) |
| 9 | grams per square centimeter (g/cm2) |
| 10 | kilograms per square centimeter (kg/cm2) |
| 11 | pascal (Pa) |
| 12 | kilopascal (kPa) |
| 13 | torr |
| 14 | atmospheres (atm) |
| 145 | inches of water at 60 deg F (inH2O@60F) |
| 237 | megapascal (MPa) |
| 238 | inches of water at 4 deg C (inH2O@4C) |
| 239 | millimeters of water at 4 deg C (mmH2O@4C) |

### Volumetric Flow (Device Variable Classification 66, Table 2.66)

| Code | Unit |
|------|------|
| 15 | cubic feet per minute |
| 16 | US gallons per minute |
| 17 | liters per minute |
| 18 | imperial gallons per minute |
| 19 | cubic meters per hour |
| 22 | US gallons per second |
| 23 | million US gallons per day |
| 24 | liters per second |
| 25 | million liters per day |
| 26 | cubic feet per second |
| 27 | cubic feet per day |
| 28 | cubic meters per second |
| 29 | cubic meters per day |
| 30 | imperial gallons per hour |
| 31 | imperial gallons per day |
| 130 | cubic feet per hour |
| 131 | cubic meters per minute |
| 132 | barrels (42 US gallons) per second |
| 133 | barrels (42 US gallons) per minute |
| 134 | barrels (42 US gallons) per hour |
| 135 | barrels (42 US gallons) per day |
| 136 | US gallons per hour |
| 137 | imperial gallons per second |
| 138 | liters per hour |
| 235 | US gallons per day |

### Temperature (Device Variable Classification 64, Table 2.64)

| Code | Unit |
|------|------|
| 32 | degrees Celsius |
| 33 | degrees Fahrenheit |
| 34 | degrees Rankine |
| 35 | Kelvin |

### Electrical

| Code | Unit |
|------|------|
| 36 | millivolts (mV) |
| 37 | volts (V) |
| 38 | ohms |
| 39 | milliamperes (mA) |

### Volume (Device Variable Classification 68, Table 2.68)

| Code | Unit |
|------|------|
| 40 | US gallons |
| 41 | liters |
| 42 | imperial gallons |
| 43 | cubic meters |
| 46 | barrels (42 US gallons) |
| 110 | bushel |
| 111 | cubic yards |
| 112 | cubic feet |
| 113 | cubic inches |
| 124 | liquid barrels (31.5 US gallons) |
| 236 | hectoliters |

### Length (Device Variable Classification 69, Table 2.69)

| Code | Unit |
|------|------|
| 44 | feet |
| 45 | meters |
| 47 | inches |
| 48 | centimeters |
| 49 | millimeters |

### Time (Device Variable Classification 70, Table 2.70)

| Code | Unit |
|------|------|
| 50 | minutes |
| 51 | seconds |
| 52 | hours |
| 53 | days |

### Mass (Device Variable Classification 71, Table 2.71)

| Code | Unit |
|------|------|
| 60 | grams (g) |
| 61 | kilograms (kg) |
| 62 | metric tons (t) |
| 63 | pounds (lb) |

### Mass Flow (Device Variable Classification 72, Table 2.67)

| Code | Unit |
|------|------|
| 70 | grams per second (g/s) |
| 71 | grams per minute (g/min) |
| 72 | grams per hour (g/h) |
| 73 | kilograms per second (kg/s) |
| 74 | kilograms per minute (kg/min) |
| 75 | kilograms per hour (kg/h) |
| 76 | kilograms per day (kg/d) |
| 77 | metric tons per minute (t/min) |
| 78 | metric tons per hour (t/h) |
| 79 | metric tons per day (t/d) |
| 80 | pounds per second (lb/s) |
| 81 | pounds per minute (lb/min) |
| 82 | pounds per hour (lb/h) |
| 83 | pounds per day (lb/d) |
| 84 | short tons per minute (ton/min) |
| 85 | short tons per hour (ton/h) |

### Density (Device Variable Classification 73, Table 2.72)

| Code | Unit |
|------|------|
| 91 | grams per cubic centimeter (g/cm3) |
| 92 | kilograms per cubic meter (kg/m3) |
| 93 | pounds per US gallon (lb/ugl) |
| 94 | pounds per cubic foot (lb/ft3) |
| 95 | grams per milliliter (g/ml) |
| 96 | kilograms per liter (kg/l) |
| 97 | grams per liter (g/l) |

### Percent / Dimensionless

| Code | Unit |
|------|------|
| 57 | percent |

### Special Codes

| Code | Meaning |
|------|---------|
| 250 | not used |
| 251 | none (dimensionless) |
| 252 | unknown |
| 253 | custom unit (device-specific) |

---

## Expansion Range Unit Codes (170-219)

Codes in this range have different meanings depending on the Device Variable Classification.
The device's classification (reported in Command 8 response) determines the interpretation.

### When Classification = Temperature (64)

*No expansion codes defined for temperature.*

### When Classification = Volumetric Flow (66)

| Code | Unit |
|------|------|
| 170 | beer barrel per second |
| 171 | beer barrel per minute |
| 172 | beer barrel per hour |
| 173 | beer barrel per day |

### When Classification = Volume (68)

| Code | Unit |
|------|------|
| 170 | beer barrel |

### When Classification = Time (70)

| Code | Unit |
|------|------|
| 170 | milliseconds |
| 171 | microseconds |
| 172 | nanoseconds |

### When Classification = Mass Flow (72)

| Code | Unit |
|------|------|
| 240 | programmable unit/s |
| 241 | programmable unit/min |
| 242 | programmable unit/h |
| 243 | programmable unit/d |

### When Classification = Mass (71)

| Code | Unit |
|------|------|
| 244 | programmable unit |

### When Classification = Volume (68, continued)

| Code | Unit |
|------|------|
| 249 | programmable unit |

### When Classification = Current (84)

| Code | Unit |
|------|------|
| 170 | nanoamperes |
| 171 | microamperes |

---

## Device Variable Classifications (Common Table 21)

| Code | Classification |
|------|---------------|
| 0 | Not classified |
| 64 | Temperature |
| 65 | Pressure |
| 66 | Volumetric flow |
| 67 | Velocity |
| 68 | Volume |
| 69 | Length |
| 70 | Time |
| 71 | Mass |
| 72 | Mass flow |
| 73 | Mass per volume (density) |
| 74 | Viscosity |
| 75 | Angular velocity |
| 76 | Area |
| 80 | Force |
| 81 | Power |
| 82 | Energy |
| 83 | Torque |
| 84 | Current |
| 85 | Voltage |
| 86 | Frequency |
| 87 | Analytical |

---

## Notes

- This compilation covers ~150 unique codes from publicly available device specifications.
- Still missing categories: viscosity, velocity, force, power, energy, torque,
  voltage, frequency, angular velocity, area, and analytical.
- The `Unknown(u8)` variant in the Rust enum handles any code not explicitly listed.
- If you have access to HCF_SPEC-183, the enum should be updated to include all codes.
- Unit codes are the same across all HART protocol versions (5, 6, 7).
- Codes 240-249 are reserved for manufacturer-defined / programmable units.
- Codes 254-255 are reserved by HCF.
