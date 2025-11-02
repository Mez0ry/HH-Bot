use std::{sync::{Arc, OnceLock}};
use crate::selector::{MySelector};

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
        GLOBAL_SELECTORS.set(Arc::new(selectors)); // Убираем сложную конструкцию с match

        println!("Global state set successfully!");

        match GLOBAL_SELECTORS.get() {
            Some(arc_selectors) => {
                println!("Iterating over selectors...");
                for elem in arc_selectors.iter() {
                    let selector = elem.get_selector().await;
                    println!("Selector: {}, selector_type: {}", selector, elem.clone().get_type().await);
                }
            },
            None => eprintln!("Failed to retrieve global selectors after setting."),
        };

        Ok(())
    }

   pub async fn find_selector(selector: &str) -> Option<MySelector> {
    println!("pre find_selector");
    match GLOBAL_SELECTORS.get() {
        Some(selectors) => {
            for s in selectors.iter() {
                if s.get_selector().await == selector {
                    println!("found, selector: {}", s.get_selector().await);
                    return Some(s.clone());
                }
            }
            None
        },
        None => None,
    }
}
}