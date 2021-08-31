use actix_web::middleware;
use actix_web::{web, App, HttpServer};
use actix_web::client::Client;
use actix_web::http::StatusCode;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

//some basic structs for deserializing messages from etherscan
#[derive(Serialize, Deserialize, Debug)]
struct JsonResponseBlock {
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonBlockDetails {
    result: InnerResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct InnerResult {
    timeStamp: String,
}

//actix-web main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //this will open http port 80 on all interfaces
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            //expose only this route 
            .route("/currentBlockTime", web::get().to(get_last_block_timestamp))
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}

//
pub async fn get_last_block_timestamp(_req: HttpRequest) -> HttpResponse {
    //create a client request
    let  client = Client::default();

    println!("got connection");

    //get last block using api
    let response1 = client
        .get("http://api.etherscan.io/api?module=proxy&action=eth_blockNumber")
        .header("User-Agent", "actix-web/3.0")
        .send()
        .await;

    //handle response data
    match response1 {
        Ok(mut data) => {
            //these are a little bit shady using unwraps, but this is just proof of concept, 
            let body = data.body().await.unwrap();
            let utf8 = std::str::from_utf8(body.as_ref()).unwrap();
            let json_data: JsonResponseBlock = serde_json::from_str(&utf8).unwrap();

            println!("last block is: \"{}\"", json_data.result);

            //parse to a valid usize
            let parsed = usize::from_str_radix(json_data.result.trim_start_matches("0x"), 16)
                .expect("unable to parse!");

            println!("last block in decimal is: \"{}\"", parsed);

            //format the url for the timestamp request
            let mut formatted_url = String::from(
                "http://api.etherscan.io/api?module=block&action=getblockreward&blockno=",
            );
            formatted_url.push_str(&parsed.to_string());

            println!("waiting 7s because we don't have an API key");
            std::thread::sleep(std::time::Duration::from_millis(7000));

            //make another request to get the timestamp
            let response2 = client
                .get(&formatted_url)
                .header("User-Agent", "actix-web/3.0")
                .send()
                .await;


            match response2 {
                Ok(mut data) => {
                    let body = data.body().await.unwrap();
                    let utf8 = std::str::from_utf8(body.as_ref()).unwrap();

                     // println!("Response: {:?}", utf8);

                    let json_data: JsonBlockDetails = serde_json::from_str(&utf8).unwrap();
                    let last_block_timestamp = json_data.result.timeStamp;

                    println!("timestamp for last block: \"{}\"", last_block_timestamp);

                    //create a json response, we could use serde but this is just so simple
                    let formatted_response = format!("{{\"last_block:\":{}, \"timestamp\": {}}}", parsed, last_block_timestamp);


                    //return a response to the client
                    return HttpResponse::build(StatusCode::OK)
                    .set_header("Content-Type", "application/json")
                    .body(&formatted_response);
                }
                Err(e) => {
                    println!("{}", e);
                    return HttpResponse::build(StatusCode::OK)
                        //.set_header("Location", "app")
                        .finish();
                }
            }
        }
        Err(e) => {
            println!("{}", e);
            return HttpResponse::build(StatusCode::OK)
                //.set_header("Location", "app")
                .finish();
        }
    }
   
}
