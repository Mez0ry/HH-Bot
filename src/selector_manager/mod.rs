use std::{sync::{Arc, OnceLock}};
use thirtyfour::By;

use crate::{element_action::{self, ElementAction}, selector::MySelector};

// Type for storing the global vector
type SelectorsVecType = Vec<MySelector>;

static GLOBAL_SELECTORS: OnceLock<Arc<SelectorsVecType>> = OnceLock::new();

pub struct SelectorManager {}

impl SelectorManager {
    pub async fn load_selectors(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting loading selectors...");

        let content = tokio::fs::read_to_string(path).await?;
        println!("Successfully read file contents!");

        let selectors: Result<Vec<MySelector>, serde_json::Error> = serde_json::from_str(&content);
        match selectors{
            Ok(_)=>{},
            Err(err)=>{println!("Error in parsing content: {}", err);}
        }

        let selectors: Vec<MySelector> = serde_json::from_str(&content)?;

        println!("Deserialization successful!");

        if selectors.is_empty() {
            eprintln!("selectors empty");
        } else {
            eprintln!("non-empty selectors count: {}", selectors.len());
        }

        println!("Setting global state...");
        let _ = GLOBAL_SELECTORS.set(Arc::new(selectors)); // Убираем сложную конструкцию с match

        println!("Global state set successfully!");

        match GLOBAL_SELECTORS.get() {
            Some(arc_selectors) => {
                println!("Iterating over selectors...");
                for elem in arc_selectors.iter() {
                    let selector = elem.get_selector();
                    println!("Selector: {}", selector);
                }
            },
            None => eprintln!("Failed to retrieve global selectors after setting."),
        };

        Ok(())
    }

   pub async fn find_selector(selector_name: &str) -> Option<MySelector> {
        println!("pre find_selector");
        match GLOBAL_SELECTORS.get() {
            Some(selectors) => {
                for s in selectors.iter() {
                    if s.get_name() == selector_name {
                        println!("found, selector: {}", s.get_selector());
                        return Some(s.clone());
                    }
                }
                None
            },
            None => None,
        }
    }

    pub async fn find_selector_as_action<'a>(driver : &'a thirtyfour::WebDriver, selector_name: &'a str) -> Option<ElementAction<'a>>{
        let selector = Self::find_selector(selector_name).await?;
        
        Some(element_action::ElementAction::new(driver, selector.get_selector(), &selector.get_type_as_callback()))
    }

}