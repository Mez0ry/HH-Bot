use thirtyfour::{error::WebDriverErrorInfo, prelude::*};
use crate::selector::MySelector;

pub struct ElementAction<'a> {
    driver: &'a WebDriver,
    selector : MySelector
}

impl<'a> ElementAction<'a> {
    pub fn new(driver: &'a WebDriver, my_selector: MySelector ) -> Self {
        ElementAction { driver : driver, selector : my_selector}
    }

    pub async fn exists(&self) -> Result<bool, WebDriverError> {
        match self.find_element().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn is_displayed(&self) -> Result<bool, WebDriverError> {
        let element = self.find_element().await?;
        element.is_displayed().await
    }

    pub async fn is_clickable(&self) -> Result<bool, WebDriverError> {
        let element = self.find_element().await?;
        element.is_clickable().await
    }
    
    pub async fn click(&self) -> Result<(), WebDriverError> {
        let element = self.find_element().await?;
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
        let element = self.find_element().await?;

        match element.send_keys(keys).await{
            Ok(())=>{
                Ok(())
            },
            Err(_)=>{
                Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("Cannot send keys to element".to_string())))
            }
        }
    }

    pub async fn try_exists(action: &ElementAction<'_>, retries: u32) -> bool {
        let mut last_error = None;

        for attempt in 1..= retries {
            match action.exists().await {
                Ok(stream) => return stream,
                Err(e) => {
                    last_error = Some(e);
                    println!("'exists' call failed, attempt {}", attempt);
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }
        
        println!("Element doesnt exists: {}", last_error.unwrap());
        false
    }

    pub async fn try_safe_click(action: &ElementAction<'_>, retries: u32) -> bool{
        for attempt in 1..= retries {
            match action.safe_click().await {
                Ok(_) => return true,
                Err(_) => {
                    println!("safe_click attempt {}", attempt);
                    if attempt < retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }

        false
    }

    async fn find_element(&self) -> WebDriverResult<WebElement>{
        self.driver.find(self.selector.clone().get_by()).await
    }

}