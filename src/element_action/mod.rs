use std::{pin::Pin, sync::Arc};

use thirtyfour::{error::WebDriverErrorInfo, prelude::*};
use crate::{retry, selector::MySelector};

#[derive(Clone)]
pub struct ElementAction {
    driver: Arc<WebDriver>,
    selector : MySelector
}

impl ElementAction {
    pub fn new(driver: Arc<WebDriver>, my_selector: MySelector ) -> Self {
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

    pub async fn is_clickable(&self) -> bool {
        let element = self.find_element().await;

        match element {
            Ok(element)=>{
                match element.is_clickable().await{
                    Ok(success)=> return success,
                    Err(_) => return false
                }
            },
            Err(err)=>{
                println!("Failed to find element, err: {}", err);
            }
        }
        false
    }
    
    pub async fn click(&self) -> Result<(), WebDriverError> {
        let element = self.find_element().await?;
        element.click().await
    }

    pub async fn safe_click(&self) -> Result<(), WebDriverError> {
        if self.exists().await? && self.is_displayed().await? && self.is_clickable().await {
            self.click().await?;
            Ok(())
        } else {
            Err(WebDriverError::ElementNotInteractable(WebDriverErrorInfo::new("cant perform action on element which does not meet action requirements".to_string())))
        }
    }

    pub async fn send_keys(&self, keys : String) -> bool{
        let result_element = self.find_element().await;

        let element = result_element.unwrap();

        match element.send_keys(keys).await{
            Ok(())=>{
                true
            },
            Err(err)=>{
                println!("Failed to send keys, error: {}", err);
                false
            }
        }
    }

    pub async fn try_exists(action: &Arc<ElementAction>, retries: u32) -> bool {
        match retry::retry_on_err(
            |attempt| {
                let retries = retries;
                Box::pin({
                let value = action.clone();
                async move {
                    match value.exists().await {
                        Ok(is_exists) => {
                            Ok(is_exists)
                        }
                        Err(_) => {
                            println!("element doesn't exist, attempt {}", attempt);

                            if attempt < retries {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }

                            return Err("element doesn't exist".to_string());
                        }
                    }
                }
                }) as Pin<Box<dyn Future<Output = Result<bool, String>> + Send + 'static>>
            },
            retries,
        ).await {
            Ok(is_exists) => {
               return is_exists;
            }, 
            Err(_) => false,
        }
    }

    pub async fn try_safe_click(action: &Arc<ElementAction>, retries: u32) -> bool{
        match retry::retry_on_err(
            |attempt| {
                let retries = retries;
                Box::pin({
                let cloned_action = action.clone();
                async move {
                    match cloned_action.safe_click().await {
                        Ok(_) => return Ok(true),
                        Err(_) => {
                            println!("safe_click attempt {}", attempt);
                            
                            if attempt < retries {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }

                            Err("element doesn't exist".to_string())
                        }
                    }
                }
                }) as Pin<Box<dyn Future<Output = Result<bool, String>> + Send + 'static>>
            },
            retries,
        ).await {
            Ok(is_safe_clicked) => {
               return is_safe_clicked;
            }, 
            Err(_) => false,
        }
    }

    async fn find_element(&self) -> WebDriverResult<WebElement>{
        self.driver.find(self.selector.clone().get_by()).await
    }

}