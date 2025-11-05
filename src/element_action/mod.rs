use thirtyfour::{error::WebDriverErrorInfo, prelude::*};

use crate::selector::MySelector;

pub struct ElementAction<'a> {
    driver: &'a WebDriver,
    selector: &'a str,
    by_strategy: &'a dyn Fn(&'a str) -> By,

    my_selector : MySelector
}

impl<'a> ElementAction<'a> {
    pub fn new(driver: &'a WebDriver, selector: &'a str, by_strategy: &'a dyn Fn(&'a str) -> By, my_selector: MySelector ) -> Self {
        ElementAction { driver, selector, by_strategy , my_selector : my_selector}
    }

    pub async fn exists(&self) -> Result<bool, WebDriverError> {
        match self.driver.find((self.by_strategy)(self.selector)).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn is_displayed(&self) -> Result<bool, WebDriverError> {
        let element = self.driver.find((self.by_strategy)(self.selector)).await?;
        element.is_displayed().await
    }

    pub async fn is_clickable(&self) -> Result<bool, WebDriverError> {
        let element = self.driver.find((self.by_strategy)(self.selector)).await?;
        element.is_clickable().await
    }

    
    pub async fn click(&self) -> Result<(), WebDriverError> {
        let element = self.driver.find((self.by_strategy)(self.selector)).await?;
        element.click().await
    }

    pub async fn safe_click(&self) -> Result<(), WebDriverError> {
        if self.exists().await? && self.is_displayed().await? && self.is_clickable().await? {
            self.click().await?;
            Ok(())
        } else {
            Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("cant perform action on element which does not meet action requirements".to_string())))
        }
    }

    pub async  fn send_keys(&self, keys : String) -> Result<(), WebDriverError>{
        let element = self.driver.find((self.by_strategy)(self.selector)).await?;

        match element.send_keys(keys).await{
            Ok(())=>{
                Ok(())
            },
            Err(_)=>{
                Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("Cannot send keys to element".to_string())))
            }
        }
    }

    pub async fn try_exists(action: &ElementAction<'_>, retries: u32) -> Result<bool, WebDriverError> {
        let mut last_error = None;

        for attempt in 1..= retries {
            match action.exists().await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    last_error = Some(e);
                    println!("'exists' call attempt {}", attempt);
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("Element doesnt exists".to_string()))))
    }

    pub async fn try_safe_click(action: &ElementAction<'_>, retries: u32) -> Result<(), WebDriverError>{
        for attempt in 1..= retries {
            match action.safe_click().await {
                Ok(_) => (),
                Err(_) => {
                    println!("safe_click attempt {}", attempt);
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }

        Ok(())
    }
}