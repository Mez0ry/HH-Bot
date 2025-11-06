use std::{collections::HashSet};
use thirtyfour::{prelude::*};

pub mod cookie_manager;
pub mod vacancy;
pub mod element_action;
pub mod selector;
pub mod selector_manager;
pub mod page;

use crate::{cookie_manager::CookieManager, element_action::ElementAction, page::{Page, PageProcessState}, selector_manager::SelectorManager, vacancy::Vacancy};

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
    
    let driver = WebDriver::new("http://localhost:63768", caps).await?;

    let mut target_url : String = "https://hh.ru/search/vacancy?text=%D0%9F%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%81%D1%82+C%2B%2B&salary=&ored_clusters=true&enable_snippets=true&hhtmFrom=vacancy_search_list&hhtmFromLabel=vacancy_search_line".to_string();

    driver.goto(&target_url).await?;
    
    let cookie_json_path: String = String::from("./resources/cookies.json");

    if let Some(cookies) = CookieManager::load_cookies(&cookie_json_path, &driver).await?{
        for cookie in cookies {
            driver.add_cookie(cookie).await?;
        }
    }

    driver.refresh().await?;

    let _ = SelectorManager::load_selectors("./resources/selectors.json").await;

    let mut page = Page::new(target_url, &driver);
    
    match page.process_page().await{
        PageProcessState::GatheringVacancies =>{
            println!("failed on gathering vacancies state");
        }
    }
    
    // let mut responded_buttons_set: HashSet<String> = std::collections::HashSet::new();

    // let mut all_vacancies = driver.find_all(SelectorManager::find_selector("vacancy").await.get_by()).await?;

    // let mut vacancies_vec : Vec<Vacancy> = vec![];
    
    // for vacancy_element in &all_vacancies{
    //     let mut vacancy = Vacancy::new(vacancy_element.clone());
    //     vacancy.update_vacancy_fields().await;

    //     vacancies_vec.push(vacancy);
    // }

    // loop{
    //     let mut reached_limit = false;
        
    //     for vacancy in &vacancies_vec{
    //         println!("Handling vacancy: title: {}", vacancy.get_title().await);
    //         let respond_button = vacancy.get_button().await;
    //         let href = vacancy.get_href().await;

    //         if respond_button.is_none() || href.is_none() || href.clone().is_some_and(|actual_href|  responded_buttons_set.contains(&actual_href)) {
    //             responded_buttons_set.insert(href.unwrap().clone());
    //             continue;
    //         }

    //         let href = href.unwrap();

    //         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    //         if !vacancy.click_respond().await{
    //             responded_buttons_set.insert(href.clone());
    //             continue;
    //         }

    //         let limit_check = ElementAction::new(&driver, SelectorManager::find_selector("vacancy_limit_reached").await);
    //         if ElementAction::try_exists(&limit_check, 3).await{
    //             reached_limit = true;
    //             break;
    //         }

    //         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    //         let relocation_popup = ElementAction::new(&driver, SelectorManager::find_selector("relocation_warning_confirm").await);

    //         if ElementAction::try_exists(&relocation_popup, 3).await{
    //             ElementAction::try_safe_click(&relocation_popup,3).await;
    //         }

    //         let accept_cookies = ElementAction::new(&driver,SelectorManager::find_selector("accept_cookies").await);

    //         if ElementAction::try_exists(&accept_cookies, 4).await{
    //             ElementAction::try_safe_click(&accept_cookies, 3).await;
    //         }
            
    //         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    //         let submit_button = ElementAction::new(&driver, SelectorManager::find_selector("submit_button").await);
    //         if ElementAction::try_exists(&submit_button, 3).await{
                
    //             ElementAction::try_safe_click(&submit_button,3).await;

    //             let response_letter_toggle = ElementAction::new(&driver, SelectorManager::find_selector("response_letter_toggle").await);
                
    //             if ElementAction::try_exists(&response_letter_toggle, 3).await{

    //                 ElementAction::try_safe_click(&response_letter_toggle,3).await;

    //                 let response_letter_form_input = ElementAction::new(&driver, SelectorManager::find_selector("response_letter_form_input").await);
    //                 if response_letter_form_input.exists().await?{
    //                     ElementAction::try_safe_click(&response_letter_form_input,3).await;

    //                     response_letter_form_input.send_keys("test".to_string()).await?;

    //                     tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    //                     let submit_button = ElementAction::new(&driver, SelectorManager::find_selector("submit_button").await);
    //                     if submit_button.exists().await?{
    //                         ElementAction::try_safe_click(&submit_button,3).await;
    //                     }

    //                 }
    //             }

    //         }
            
    //         responded_buttons_set.insert(href.clone());
    //         println!("Handled Vacancy: Title: {}, href: {}", vacancy.get_title().await, &href);

    //         let current_url = driver.current_url().await?;

    //         if current_url.to_string() != target_url.clone() {
    //             if ElementAction::try_exists(&limit_check, 3).await{
    //                 reached_limit = true;
    //                 break;
    //             }

    //             driver.goto(&target_url).await?;
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //             break;
    //         }
    //     }// !vacancies
        
    //     all_vacancies.clear();
    //     all_vacancies = driver.find_all(SelectorManager::find_selector("vacancy").await.get_by()).await?;

    //     all_vacancies.iter().next_back().unwrap().wait_until().clickable().await?;

    //     if !all_vacancies.is_empty(){
    //         vacancies_vec.clear();
            
    //         let vacancies_on_page = 50 + 1;
    //         let elements_to_skip = responded_buttons_set.len() % vacancies_on_page;
                
    //         for vacancy_element in all_vacancies.iter().skip(elements_to_skip){
    //             let mut vacancy = Vacancy::new(vacancy_element.clone());
    //             vacancy.update_vacancy_fields().await;
                
    //             let respond_button = vacancy.get_button().await;
    //             let title =vacancy.get_title().await;
    //             let href = vacancy.get_href().await;

    //             if respond_button.is_none() || title.is_empty() || href.is_none() || href.is_some_and(|actual_href| responded_buttons_set.contains(&actual_href)){
    //                 eprintln!("Skipping vacancy with no button/title/href or already responded on it");
    //                 continue;
    //             }
                
    //             vacancies_vec.push(vacancy);
    //         }
    //     }

    //     if vacancies_vec.is_empty(){
    //         let page_next = ElementAction::new(&driver, SelectorManager::find_selector("next_page").await);
    //         if ElementAction::try_exists(&page_next, 3).await{
    //             ElementAction::try_safe_click(&page_next,3).await;
    //             tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    //             let url  = driver.current_url().await?;
    //             target_url = url.clone().to_string();
    //             driver.goto(&target_url).await?;
    //             tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    //             all_vacancies = driver.find_all(SelectorManager::find_selector("vacancy").await.get_by()).await?;

    //             if !all_vacancies.is_empty(){
    //                 vacancies_vec.clear();
                    
    //                 let vacancies_on_page = 49;
    //                 let elements_to_skip = responded_buttons_set.len() % vacancies_on_page;

    //                 for vacancy_element in all_vacancies.iter().skip(elements_to_skip){
    //                     vacancy_element.wait_until().clickable().await?;

    //                     let mut vacancy = Vacancy::new(vacancy_element.clone());
    //                     vacancy.update_vacancy_fields().await;
                        
    //                     let respond_button = vacancy.get_button().await;
    //                     let title = vacancy.get_title().await;
    //                     let href = vacancy.get_href().await;

    //                     if respond_button.is_none() || title.is_empty() || href.is_none() || href.is_some_and(|actual_href| responded_buttons_set.contains(&actual_href)){
    //                         eprintln!("Skipping vacancy with no button/title/href or already responded on it");
    //                         continue;
    //                     }
                        
    //                     vacancies_vec.push(vacancy);
    //                 }
    //             }
    //             responded_buttons_set.clear();
    //             println!("moving to page: target_url {}", &target_url);
    //         }
    //     }

    //     if reached_limit{
    //         println!("Reached the limit in 200 vacancies");
    //         break;
    //     }
    // }// !loop

    CookieManager::save_cookies(cookie_json_path, driver.get_all_cookies().await?).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    driver.quit().await?;

    Ok(())
}