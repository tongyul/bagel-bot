mod arg;

use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    default_prefix: &'static str,
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content[..].starts_with(self.default_prefix) && msg.webhook_id.is_none() {
            eprintln!("Received non-webhook command (default prefix)\n{:?}", msg.content);
            let reply = match arg::parse(&msg.content[self.default_prefix.len()..]) {
                Ok(args) => format!("```\n{:?}\n```", args),
                Err(s) => format!("```\n{}\n```", s),
            };
            eprintln!("Reply: {}", reply);
            if let Err(why) = msg.channel_id.say(&ctx.http, &reply).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Retrieve the default prefix
    let default_prefix =
        env::var("DEFAULT_PREFIX")
        .expect("Expected DEFAULT_PREFIX in environ")
        .leak::<'static>();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let handler = Handler { default_prefix };
    let mut client =
        Client::builder(&token, intents).event_handler(handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
