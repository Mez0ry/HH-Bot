use std::{collections::HashSet};
use thirtyfour::prelude::*;

use tokio::stream;
use futures;
use futures::stream::{StreamExt, TryStreamExt};

pub mod cookie_manager;
pub mod vacancy;

use crate::{cookie_manager::cookie_manager::CookieManager, vacancy::vacancy::Vacancy};

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

    let driver = WebDriver::new("http://localhost:64876", caps).await?;

    let target_url = "https://hh.ru/search/vacancy?text=%D0%9F%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%81%D1%82+C%2B%2B&salary=&ored_clusters=true&enable_snippets=true&hhtmFrom=vacancy_search_list&hhtmFromLabel=vacancy_search_line";

    driver.goto(target_url).await?;

    //let cookie_manager = cookie_manager::cookie_manager::CookieManager::new("../resources/Cookies/cookies.json".to_string());
    let cookie_json_path: String = String::from("./resources/cookies.json");

    if let Some(cookies) = CookieManager::load_cookies(&cookie_json_path).await?{
        for cookie in cookies {
            driver.add_cookie(cookie).await?;
        }
    }

    driver.refresh().await?;

    // let respond_button_selector = "[data-qa=\"vacancy-serp__vacancy_response\"]";
    // let mut respond_buttons = driver.find_all(By::Css(respond_button_selector)).await?;
    
    // if respond_buttons.is_empty(){
    //     //handle
    // }

    // respond_buttons.iter().next_back().unwrap().wait_until().clickable().await?;

    let mut responded_buttons_set: HashSet<String> = std::collections::HashSet::new();

    let vacancy_selector = "[class=\"vacancy-card--n77Dj8TY8VIUF0yM font-inter\"]";
    let mut all_vacancies = driver.find_all(By::Css(vacancy_selector)).await?;
    
    all_vacancies.iter().next_back().unwrap().wait_until().clickable().await?;

    let mut vacancies_vec : Vec<Vacancy> = vec![];
    
    for vacancy_element in &all_vacancies{
        let mut vacancy = Vacancy::new(vacancy_element.clone());
        vacancy.update_vacancy_fields(&driver).await;

        vacancies_vec.push(vacancy);
    }

    loop{

        for vacancy in &vacancies_vec{
            dbg!("Handling vacancy: title: {}", vacancy.get_title().await);

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let respond_button = vacancy.get_button().await;

            if respond_button.is_none(){
                continue;
            }

            let respond_button = respond_button.unwrap();

            let button_href = respond_button.attr("href").await?;

            if button_href.clone().is_some_and(|href| responded_buttons_set.contains(&href)){
                dbg!("already responded on this vacancy {}", button_href.as_ref().unwrap());
                continue;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            vacancy.click_respond().await;

            let relocation_warning_popup = match driver.find(By::Css("[data-qa=\"relocation-warning-confirm\"]")).await {
                Ok(el) => Some(el),
                Err(_) => {
                    // let href_copy = button_href.clone();
                    eprintln!("Failed to locate relocation warning popup:");
                    None
                },
            };

            if let Some(popup) = relocation_warning_popup {
                // Работаем с найденным элементом p
                let is_displayed = match popup.is_displayed().await {
                    Ok(displayed) => displayed,
                    Err(_) => {
                        eprintln!("Failed to determine if relocation warning popup is displayed");
                        false
                    },
                };
        
                let is_clickable = match popup.is_clickable().await {
                    Ok(clickable) => clickable,
                    Err(_) => {
                        eprintln!("Failed to determine if relocation warning popup is clickable");
                        false
                    },
                };
        
                if is_displayed && is_clickable {
                    match popup.click().await {
                        Ok(_) => eprintln!("Relocation warning popup clicked successfully"),
                        Err(err) => eprintln!("Failed to click relocation warning popup: {}", err),
                    };
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
            let submit_button = driver.find(By::Css(submit_button_selector)).await;

            match submit_button{
                Ok(submit_button_el) =>{
                    submit_button_el.wait_until().displayed().await?;

                    let response_letter_toggle = "[data-qa=\"vacancy-response-letter-toggle\"]";
                    let response_letter_toggle_button = driver.find(By::Css(response_letter_toggle)).await;

                    match response_letter_toggle_button {
                        Ok(response_letter_toggle_button_el)=>{
                            match response_letter_toggle_button_el.click().await{
                                Ok(_)=>{
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                },
                                Err(_)=>{
                                    dbg!("couldnt be pressed");
                                }
                            }
                        },
                        Err(err)=>{
                            dbg!("Didnt found response letter toogle button, error: {}", err);
                        }
                    };

                    let response_popup_form_selector = "[data-qa=\"vacancy-response-popup-form-letter-input\"]";
                    let letter_input_form = driver.find(By::Css(response_popup_form_selector)).await;

                    match letter_input_form{
                        Ok(letter_input_form_el) => {

                            letter_input_form_el.click().await?;
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            letter_input_form_el.send_keys("test").await?;

                            let submit_button_selector = "[data-qa=\"vacancy-response-submit-popup\"]";
                            let submit_button = driver.find(By::Css(submit_button_selector)).await?;

                            submit_button.wait_until().displayed().await?;
                            submit_button.wait_until().enabled().await?;
                            submit_button.wait_until().clickable().await?;

                            match submit_button.click().await {
                                Ok(_)=>{
                                    dbg!("Submit button inside letter form pressed");
                                },
                                Err(err)=>{
                                    dbg!("Submit button can't be pressed: {}", err);
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        },
                        Err(err)=> {
                            dbg!("Didnt found response letter toogle button, error: {}", err);
                        }
                    };

                    match button_href{
                        Some(href) => {
                            responded_buttons_set.insert(href.to_string());
                            dbg!("responded_buttons_set.insert, href{}", href);
                        },
                        None => {dbg!("button_href None");}
                    }

                },
                Err(_)=>{
                    dbg!("submit button ERR");
                    continue;
                }
            }

            let current_url = driver.current_url().await?;

            if current_url.as_str() != target_url {
                driver.goto(target_url).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                break;
            } else {
                dbg!("Already on desired page, skipping navigation");
            }
        }

        dbg!("repeat");

        all_vacancies.clear();

        all_vacancies = driver.find_all(By::Css(vacancy_selector)).await?;

        if !all_vacancies.is_empty(){
            all_vacancies.iter().next_back().unwrap().wait_until().clickable().await?;
            vacancies_vec.clear();
            
            for vacancy_element in &all_vacancies{
                let mut vacancy = Vacancy::new(vacancy_element.clone());
                vacancy.update_vacancy_fields(&driver).await;
                
                let respond_button = vacancy.get_button().await;

                if respond_button.is_none(){
                    continue;
                }

                let respond_button = respond_button.unwrap();

                let button_href = respond_button.attr("href").await?;

                if button_href.clone().is_some_and(|href| responded_buttons_set.contains(&href)){
                    dbg!("already responded on this vacancy {}", button_href.as_ref().unwrap());
                    continue;
                }
                
                vacancies_vec.push(vacancy);
                
            }

        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }// !loop

    CookieManager::save_cookies(cookie_json_path, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    driver.quit().await?;

    Ok(())
}