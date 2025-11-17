use std::{sync::{Arc, OnceLock}};
use thirtyfour::{By, WebDriver};
/**
 * @TODO implement mechanism of waiting until selectors are loaded in multithreading enviroment
 */
use crate::{element_action::{self, ElementAction}, selector::MySelector};

type SelectorsVecType = Vec<MySelector>;

static GLOBAL_SELECTORS: OnceLock<Arc<SelectorsVecType>> = OnceLock::new();

pub struct SelectorManager {}

impl SelectorManager {
    /**
     * @TODO rewrite
     */
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

   pub async fn find_selector<S: AsRef<str>>(selector_name: S) -> MySelector{
        match GLOBAL_SELECTORS.get() {
            Some(selectors) => {
                for s in selectors.iter() {
                    if s.get_name() == selector_name.as_ref() {
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

    pub async fn find_selector_as_action(driver : Arc<WebDriver>, selector_name: String) -> Option<ElementAction>{
        Some(element_action::ElementAction::new(driver.clone(), Self::find_selector(selector_name.clone()).await))
    }

}