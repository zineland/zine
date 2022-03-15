/// A serde module to serialize and deserialize [`time::Date`] type.
pub mod serde_date {
    use serde::{de, Serialize, Serializer};
    use time::{format_description, Date};

    pub fn serialize<S: Serializer>(date: &Date, serializer: S) -> Result<S::Ok, S::Error> {
        let format = format_description::parse("[year]-[month]-[day]").expect("Shouldn't happen");
        date.format(&format)
            .expect("Serialize date error")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Date, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_any(DateVisitor)
    }

    struct DateVisitor;

    impl<'de> de::Visitor<'de> for DateVisitor {
        type Value = Date;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a date value like YYYY-MM-dd")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let format =
                format_description::parse("[year]-[month]-[day]").expect("Shouldn't happen");
            Ok(Date::parse(v, &format)
                .unwrap_or_else(|_| panic!("The date value {} is invalid", &v)))
        }
    }
}
