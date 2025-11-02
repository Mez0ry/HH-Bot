use core::fmt;
use thirtyfour::prelude::*;
use crate::selector;
use serde::{Deserialize};
use serde::de::{
    MapAccess
};

use serde::Deserializer;

#[derive(Debug, Clone) ]
pub struct MySelector {
    selector: String,
    selector_type: thirtyfour::By,
}

impl MySelector{
    pub fn new(selector: String, by_strategy: thirtyfour::By) -> Self {
        MySelector { selector : selector, selector_type : by_strategy}
    }

    pub async fn get_selector(&self) -> &str{
        self.selector.as_str()
    }

    pub async fn get_type(self) -> thirtyfour::By{
        self.selector_type
    }

}

impl<'de> Deserialize<'de> for MySelector {
    fn deserialize<D>(deserializer: D) -> std::result::Result<selector::MySelector, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            MySelector,
            Type,
        }

        struct SelectorVisitor;

        impl<'de> serde::de::Visitor<'de> for SelectorVisitor {
            type Value = MySelector;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an object with two fields: \"selector\", \"type\"")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MySelector, <V as MapAccess<'de>>::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut selector_val : Option<String> = None;
                let mut selector_type_val = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MySelector => {
                            selector_val = Some(map.next_value()?);
                        }
                        Field::Type => {
                            selector_type_val = Some(map.next_value()?);
                        }
                    }
                }

                let selector = selector_val.ok_or_else(|| serde::de::Error::missing_field("selector"))?;
                let selector_type = selector_type_val.ok_or_else(|| serde::de::Error::missing_field("type"))?;

                let by = match selector_type {
                    "css" => By::Css(selector.clone()),
                    "xpath" => By::XPath(selector.clone()),
                    "id" => By::Id(selector.clone()),
                    "name" => By::Name(selector.clone()),
                    "class_name" => By::ClassName(selector.clone()),
                    "tag" => By::Tag(selector.clone()),
                    "link_text" => By::LinkText(selector.clone()),
                    "partial_link_text" => By::PartialLinkText(selector.clone()),
                    "test_id" => By::Testid(selector.clone()),
                    other => return Err(serde::de::Error::unknown_variant(other, &[
                        "css", "xpath", "id", "name", "class_name", "tag", "link_text", "partial_link_text", "test_id",
                    ])),
                };

                Ok(MySelector { selector, selector_type: by })
            }
        }

        deserializer.deserialize_map(SelectorVisitor)
    }
}