use actix_web::{error, http::StatusCode, test, test::call_service, web, App, HttpResponse};
use actix_web_4_validator::{Json, JsonConfig};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, PartialEq, Validate, Serialize, Deserialize)]
struct JsonPayload {
    #[validate(url)]
    page_url: String,
    #[validate(range(min = 18, max = 28))]
    age: u8,
}

async fn test_handler(query: Json<JsonPayload>) -> HttpResponse {
    dbg!(&query.into_inner());
    HttpResponse::Ok().finish()
}

#[actix_rt::test]
async fn test_json_validation() {
    let mut app = test::init_service(
        App::new().service(web::resource("/test").route(web::post().to(test_handler))),
    )
    .await;

    // Test 200 status
    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "https://my_page.com".to_owned(),
            age: 24,
        })
        .to_request();
    let resp = call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Test 400 status
    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "invalid_url".to_owned(),
            age: 24,
        })
        .to_request();
    let resp = call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_custom_json_validation_error() {
    let json_cfg = web::JsonConfig::default()
        // limit request payload size
        .limit(4096)
        // only accept text/plain content type
        .content_type(|mime| mime == mime::TEXT_PLAIN)
        // use custom error handler
        .error_handler(|err, _| {
            error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });

    let mut app = test::init_service(
        App::new().service(
            web::resource("/test")
                .app_data(json_cfg)
                .route(web::post().to(test_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "invalid".to_owned(),
            age: 24,
        })
        .to_request();
    let resp = call_service(&mut app, req).await;
    dbg!(&resp);
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

async fn test_validated_json_asref_deref_test(payload: Json<JsonPayload>) -> HttpResponse {
    assert_eq!(payload.age, 24);
    let reference = JsonPayload {
        page_url: "https://my_page.com".to_owned(),
        age: 24,
    };
    assert_eq!(payload.as_ref(), &reference);
    HttpResponse::Ok().finish()
}

#[actix_rt::test]
async fn test_validated_json_asref_deref() {
    let mut app = test::init_service(
        App::new().service(web::resource("/test").to(test_validated_json_asref_deref_test)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "https://my_page.com".to_owned(),
            age: 24,
        })
        .to_request();
    call_service(&mut app, req).await;
}

async fn test_validated_json_into_inner_test(payload: Json<JsonPayload>) -> HttpResponse {
    let payload = payload.into_inner();
    assert_eq!(payload.age, 24);
    assert_eq!(payload.page_url, "https://my_page.com");
    HttpResponse::Ok().finish()
}

#[actix_rt::test]
async fn test_validated_json_into_inner() {
    let mut app = test::init_service(
        App::new().service(web::resource("/test").to(test_validated_json_into_inner_test)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "https://my_page.com".to_owned(),
            age: 24,
        })
        .to_request();
    call_service(&mut app, req).await;
}

#[actix_rt::test]
async fn test_validated_json_limit() {
    let mut app = test::init_service(
        App::new()
            .app_data(JsonConfig::default().limit(1))
            .service(web::resource("/test").route(web::post().to(test_handler))),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&JsonPayload {
            page_url: "https://my_page.com".to_owned(),
            age: 24,
        })
        .to_request();
    let resp = call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
