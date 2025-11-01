use std::{collections::HashSet};
use thirtyfour::prelude::*;

pub mod cookie_manager;
pub mod vacancy;
pub mod element_action;

use crate::{cookie_manager::cookie_manager::CookieManager, element_action::ElementAction, vacancy::vacancy::Vacancy};

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

    //class="bloko-translate-guard"

    let driver = WebDriver::new("http://localhost:64876", caps).await?;

    let target_url = "https://hh.ru/search/vacancy?text=%D0%9F%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%81%D1%82+C%2B%2B&salary=&ored_clusters=true&enable_snippets=true&hhtmFrom=vacancy_search_list&hhtmFromLabel=vacancy_search_line";

    driver.goto(target_url).await?;
    
    let cookie_json_path: String = String::from("./resources/cookies.json");

    if let Some(cookies) = CookieManager::load_cookies(&cookie_json_path).await?{
        for cookie in cookies {
            driver.add_cookie(cookie).await?;
        }
    }

    driver.refresh().await?;

    let mut responded_buttons_set: HashSet<String> = std::collections::HashSet::new();

    let vacancy_selector = "[class=\"vacancy-card--n77Dj8TY8VIUF0yM font-inter\"]";
    let mut all_vacancies = driver.find_all(By::Css(vacancy_selector)).await?;
    
    all_vacancies.iter().next_back().unwrap().wait_until().clickable().await?;

    let mut vacancies_vec : Vec<Vacancy> = vec![];
    
    for vacancy_element in &all_vacancies{
        let mut vacancy = Vacancy::new(vacancy_element.clone());
        vacancy.update_vacancy_fields().await;

        vacancies_vec.push(vacancy);
    }

    let css_strategy = |selector: &str| By::Css(selector.to_owned());
    
    loop{
        let mut reached_limit = false;

        for vacancy in &vacancies_vec{
            println!("Handling vacancy: title: {}", vacancy.get_title().await);
            let respond_button = vacancy.get_button().await;

            if respond_button.is_none(){
                continue;
            }

            let respond_button = respond_button.unwrap();

            let button_href = respond_button.attr("href").await?;

            if button_href.clone().is_some_and(|href| responded_buttons_set.contains(&href)){
                continue;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            vacancy.click_respond().await;

            let limit_check = ElementAction::new(&driver, "[class=\"bloko-translate-guard\"]",&css_strategy);
            if ElementAction::try_exists(&limit_check, 3).await?{
                reached_limit = true;
                break;
            }

            let relocation_popup = ElementAction::new(&driver, "[data-qa=\"relocation-warning-confirm\"]", &css_strategy);

            if ElementAction::try_exists(&relocation_popup, 3).await?{
                //relocation_popup.safe_click().await?;
                ElementAction::try_safe_click(&relocation_popup,3).await?;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let submit_button = ElementAction::new(&driver, "[data-qa=\"vacancy-response-submit-popup\"]", &css_strategy);

            if ElementAction::try_exists(&submit_button, 3).await?{
                //submit_button.safe_click().await?;
                ElementAction::try_safe_click(&submit_button,3).await?;

                let response_letter_toggle = ElementAction::new(&driver, "[data-qa=\"vacancy-response-letter-toggle\"", &css_strategy);
                
                if ElementAction::try_exists(&response_letter_toggle, 3).await?{
                    //response_letter_toggle.safe_click().await?;
                    ElementAction::try_safe_click(&response_letter_toggle,3).await?;
                    let response_popup_letter_form = ElementAction::new(&driver, "[data-qa=\"vacancy-response-popup-form-letter-input\"]", &css_strategy);
                    if response_popup_letter_form.exists().await?{
                        //response_popup_letter_form.safe_click().await?;
                        ElementAction::try_safe_click(&response_popup_letter_form,3).await?;

                        response_popup_letter_form.send_keys("test".to_string()).await?;

                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                        let submit_button = ElementAction::new(&driver, "[data-qa=\"vacancy-response-submit-popup\"", &css_strategy);
                        if submit_button.exists().await?{
                            //submit_button.safe_click().await?;
                            ElementAction::try_safe_click(&submit_button,3).await?;
                        }

                    }
                }
                
                match button_href{
                    Some(href) => {
                        responded_buttons_set.insert(href.to_string());
                        println!("Added vacancy to responded set. Title:{}, href: {},", vacancy.get_title().await, href);
                    },
                    None => {}
                }
            }

            let current_url = driver.current_url().await?;

            if current_url.as_str() != target_url {
                if ElementAction::try_exists(&limit_check, 3).await?{
                    reached_limit = true;
                    break;
                }

                driver.goto(target_url).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                break;
            }
        }// !vacancies
        
        all_vacancies.clear();
        all_vacancies = driver.find_all(By::Css(vacancy_selector)).await?;

        if !all_vacancies.is_empty(){
            all_vacancies.iter().next_back().unwrap().wait_until().clickable().await?;
            vacancies_vec.clear();
            
            for vacancy_element in &all_vacancies{
                let mut vacancy = Vacancy::new(vacancy_element.clone());
                vacancy.update_vacancy_fields().await;
                
                let respond_button = vacancy.get_button().await;
                let title =vacancy.get_title().await;

                if respond_button.is_none() || title.is_empty(){
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

        if reached_limit{
            println!("You reached limit");
            break;
        }
    }// !loop

    CookieManager::save_cookies(cookie_json_path, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    driver.quit().await?;

    Ok(())
}