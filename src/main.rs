use thirtyfour::{prelude::*};
extern crate num_cpus;

pub mod cookie_manager;
pub mod vacancy;
pub mod element_action;
pub mod selector;
pub mod selector_manager;
pub mod page;
pub mod retry;

use crate::{cookie_manager::CookieManager, page::{Page, PageProcessState}, selector_manager::SelectorManager};

type ThirtyFourError = thirtyfour::error::WebDriverError; // Исправили тип ошибки

#[tokio::main]
async fn main() -> Result<(), ThirtyFourError> {
    let mut caps = DesiredCapabilities::chrome();

    //caps.add_arg("--headless")?;
    
    caps.add_arg("--enable-debugger-agent")?;
    caps.add_arg("--remote-debugging-port=9222")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VoiceInteraction")?;
    caps.add_arg("--disable-speech-api")?;
    caps.add_arg("--disable-background-networking")?;
    caps.add_arg("--disable-background-timer-throttling")?;
    caps.add_arg("--disable-renderer-backgrounding")?;
    caps.add_arg("--disable-backgrounding-occluded-windows")?;
    caps.add_arg("--disable-client-side-phishing-detection")?;
    caps.add_arg("--disable-sync")?;
    caps.add_arg("--disable-translate")?;
    caps.add_arg("--disable-ipc-flooding-protection")?;
    caps.add_arg("--log-level=3")?;
    caps.add_arg("--enable-unsafe-swiftshader")?;
    
    let driver = WebDriver::new("http://localhost:64904", caps).await?;

    let target_url : String = "https://hh.ru/search/vacancy?text=%D0%9F%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%81%D1%82+C%2B%2B&salary=&ored_clusters=true&enable_snippets=true&hhtmFrom=vacancy_search_list&hhtmFromLabel=vacancy_search_line".to_string();

    driver.goto(&target_url).await?;
    
    let cookie_json_path: String = String::from("./resources/cookies.json");

    if CookieManager::load_cookies(&cookie_json_path, &driver).await{
        println!("Successfully parsed and applied cookies");
        driver.goto(&target_url).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let _ = SelectorManager::load_selectors("./resources/selectors.json").await;

    let cores_amount = num_cpus::get();
    println!("Cpu cores: {}", cores_amount);
    
    let mut page = Page::new(target_url, &driver);
    
    match page.process_page().await{
        PageProcessState::GatheringVacancies =>{
            println!("failed on gathering vacancies");
        },
        PageProcessState::ProcessingVacancy =>{
            println!("failed on processing vacancies");
        },
        PageProcessState::ReGatheringVacancies =>{
            println!("Failed on regathering vacancies")
        },
        PageProcessState::PageProcessed =>{
            println!("Successfully processed page");
        },
    }

    CookieManager::save_cookies(cookie_json_path, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    driver.quit().await?;

    Ok(())
}