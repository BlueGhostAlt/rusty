use std::env;

use rusty::eval;

use dotenv;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id.as_u64() != &231500544533856256 {
            return;
        }

        let code = rusty::extract_code_from_message(&msg.content);

        match code {
            Some(code) => {
                let output = eval::execute_code(code).map_or_else(|e| format!("{:?}", e), |o| o);

                if let Err(reason) = msg.channel_id.say(&ctx.http, output).await {
                    println!("An error occured while sending the message: {:?}", reason);
                }
            }
            None => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is up and running!", ready.user.tag());
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("TOKEN").expect("No Discord token has been provided!");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("An error occured while creating the client!");

    if let Err(err) = client.start().await {
        println!("The client encountered an error: {:?}", err);
    }
}
