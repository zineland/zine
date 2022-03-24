use std::{collections::HashMap, fs, path::Path};

use fluent::{bundle::FluentBundle, FluentArgs, FluentResource, FluentValue};
use intl_memoizer::concurrent::IntlLangMemoizer;
use serde_json::Value;

static FLUENT_EN: &str = include_str!("../locales/en.ftl");
static FLUENT_ZH_CN: &str = include_str!("../locales/zh.ftl");

/// Fluent locale loader to localize text.
///
/// [`FluentLoader`] implements [`tera::Function`] trait,
/// so it can be register as a tera function.
pub struct FluentLoader {
    bundle: FluentBundle<FluentResource, IntlLangMemoizer>,
}

impl FluentLoader {
    pub fn new(source: &Path, mut locale: &str) -> Self {
        let resource = match locale {
            "en" => FluentResource::try_new(FLUENT_EN.to_owned()),
            "zh" => FluentResource::try_new(FLUENT_ZH_CN.to_owned()),
            _ => {
                // Not a buitlin locale, load the user translation resource.
                let file = format!("locales/{}.ftl", locale);
                let path = source.join(&file);
                if path.exists() {
                    let translation = fs::read_to_string(path)
                        .unwrap_or_else(|err| panic!("{file} read failed: {}", err));
                    FluentResource::try_new(translation)
                } else {
                    println!("`{file}` does not exist, please add your translation to this file.");
                    println!("fallback to default `en` locale.");

                    locale = "en";
                    FluentResource::try_new(FLUENT_EN.to_owned())
                }
            }
        }
        .expect("Load translation failed.");

        let lang_id = locale.parse().expect("Invalid locale string.");
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.add_resource(resource).unwrap();
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
            .unwrap_or_else(|| panic!("Invalid fluent key: `{}`", key))
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
