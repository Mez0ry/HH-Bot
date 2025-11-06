use std::{sync::{Arc, OnceLock}};

use thirtyfour::By;

use crate::{element_action::{self, ElementAction}, selector::MySelector};

type SelectorsVecType = Vec<MySelector>;

static GLOBAL_SELECTORS: OnceLock<Arc<SelectorsVecType>> = OnceLock::new();

pub struct SelectorManager {}

impl SelectorManager {
    pub async fn load_selectors(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = tokio::fs::read_to_string(path).await?;

        let selectors: Result<Vec<MySelector>, serde_json::Error> = serde_json::from_str(&content);
        match selectors{
            Ok(_)=>{},
            Err(err)=>{println!("Error in parsing content: {}", err);}
        }

        let selectors: Vec<MySelector> = serde_json::from_str(&content)?;

        if selectors.is_empty() {
            eprintln!("selectors empty");
        }

        let _ = GLOBAL_SELECTORS.set(Arc::new(selectors));

        Ok(())
    }

   pub async fn find_selector(selector_name: &str) -> MySelector{
        match GLOBAL_SELECTORS.get() {
            Some(selectors) => {
                for s in selectors.iter() {
                    if s.get_name() == selector_name {
                        return s.clone();
                    }
                }
                panic!("My selector wasn't found");
            },
            None =>{
                MySelector::new("".to_string(), "".to_string(), By::Css(""))
            }
        }
    }

    pub async fn find_selector_as_action<'a>(driver : &'a thirtyfour::WebDriver, selector_name: &'a str) -> Option<ElementAction<'a>>{
        Some(element_action::ElementAction::new(driver, Self::find_selector(selector_name).await))
    }

}