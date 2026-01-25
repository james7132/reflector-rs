use arch_mirrors_rs::Status;

const URL: &str = "https://archlinux.org/mirrors/status/json/";

// Fetch the latest mirrors and ensure that it deserializes correctly
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn default_mirrors_fetch_test() {
    let response: Status = reqwest::get(URL).await.unwrap().json().await.unwrap();
    assert!(!response.urls.is_empty());
}
