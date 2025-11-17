use std::sync::Arc;
use std::{fs::File, io::Write};
use thirtyfour::Cookie;
use thirtyfour::error::WebDriverErrorInfo;
use thirtyfour::prelude::*;
use tokio::fs::OpenOptions;
use crate::element_action::{self, ElementAction};
use crate::retry;
use crate::selector_manager::SelectorManager;
use std::pin::Pin;
pub struct CookieManager {
    
}

impl CookieManager {
    pub async fn load_cookies(path: &'static  str, driver : Arc<WebDriver>) -> bool {
        
        let result = retry::retry_on_err(
            |attempt| {
                let cloned_driver = driver.clone();
                Box::pin(async move {
                    let cookies = CookieManager::parse_cookies(&path, cloned_driver.clone()).await;

                    match cookies {
                        Ok(ref cookies) => {
                            match cookies {
                                Some(cookies) => {
                                    if cookies.is_empty() {
                                        println!("Cookies are empty");
                                        return Err("Cookies are empty");
                                    }

                                    for cookie in cookies {
                                        let _ = cloned_driver.add_cookie(cookie.clone()).await;
                                    }

                                    let _ = cloned_driver.refresh().await;

                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                                    let login_button: Arc<ElementAction> = element_action::ElementAction::new(cloned_driver.clone(), SelectorManager::find_selector("login_button").await).into();

                                    if ElementAction::try_exists(&login_button, 3).await {
                                        println!("Didn't logged in, retrying");
                                        return Err("Not logged in");
                                    }

                                    return Ok(true)
                                }
                                None => return Err("No cookies found"),
                            }
                        }
                        Err(err) => {
                            println!("Failed to parse cookies attempt: {}, error: {}", attempt, err);
                            return Err("Failed to parse cookies")
                        }
                    }
                }) as Pin<Box<dyn Future<Output = Result<bool, &str>> + Send + 'static>>
            },
            20,
        ).await;

        match result {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn save_cookies(path: &str, cookies: Vec<Cookie>) -> Result<(), WebDriverError> {
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

    async fn login(driver: Arc<WebDriver>) -> Result<(), WebDriverError> {
        driver.goto("http://hh.ru/login").await?;
        let _ = tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

        let submit_button: Arc<ElementAction> = element_action::ElementAction::new(driver.clone(), SelectorManager::find_selector("submit_button").await).into();
        
        if ElementAction::try_exists(&submit_button, 3).await{
            ElementAction::try_safe_click(&submit_button, 3).await;
        }
        
        Ok(())
    }

    pub async fn parse_cookies(path: &'static str, driver :Arc<WebDriver>) -> Result<Option<Vec<Cookie>>, WebDriverError> {

        let str_as_path = std::path::Path::new(path);

        if !str_as_path.exists() || str_as_path.is_dir() {
            OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path).await?;
        }

        const MAX_ATTEMPTS: u32 = 5222;
        const DELAY_BETWEEN_ATTEMPTS_MILLIS: u64 = 2000;
        let mut login_once: bool = false;
        
        retry::retry_on_err(|attempt| {
            let cloned_driver = driver.clone();
                Box::pin(async move {
                    let chat_activator_button = element_action::ElementAction::new(cloned_driver.clone(),SelectorManager::find_selector("chat_activator_button").await,).into();
                    let login_button = element_action::ElementAction::new(cloned_driver.clone(),SelectorManager::find_selector("login_button").await,).into();

                    let content = std::fs::read_to_string(&str_as_path)
                        .map_err(|e| WebDriverError::InvalidArgument(WebDriverErrorInfo::new(format!("File read error: {}", e))))?;

                    let parsed_cookies: Result<Vec<Cookie>, serde_json::Error> = serde_json::from_str(&content);

                    match parsed_cookies {
                        Ok(cookies) => {
                            if cookies.is_empty() {
                                return Ok(None);
                            }
                            return Ok(Some(cookies));
                        },
                        Err(serde_err) => {
                            if attempt >= MAX_ATTEMPTS - 1 {
                                return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                                    format!("Couldn't parse cookies: {}", serde_err),
                                )));
                            }

                            if ElementAction::try_exists(&chat_activator_button, 3).await {
                                let cookies = cloned_driver.get_all_cookies().await?;

                                CookieManager::save_cookies(path, cookies.clone()).await?;
                                return Ok(Some(cookies));
                            } else if !login_once {
                                CookieManager::login(cloned_driver.clone()).await?;

                                let current_url = cloned_driver.current_url().await?.to_string();
                                if !current_url.contains("login") {
                                    let cookies = cloned_driver.get_all_cookies().await?;

                                    CookieManager::save_cookies(path, cookies.clone()).await?;
                                    return Ok(Some(cookies));
                                }

                                login_once = true;
                            } else if !ElementAction::try_exists(&login_button, 3).await
                                && !cloned_driver.current_url().await?.to_string().contains("login")
                            {
                                let cookies = cloned_driver.get_all_cookies().await?;

                                CookieManager::save_cookies(path, cookies.clone()).await?;
                                return Ok(Some(cookies));
                            }

                            println!("Login required, attempt: {}", attempt);
                            tokio::time::sleep(tokio::time::Duration::from_millis(DELAY_BETWEEN_ATTEMPTS_MILLIS)).await;
                        }
                    }

                    Ok(None)
                }) as Pin<Box<dyn Future<Output=Result<Option<Vec<Cookie>>, WebDriverError>> + Send + 'static>>
            },
            MAX_ATTEMPTS,
        ).await
    }
}
