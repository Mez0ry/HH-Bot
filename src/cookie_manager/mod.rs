pub mod cookie_manager {

    use std::{fs::File, io::Write};
    use std::sync::{Mutex};
    use thirtyfour::Cookie;
    use thirtyfour::error::WebDriverErrorInfo;
    use thirtyfour::prelude::*;
    use crate::element_action::{self, ElementAction};

    static ONCE_LOCK: Mutex<Option<()>> = Mutex::new(None);

    pub struct CookieManager {
        
    }

    impl CookieManager {
        pub async fn load_cookies(path: &String, driver : &WebDriver) -> Result<Option<Vec<Cookie>>, WebDriverError> {
            let cookies = CookieManager::parse_cookies(path, driver).await;
            match &cookies {
                Ok(_data) => {
                    if _data.is_some() && _data.clone().unwrap().is_empty() {
                        return Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("cookies are empty".to_string())));
                    }
                    println!("Cookies successfully deserialized and applied");
                },
                Err(err) =>{ 
                    println!("error is:  {}", err)
                },
            }

            return cookies;
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
            let guard = ONCE_LOCK.lock();

            if guard.is_ok() {
                let css_strategy = |selector: &str| By::Css(selector.to_owned());

                driver.goto("http://hh.ru/login").await?;
                let submit_button = element_action::ElementAction::new(driver, "[data-qa=\"submit-button\"]", &css_strategy);
                
                if ElementAction::try_exists(&submit_button, 3).await?{
                    ElementAction::try_safe_click(&submit_button, 3).await?;
                }

                *guard.unwrap() = Some(());
            }

            Ok(())
        }

        pub async fn parse_cookies(path: &String, driver : &WebDriver) -> Result<Option<Vec<Cookie>>, WebDriverError> {
            let css_strategy = |selector: &str| By::Css(selector.to_owned());

            let logged_in = element_action::ElementAction::new(driver, "[class=\"magritte-component-with-badge___p49ZX_3-2-5\"]", &css_strategy);
            
            let str_as_path = std::path::Path::new(&path);

            if !str_as_path.exists() || str_as_path.is_dir() {
                File::create(path)?;
            }

            const MAX_ATTEMPTS: usize = 200;
            const DELAY_BETWEEN_ATTEMPTS_MILLIS: u64 = 2000;
            let mut login_once: bool = false;
            
            for attempt in 0..MAX_ATTEMPTS {
                let content = std::fs::read_to_string(&str_as_path)?;
                let parsed_cookies: Result<Vec<Cookie>, serde_json::Error> = serde_json::from_str(&content);
                
                match parsed_cookies {
                    Ok(cookies) => {
                        if cookies.is_empty() {
                            return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                                "Cookies are empty".to_string(),
                            )));
                        }
                        return Ok(Some(cookies));
                    },
                    Err(serde_err) => {
                        if attempt >= MAX_ATTEMPTS - 1 {
                            return Err(WebDriverError::InvalidArgument(WebDriverErrorInfo::new(
                                format!("Couldn't parse cookies: {}", serde_err),
                            )));
                        }

                        if ElementAction::try_exists(&logged_in, 3).await?{
                            let cookies = driver.get_all_cookies().await?;

                            CookieManager::save_cookies(path.to_string(), driver.get_all_cookies().await?).await?;
                            return Ok(Some(cookies));
                        }else{
                            println!("Loggin required, attempt: {}", attempt)
                        }
                        if !login_once{
                            CookieManager::login(driver).await?;
                            login_once = true;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(DELAY_BETWEEN_ATTEMPTS_MILLIS)).await;
                    }
                }
            }

            unreachable!();
        }
    }
}
