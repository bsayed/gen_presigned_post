use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Client;
use reqwest::Error;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;

fn generate_basic_auth(username: &str, password: &str) -> String {
    let auth_str = format!("{}:{}", username, password);
    let auth_encoded = general_purpose::STANDARD.encode(auth_str);
    format!("Basic {}", auth_encoded)
}

async fn send_post_request() -> Result<(), Error> {
    let content: Vec<u8> = tokio::fs::read("./images/face.png").await.unwrap();

    // Assuming `content` is your Vec<u8>
    let mut hasher = Sha256::new();
    hasher.update(&content);

    let result = hasher.finalize();
    let encoded = general_purpose::STANDARD.encode(result);
    let client = Client::new();

    let signup_id = "e2e-signup-bassam--2";

    let orb_id = env::var("ORBID").unwrap();
    let password = env::var("PASSWORD").unwrap();
    let service_url = env::var("SERVICE_URL").unwrap();

    println!("Username: {}", orb_id);
    println!("Password: {}", password);
    println!("Service URL: {}", service_url);

    let auth_token = generate_basic_auth(&orb_id, &password);
    println!("Auth Token: {}", auth_token);

    let get_resp = client
        .post(format!(
            "{}/api/v2/signups/{}/package",
            service_url, signup_id
        ))
        .header("Authorization", auth_token)
        .json(&serde_json::json!({
            "orbId": orb_id,
            "sessionId": "test-session-id",
            "checksum":encoded,
        }))
        .send()
        .await?;
    println!("Get Presigned POST Status: {}", get_resp.status());
    let body = get_resp.text().await?;
    println!("Presigned POST Body: {}\n", body);

    // Parse the JSON body
    let v: Value = serde_json::from_str(&body).unwrap();

    let fields = v.get("fields").unwrap();
    let url = v.get("url").unwrap().as_str().unwrap();

    // Convert the JSON Value of the fields field into a HashMap
    let map: HashMap<String, String> = fields
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_owned()))
        .collect();

    let mut form = Form::new();

    for (key, value) in map {
        form = form.text(key, value);
    }
    // .text("x-amz-checksum-algorithm", "SHA256")
    // .text("x-amz-checksum-sha256", "5nseLI+wdPxg/RKzc3Y8Qw4lPTzsDH8+5P2Y+Enwmps=")
    // .text("key", "e2e-orb-1/e2e-signup-bassam--2/biometrics")
    // .text("x-amz-algorithm", "AWS4-HMAC-SHA256")
    // .text("x-amz-credential", "ASIATYJV6Z4VYNWQBF3A/20240402/eu-central-1/s3/aws4_request")
    // .text("x-amz-date", "20240402T151434Z")
    // .text("x-amz-security-token", "IQoJb3JpZ2luX2VjEDUaCXVzLWVhc3QtMSJIMEYCIQClo4DFFDp4MxTqz461d0i6Uq1sWIFpQp/M9NAvacrWQAIhALmNIchlfv4GElDQqj1MAHuNDLZPBAO3tjLxlz+lteeuKrgDCF4QAhoMMjU4MzQ4MDc2ODQzIgzaBd5rd01DkfNPWwAqlQNVcDHo9pS6tDkgWHHq2fDtjdlpjRzOmrze8MLckYpmh5OEFHvtSfdA5XPlV8WkOx5ODGSkQwtzUJW3NoItJ2lYmjFyEwlaAL21oVxEG2eI52i0p6oX0EIxoojYgrdYo6hFDuk6H3XD1QbGNTZLgNgmq/1fmMoaOoM75t4D4uNLW8uisGt/xkX9dRONleDdIay+wsXES1tONvZjRMlzrmjWQxQAGNX56O5AINQmJus/hHbl40+oOKVCHNxZXneMOSrLOE46TyvYy3qCljyV4s3yA78ilgtHrIsHRtnTuzpH+IFNzpX1WV7cj6Ct/Gm+q9KwZfWm2B1tdws8hrxtI9fWlH7oO4CbIxJffpE/07dI9H0W+XvvFr3+GEJFjiSPY3Ws28DQirNyUt4gPCTTI8mHL4k9rRipOrzY73nG+HAayiHGSGgN0NN2SQrTd/Kp/rWX4EvEsdk3FlLCrWkorxVvFU21yqxY6KIMobDpyt97W4tCwCQdtDd2M5M4CQR9/yZLtPtAL200rvMtzIs6WdGSz/+OHE4wwvmvsAY6pQECnysb2YB3oZDh3TLKs8/5i65M31oFnwFvOhLsfvKbu18tUQEjldhzaS0T8RUzzfjkvIkJBzUOxl+7cctGIpPL3s6n9RXU8mY6sZIh3OtGyQrMF1pc8CiywwNG+EydyZnBHnBDvrvF0E2zd211mO2AQy8nt3Vob+1CojlFUybNZDU8Pi1X2nxkwLSwPQDynKFAo4y0X51MStJy7cCAPNqFwXH1ntA=")
    // .text("policy", "eyJjb25kaXRpb25zIjpbeyJidWNrZXQiOiJiYXNzYW0tdGZoLXRlc3QifSxbInN0YXJ0cy13aXRoIiwiJGtleSIsImUyZS1vcmItMS9lMmUtc2lnbnVwLWJhc3NhbS0tMi9iaW9tZXRyaWNzIl0seyJ4LWFtei1jcmVkZW50aWFsIjoiQVNJQVRZSlY2WjRWWU5XUUJGM0EvMjAyNDA0MDIvZXUtY2VudHJhbC0xL3MzL2F3czRfcmVxdWVzdCJ9LHsieC1hbXotc2VjdXJpdHktdG9rZW4iOiJJUW9KYjNKcFoybHVYMlZqRURVYUNYVnpMV1ZoYzNRdE1TSklNRVlDSVFDbG80REZGRHA0TXhUcXo0NjFkMGk2VXExc1dJRnBRcC9NOU5BdmFjcldRQUloQUxtTkljaGxmdjRHRWxEUXFqMU1BSHVORExaUEJBTzN0akx4bHorbHRlZXVLcmdEQ0Y0UUFob01NalU0TXpRNE1EYzJPRFF6SWd6YUJkNXJkMDFEa2ZOUFd3QXFsUU5WY0RIbzlwUzZ0RGtnV0hIcTJmRHRqZGxwalJ6T21yemU4TUxja1lwbWg1T0VGSHZ0U2ZkQTVYUGxWOFdrT3g1T0RHU2tRd3R6VUpXM05vSXRKMmxZbWpGeUV3bGFBTDIxb1Z4RUcyZUk1MmkwcDZvWDBFSXhvb2pZZ3JkWW82aEZEdWs2SDNYRDFRYkdOVFpMZ05nbXEvMWZtTW9hT29NNzV0NEQ0dU5MVzh1aXNHdC94a1g5ZFJPTmxlRGRJYXkrd3NYRVMxdE9OdlpqUk1senJtaldReFFBR05YNTZPNUFJTlFtSnVzL2hIYmw0MCtvT0tWQ0hOeFpYbmVNT1NyTE9FNDZUeXZZeTNxQ2xqeVY0czN5QTc4aWxndEhySXNIUnRuVHV6cEgrSUZOenBYMVdWN2NqNkN0L0dtK3E5S3daZldtMkIxdGR3czhocnh0STlmV2xIN29PNENiSXhKZmZwRS8wN2RJOUgwVytYdnZGcjMrR0VKRmppU1BZM1dzMjhEUWlyTnlVdDRnUENUVEk4bUhMNGs5clJpcE9yelk3M25HK0hBYXlpSEdTR2dOME5OMlNRclRkL0twL3JXWDRFdkVzZGszRmxMQ3JXa29yeFZ2RlUyMXlxeFk2S0lNb2JEcHl0OTdXNHRDd0NRZHREZDJNNU00Q1FSOS95Wkx0UHRBTDIwMHJ2TXR6SXM2V2RHU3ovK09IRTR3d3ZtdnNBWTZwUUVDbnlzYjJZQjNvWkRoM1RMS3M4LzVpNjVNMzFvRm53RnZPaExzZnZLYnUxOHRVUUVqbGRoemFTMFQ4UlV6emZqa3ZJa0pCelVPeGwrN2NjdEdJcFBMM3M2bjlSWFU4bVk2c1pJaDNPdEd5UXJNRjFwYzhDaXl3d05HK0V5ZHlabkJIbkJEdnJ2RjBFMnpkMjExbU8yQVF5OG50M1ZvYisxQ29qbEZVeWJOWkRVOFBpMVgybnhrd0xTd1BRRHluS0ZBbzR5MFg1MU1TdEp5N2NDQVBOcUZ3WEgxbnRBPSJ9LHsieC1hbXotYWxnb3JpdGhtIjoiQVdTNC1ITUFDLVNIQTI1NiJ9LHsieC1hbXotZGF0ZSI6IjIwMjQwNDAyVDE1MTQzNFoifSxbIngtYW16LWNoZWNrc3VtLWFsZ29yaXRobSIsIlNIQTI1NiJdLFsieC1hbXotY2hlY2tzdW0tc2hhMjU2IiwiNW5zZUxJK3dkUHhnL1JLemMzWThRdzRsUFR6c0RIOCs1UDJZK0Vud21wcz0iXV0sImV4cGlyYXRpb24iOiIyMDI0LTA0LTAyVDE1OjE0OjM0LjY3MloifQ")
    // .text("x-amz-signature", "506230d858ca6345493806971d7c31c6cdc98345db6b9b6943676a86018cb100")

    let bytes_part = Part::bytes(content)
        .file_name("biometrics.zip")
        .mime_str("application/octet-stream") // or "application/octet-stream"
        .unwrap();

    form = form.part("file", bytes_part);

    let res = client
        // .post("https://s3.eu-central-1.amazonaws.com/bassam-tfh-test")
        .post(url)
        // .post("http://localhost:10001")
        .multipart(form)
        .send()
        .await?;

    println!("Status: {}", res.status());
    assert!(res.status() == 204); // 204 No Content means the upload was successful and the file is now in the bucket

    Ok(())
}

#[tokio::main]
async fn main() {
    send_post_request().await.unwrap();
}
