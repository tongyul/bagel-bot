// Modules
mod arg;
mod batch;
mod cmd;
mod echo;
mod help;
mod xiv;

// Namespace imports
use std::{ collections::HashMap, env, sync::{ RwLock, Arc } };

use serenity::async_trait;
use serenity::model::{
    channel::Message,
    id::ChannelId,
    gateway::Ready,
};
use serenity::prelude::*;

use async_recursion::async_recursion;

use crate::{
    arg::Arg,
    cmd::Command,
};

// Main code of the bot
struct Handler {
    default_prefix: &'static str,

    default_prefix_args: &'static [Arg<'static>],
    command_table: RwLock<HashMap<&'static str, Arc<dyn Command>>>,
}

impl Handler {
    async fn new(default_prefix: &'static str) -> Self {
        let default_prefix_args =
            arg::parse(default_prefix)
            .expect("DEFAULT_PREFIX should be a valid sequence of arguments")
            .leak::<'static>();

        Self {
            default_prefix,
            default_prefix_args,

            command_table: RwLock::new(HashMap::new()),
        }
    }
    async fn register(&self, c: Arc<dyn Command>) {
        let mut wlock = self.command_table.write()
            .expect("(Handler) poisoned command table on write()");
        for n in c.prefixes().await {
            if let Some(_) = wlock.insert(n, c.clone()) {
                panic!("(Handler) duplicate command name {:?}", n);
            }
        }
    }
    async fn try_say(&self, cx: impl std::fmt::Debug + CacheHttp, chan: &ChannelId, txt: impl Into<String>) {
        if let Err(why) = chan.say(cx.http(), txt).await {
            eprintln!("Error sending message: {:?}\nContext: {:?}\nChannel: {:?}", why, cx, chan);
        }
    }
    #[async_recursion]
    async fn run_command(&self, ctx: &Context, msg: &Message, args: &[Arg<'_>]) {
        // some built-in commands
        if args.len() == 0 {
            let reply = format!("Hi! Try running `{} help`!!", self.default_prefix);
            self.try_say(ctx, &msg.channel_id, reply).await;
        } else {
            let name = format!("{}", args[0]);
            let cmd = self.command_table.read()
                .map_err(|_| panic!("(Handler) poisoned command table on read()"))
                .ok().and_then(|ct| ct.get(&name[..]).map(|c| c.clone()));
            match cmd {
                Some(c) => c.run(self, ctx, msg, args).await,
                None => {
                    let reply = format!("The command {} doesn't exist.", name);
                    self.try_say(ctx, &msg.channel_id, reply).await
                }
            }
        }
    }
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
            match arg::parse(&msg.content) {
                Err(s) => {
                    eprintln!("Argument parsing error: {}", s);
                    let reply = format!("```\n{}\n```", s);
                    self.try_say(&ctx, &msg.channel_id, reply).await;
                }
                Ok(args) => {
                    let prefix = self.default_prefix_args;
                    if args.len() < prefix.len() {
                        panic!("Less actual arguments than there were in prefix. How did this happen!?");
                    }
                    if prefix.iter().zip(args.iter()).any(|(a, b)| a != b) {
                        let reply = format!(
                            "```\nThe default prefix ({:?}) is somehow messed up; did you forget to add a space?\n```",
                            self.default_prefix,
                        );
                        self.try_say(&ctx, &msg.channel_id, reply).await;
                    } else {
                        self.run_command(&ctx, &msg, &args[prefix.len()..]).await;
                    }
                }
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
    let handler = Handler::new(default_prefix).await;
    handler.register(Arc::new(batch::Batch)).await;
    handler.register(Arc::new(echo::Echo)).await;
    handler.register(Arc::new(help::Help)).await;
    handler.register(Arc::new(xiv::Archive)).await;
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
