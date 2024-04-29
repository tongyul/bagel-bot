mod arg;
mod help_msgs;

use std::env;

use serenity::async_trait;
use serenity::model::{
    channel::Message,
    id::ChannelId,
    gateway::Ready,
};
use serenity::prelude::*;

use async_recursion::async_recursion;

use arg::Arg;
use help_msgs::*;

struct Handler {
    default_prefix: &'static str,
    default_prefix_args: &'static [Arg<'static>],
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
            self.try_say(ctx, &msg.channel_id, "Haiii!!!!").await;
        } else {
            let cmd = &args[0];
            let args = &args[1..];
            match cmd {
                Arg::Pos(cmdname @ ("echo" | "printargs" | "print_args" | "join")) => {
                    let reply = if args.len() == 0 {
                        "(empty)".to_owned()
                    } else if *cmdname == "echo" || *cmdname == "join" {
                        args.iter().fold(String::new(), |mut acc, arg| {
                            if *cmdname == "echo" {
                                acc.push(' ');
                            }
                            acc.push_str(&format!("{}", arg));
                            acc
                        })
                    } else {
                        fn __custom_format(a: &Arg<'_>) -> String {
                            match a {
                                Arg::Pos(v) => format!("positional argument `{}`", v),
                                Arg::Kw(k, v) => format!("keyword argument {}=`{}`", k, v),
                                Arg::Flag(on, k) => format!("flag ({}) {}", if *on {"on"} else {"off"}, k),
                            }
                        }
                        args.iter().enumerate()
                            .fold(String::new(), |mut acc, (i, a)| {
                                acc.push_str(&format!("{}. ", i + 1));
                                acc.push_str(&__custom_format(a));
                                acc.push('\n');
                                acc
                            })
                    };
                    self.try_say(ctx, &msg.channel_id, reply).await;
                }
                Arg::Pos("batch") => self.the_batch_command(ctx, msg, args).await,
                _ => {
                    self.try_say(
                        ctx, &msg.channel_id,
                        format!("```\nUnknown command {:?}.\n```", cmd),
                    ).await;
                }
            }
        }
    }
    async fn the_batch_command(&self, ctx: &Context, msg: &Message, args: &[Arg<'_>]) {
        if args.len() == 0 {
            self.try_say(ctx, &msg.channel_id, THE_BATCH_COMMAND_HELP).await;
            return
        }
        let sep = &args[0];
        if let Arg::Pos(_) = sep {} else {
            let reply = format!(
                "```\nExpected positional-argument separator; found {} ({:?})\n```", sep, sep);
            self.try_say(ctx, &msg.channel_id, reply).await;
            return
        }
        let mut j = 1;
        let mut acc = vec![];
        for i in 1..=args.len() {
            if i == args.len() || &args[i] == sep {
                if j + 1 < i {
                    acc.push(&args[j..i]);
                }
                j = i + 1;
            }
        }
        for args in acc.into_iter() {
            self.run_command(ctx, msg, args).await
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
            // let reply = match arg::parse(&msg.content[self.default_prefix.len()..]) {
            //     Ok(args) => format!("```\n{:?}\n```", args),
            //     Err(s) => format!("```\n{}\n```", s),
            // };
            // eprintln!("Reply: {}", reply);
            // if let Err(why) = msg.channel_id.say(&ctx.http, &reply).await {
            //     eprintln!("Error sending message: {:?}", why);
            // }
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
