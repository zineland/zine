use std::{fs, path::Path};

use fluent::{bundle::FluentBundle, FluentArgs, FluentResource, FluentValue};
use intl_memoizer::concurrent::IntlLangMemoizer;

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
                    println!("Warning: `{file}` does not exist, please add your translation to this file.");
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

    pub(crate) fn format(&self, key: &str, number: Option<i64>) -> String {
        let pattern = self
            .bundle
            .get_message(key)
            .unwrap_or_else(|| panic!("Invalid fluent key: `{}`", key))
            .value()
            .expect("Missing Value.");

        let mut fluent_args = FluentArgs::new();
        if let Some(number) = number {
            fluent_args.set("number", FluentValue::from(number));
        }

        self.bundle
            .format_pattern(pattern, Some(fluent_args).as_ref(), &mut vec![])
            .into_owned()
    }
}
