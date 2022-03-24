use std::collections::HashMap;

use fluent::{bundle::FluentBundle, FluentArgs, FluentResource, FluentValue};
use intl_memoizer::concurrent::IntlLangMemoizer;
use serde_json::Value;

static FLUENT_EN: &str = include_str!("../locales/en.ftl");
static FLUENT_ZH_CN: &str = include_str!("../locales/zh-CN.ftl");

/// Fluent locale loader to localize text.
///
/// [`FluentLoader`] implements [`tera::Function`] trait,
/// so it can be register as a tera function.
pub struct FluentLoader {
    bundle: FluentBundle<FluentResource, IntlLangMemoizer>,
}

impl FluentLoader {
    pub fn new(locale: &str) -> Self {
        let mut bundle =
            FluentBundle::new_concurrent(vec![locale.parse().expect("Invalid locale string.")]);
        if let Some(resource) = match locale {
            "en" => FluentResource::try_new(FLUENT_EN.to_owned()).ok(),
            "zh-CN" => FluentResource::try_new(FLUENT_ZH_CN.to_owned()).ok(),
            _ => None,
        } {
            bundle.add_resource(resource).unwrap();
        }
        FluentLoader { bundle }
    }
}

fn json_to_fluent(json: &Value) -> FluentValue {
    match json {
        Value::Number(n) if n.is_u64() => FluentValue::from(n.as_u64().unwrap()),
        Value::Number(n) if n.is_i64() => FluentValue::from(n.as_i64().unwrap()),
        Value::Number(n) if n.is_f64() => FluentValue::from(n.as_f64().unwrap()),
        Value::String(s) => FluentValue::String(s.into()),
        _ => {
            println!("Invalid value to convert to fluent: {}", &json);
            FluentValue::None
        }
    }
}

impl tera::Function for FluentLoader {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let key = args
            .get("key")
            .and_then(Value::as_str)
            .expect("Missing `key` argument.");

        let pattern = self
            .bundle
            .get_message(key)
            .expect("Invalid fluent `key`")
            .value()
            .expect("Missing Value.");

        let mut fluent_args = FluentArgs::new();
        for (key, value) in args.iter().filter(|(key, _)| &**key != "key") {
            fluent_args.set(&**key, json_to_fluent(value));
        }

        Ok(Value::String(
            self.bundle
                .format_pattern(pattern, Some(fluent_args).as_ref(), &mut vec![])
                .into_owned(),
        ))
    }
}
