use std::collections::HashMap; 

pub enum Part {
    // String-derived
    Id(String),
    IdRef(String),
    Language(String),
    Name(String),
    NmToken(String),
    NormalizedString(String),
    String(String),
    Token(String),

    // Date-derived
    Date(String),
    Time(String),
    DateTime(String),
    Duration(String),

    // Numeric types
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    UnsignedByte(u8),
    UnsignedShort(u16),
    UnsignedInt(u32),
    UnsignedLong(u64),
    Decimal(f64),
    Integer(i64),
    NegativeInteger(u64),    // -1, -2, -3...
    PositiveInteger(u64),    // 1, 2, 3...
    NonNegativeInteger(u64), // 0, 1, 2, 3...
    NonPositiveInteger(u64), // 0, -1, -2, -3...

    // Misc.
    Boolean(bool),
    Base64Binary(String),
    HexBinary(String),
    AnyUri(String),

    // Attrs, Content.
    ComplexType(HashMap<String, Part>, HashMap<String, Part>),
}

impl Part {
    pub fn xsd_type(&self) -> String {
        match self {
            &Part::String(_) => "xsd:string",
            _ => "xsd:string",
        }.to_string()
    }
}

