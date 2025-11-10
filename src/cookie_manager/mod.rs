use std::{fs::File, io::Write};
use thirtyfour::Cookie;
use thirtyfour::error::WebDriverErrorInfo;
use thirtyfour::prelude::*;
use crate::element_action::{self, ElementAction};
use crate::retry;
use crate::selector_manager::SelectorManager;

pub struct CookieManager {
    
}

impl CookieManager {
    pub async fn load_cookies(path: &String, driver : &WebDriver) -> bool {
        
        retry::retry_on_err(async |attempt|{
            let cookies = CookieManager::parse_cookies(path, driver).await;

            match cookies {
                Ok(ref cookies)=>{
                    match cookies {
                        Some(cookies)=>{
                            if cookies.is_empty(){
                                println!("Cookies are empty");
                                return false;
                            }

                            for cookie in cookies {
                                let _ = driver.add_cookie(cookie.clone()).await;
                            }

                            let _ = driver.refresh().await;

                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            
                            
                            let login_button = element_action::ElementAction::new(driver, SelectorManager::find_selector("login_button").await);
                            // TODO validate that user logged in
                           
                            if ElementAction::try_exists(&login_button, 3).await{
                                println!("didnt logged in, retrying");
                                return false;
                            }
                            
                            return true;
                        },

                        None=>{
                            return false;
                        }
                    }
                },
                Err(err)=>{
                    println!("Failed to parse cookies attempt: {}, error: {}", attempt, err);
                }
            }
          

            return false;
        }, 20).await
    }

    pub async fn save_cookies(path: String, cookies: Vec<Cookie>) -> Result<(), WebDriverError> {
        if cookies.is_empty() {
            return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                "invalid argument, cookies are empty".to_string(),
            )));
        }

        let json_data = serde_json::to_string_pretty(&cookies)?;
        let mut file = File::create(&path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    async fn login(driver: &WebDriver) -> Result<(), WebDriverError> {
        driver.goto("http://hh.ru/login").await?;
        let _ = tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

        let submit_button = element_action::ElementAction::new(driver, SelectorManager::find_selector("submit_button").await);
        
        if ElementAction::try_exists(&submit_button, 3).await{
            ElementAction::try_safe_click(&submit_button, 3).await;
        }
        
        Ok(())
    }

    pub async fn parse_cookies(path: &String, driver : &WebDriver) -> Result<Option<Vec<Cookie>>, WebDriverError> {
        let str_as_path = std::path::Path::new(&path);

        if !str_as_path.exists() || str_as_path.is_dir() {
            File::create(path)?;
        }

        const MAX_ATTEMPTS: usize = 5222;
        const DELAY_BETWEEN_ATTEMPTS_MILLIS: u64 = 2000;
        let mut login_once: bool = false;

        for attempt in 0..MAX_ATTEMPTS {
            let chat_activator_button = element_action::ElementAction::new(driver, SelectorManager::find_selector("chat_activator_button").await);
            let login_button = element_action::ElementAction::new(driver, SelectorManager::find_selector("login_button").await);
            
            let content = std::fs::read_to_string(&str_as_path)?;
            let parsed_cookies: Result<Vec<Cookie>, serde_json::Error> = serde_json::from_str(&content);

            match parsed_cookies {
                Ok(cookies) => {
                    if cookies.is_empty() {
                        continue;
                    }

                    return Ok(Some(cookies));
                },
                Err(serde_err) => {
                    if attempt >= MAX_ATTEMPTS - 1 {
                        return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                            format!("Couldn't parse cookies: {}", serde_err),
                        )));
                    }
                    
                    if ElementAction::try_exists(&chat_activator_button, 3).await{
                        let cookies = driver.get_all_cookies().await?;

                        CookieManager::save_cookies(path.to_string(), driver.get_all_cookies().await?).await?;
                        return Ok(Some(cookies));
                    }else if !login_once{
                        CookieManager::login(driver).await?;

                        let current_url = driver.current_url().await?.to_string();
                        if !current_url.contains("login"){
                            let cookies = driver.get_all_cookies().await?;

                            CookieManager::save_cookies(path.to_string(), driver.get_all_cookies().await?).await?;
                            return Ok(Some(cookies));
                        }

                        login_once = true;
                    }else if !ElementAction::try_exists(&login_button, 3).await && !driver.current_url().await?.to_string().contains("login"){
                        let cookies = driver.get_all_cookies().await?;

                        CookieManager::save_cookies(path.to_string(), driver.get_all_cookies().await?).await?;
                        return Ok(Some(cookies));
                    }

                    println!("Login required, attempt: {}", attempt);
                    tokio::time::sleep(tokio::time::Duration::from_millis(DELAY_BETWEEN_ATTEMPTS_MILLIS)).await;
                }
            }
        }

        unreachable!();
    }
}
