use serenity::{ async_trait, prelude::*, model::channel::Message };
use crate::{ Handler, arg::Arg, cmd::Command };

pub struct Archive;

#[async_trait]
impl Command for Archive {
    async fn prefixes(&self) -> &'static [&'static str] {
        &[
            "xiv", "archive", "transcribe", "del", "delete",
            "ss", "setstart", "set_start", "setStart",
            "se", "setend", "set_end", "setEnd",
        ]
    }
    async fn help(&self, _: &[Arg<'_>]) -> String {
        "The archiver is yet to be implemented.".to_owned()
    }
    async fn run(&self, h: &Handler, c: &Context, m: &Message, _: &[Arg<'_>]) {
        let reply = "The archiver is yet to be implemented.";
        h.try_say(c, &m.channel_id, reply).await
    }
}
