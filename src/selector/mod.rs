
use thirtyfour::prelude::*;
use std::{collections::{HashMap}, fmt};
use serde::{Deserialize, Serialize, Serializer, de::{self,}, ser::SerializeStruct};

#[derive(Clone) ]
pub struct MySelector {
    name : String,
    selector: String,
    
    selector_strategy: thirtyfour::By,
}

impl fmt::Debug for MySelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySelector")
            .field("username", &self.name)
            .field("email", &self.selector)
            .finish()
    }
}

impl Serialize for MySelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MySelector", 2)?;
        state.serialize_field("selector", &self.selector)?;
        state.serialize_field("selector_strategy", format!("{:?}", "empty").as_str())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MySelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempData {
            #[serde(flatten)]
            data: HashMap<String, String>,
        }

        let temp_data: TempData = TempData::deserialize(deserializer)?;

        let mut iter = temp_data.data.into_iter();

        let first_pair = iter.next().ok_or_else(|| de::Error::custom("Missing first pair"))?;
        let second_pair = iter.next().ok_or_else(|| de::Error::custom("Missing second pair"))?;

        let (key1, val1) = first_pair;
        let (key2, val2) = second_pair;

        let name : String;

        let (selector_strategy, selector) = if key1.contains("type") || key1.contains("selector_strategy") {
            (val1, val2)
        } else {
            (val2, val1)
        };

        if key1.contains("type") || key2.contains("selector_strategy"){
            name = key2;
        }else{
            name = key1;
        }
        
        let selector_strategy = match selector_strategy.as_str() {
            "css" => By::Css(selector.clone()),
            "xpath" => By::XPath(selector.clone()),
            "id" => By::Id(selector.clone()),
            "name" => By::Name(selector.clone()),
            "class_name" => By::ClassName(selector.clone()),
            "tag" => By::Tag(selector.clone()),
            "link_text" => By::LinkText(selector.clone()),
            "partial_link_text" => By::PartialLinkText(selector.clone()),
            "test_id" => By::Testid(selector.clone()),
            _ => return Err(de::Error::custom(format!("Unknown selector type: {}", selector_strategy.as_str()))),
        };

        Ok(MySelector {
            name,
            selector,
            selector_strategy,
        })
    }
}

impl MySelector{
    pub fn new(name : String,selector: String, by_strategy: thirtyfour::By) -> Self {
        MySelector {name : name , selector : selector, selector_strategy : by_strategy}
    }

    pub fn get_selector(&self) -> &str{
        self.selector.as_str()
    }

    pub fn get_name(&self) -> &str{
        self.name.as_str()
    }

    pub fn get_selector_non_async(&self) -> &str{
        self.selector.as_str()
    }

    pub fn get_by(&self) -> By{
        self.selector_strategy.clone()
    }

}

