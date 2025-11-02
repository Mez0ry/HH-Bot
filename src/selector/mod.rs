
use thirtyfour::prelude::*;
use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize, Serializer, de::{self,}, ser::SerializeStruct};

#[derive(Debug, Clone) ]
pub struct MySelector {
    selector: String,
    selector_type: By,
}

// Реализация сериализации для удобства отладки
impl Serialize for MySelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MySelector", 2)?;
        state.serialize_field("selector", &self.selector)?;
        state.serialize_field("selector_type", format!("{:?}", self.selector_type).as_str())?;
        state.end()
    }
}

// Реализация десериализации
impl<'de> Deserialize<'de> for MySelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // Временная структура для приема любых пар ключ-значение
        #[derive(Deserialize)]
        struct TempData {
            #[serde(flatten)]
            data: HashMap<String, String>,
        }

            let temp_data: TempData = TempData::deserialize(deserializer)?;

        // Забираем первые две пары из упорядоченного множества
        let mut iter = temp_data.data.into_iter();

        // Первая пара (ключ, значение)
        let first_pair = iter.next().ok_or_else(|| de::Error::custom("Missing first pair"))?;
        let second_pair = iter.next().ok_or_else(|| de::Error::custom("Missing second pair"))?;

        // Определяем роли каждой пары
        let (key1, val1) = first_pair;
        let (key2, val2) = second_pair;

        // Назначаем роли (можно менять местами в зависимости от нужд)
        let (selector_type, selector) = if key1.contains("type") || key1.contains("selector_type") {
            (val1, val2)
        } else {
            (val2, val1)
        };

        let selector_type = match selector_type.as_str() {
            "css" => By::Css(selector.clone()),
            "xpath" => By::XPath(selector.clone()),
            "id" => By::Id(selector.clone()),
            "name" => By::Name(selector.clone()),
            "class_name" => By::ClassName(selector.clone()),
            "tag" => By::Tag(selector.clone()),
            "link_text" => By::LinkText(selector.clone()),
            "partial_link_text" => By::PartialLinkText(selector.clone()),
            "test_id" => By::Testid(selector.clone()),
            _ => return Err(de::Error::custom(format!("Unknown selector type: {}", selector_type.as_str()))),
        };

        Ok(MySelector {
            selector,
            selector_type,
        })
    }
}

impl MySelector{
    pub fn new(selector: String, by_strategy: thirtyfour::By) -> Self {
        MySelector { selector : selector, selector_type : by_strategy}
    }

    pub async fn get_selector(&self) -> &str{
        self.selector.as_str()
    }

    pub fn get_selector_non_async(&self) -> &str{
        self.selector.as_str()
    }

    pub async fn get_type(self) -> thirtyfour::By{
        self.selector_type
    }

}

