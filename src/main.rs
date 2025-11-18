use std::{path::{PathBuf}, sync::Arc};

use clap::{Arg, Command};

use thirtyfour::{prelude::*};
use tokio::fs::{self};

pub mod cookie_manager;
pub mod vacancy;
pub mod element_action;
pub mod selector;
pub mod selector_manager;
pub mod page;
pub mod retry;


use crate::{cookie_manager::CookieManager, page::{Page, PageProcessState}, selector_manager::SelectorManager};

type ThirtyFourError = thirtyfour::error::WebDriverError; 

#[tokio::main]
async fn main() -> Result<(), ThirtyFourError> {

    let matches = Command::new("HHBot")
        .version("1.0")
        .author("Mez0ry mez0ry@mail.ru")
        .about("Accepts a target URL as an argument")
        .arg(
            Arg::new("target_url")
            .long("target_url")
            .value_name("URL")
            .help("Sets the target URL")
            .required(true),
        )
        .arg(
            Arg::new("headless_mode")
            .long("headless_mode")
            .value_name("mode")
            .help("if true runs browser in headless mode")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(
            Arg::new("stealth_mode")
            .long("stealth_mode")
            .value_name("mode")
            .help("if true runs headless browser in a way that it hides that it was ran in headless mode")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .get_matches();
    
    let mut target_url : &String = &String::new();

    if let Some(target_url_arg) = matches.get_one::<String>("target_url") {
        target_url = target_url_arg;
        println!("Setting up target url: {}", target_url);
    }

    let mut caps = DesiredCapabilities::chrome();

    
    if matches.get_flag("headless_mode"){
        caps.add_arg("--headless")?;
        println!("Headless mode: enabled");
    }
    
    caps.add_arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")?;
    
    if matches.get_flag("stealth_mode"){

        let shared_profile_dir_path = std::path::Path::new("./resources/user/shared_profile");

        if !shared_profile_dir_path.exists(){
            fs::create_dir_all(shared_profile_dir_path).await?;
        }

        let path = fs::canonicalize(PathBuf::from("./resources/user/shared_profile")).await?;
        
        let mut user_data_dir = "--user-data-dir=".to_string();
        
        if let Some(actual_path) = path.to_str() {
            let cleaned_path = if actual_path.starts_with(r#"\\?\"#) {
                &actual_path[4..]
            } else {
                actual_path
            };
            user_data_dir.push_str(cleaned_path);
        }
        
        caps.add_arg(&user_data_dir)?;
    }

    caps.add_arg("--hide-scrollbars")?;
    caps.add_arg("--mute-audio")?;

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
    
    let driver: Arc<WebDriver> = Arc::new(WebDriver::new("http://localhost:53609", caps).await?);
    
    driver.goto(target_url).await?;

    if matches.get_flag("stealth_mode"){
        let _ = driver.execute(r#"Object.defineProperty(navigator, 'webdriver', {
                                    get: () => undefined
                                    });
                                    "#,Vec::new()).await;
    
        let _ = driver.execute(r"Intl.DateTimeFormat = () => ({resolvedOptions: () => ({timeZone: 'America/New_York'})})", Vec::new()).await;
    
        let _ = driver.execute(r#"
                                    Object.defineProperty(navigator, 'webdriver', {
                                    get: () =&gt; undefined
                                    });
                                    Object.defineProperty(navigator, 'languages', {
                                    get: () =&gt; ['en-US', 'en']
                                    });
                                    Object.defineProperty(navigator, 'plugins', {
                                    get: () =&gt; [1, 2, 3]
                                    });
                                    "#, Vec::new()).await;
        println!("Stealth mode enabled");
    }
    
    let cookie_json: &'static str = &"./resources/user/cookies.json";

    if CookieManager::load_cookies(&cookie_json, driver.clone()).await{
        println!("Successfully parsed and applied cookies");
        driver.goto(target_url).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let _ = SelectorManager::load_selectors("./resources/selectors.json").await;
    
    let mut page = Page::new(target_url.clone(), driver.clone());
    
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

    CookieManager::save_cookies(&cookie_json, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    <thirtyfour::WebDriver as Clone>::clone(&driver).quit().await?;
    
    Ok(())
}