use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use crate::Handler;
use crate::arg::Arg;

#[async_trait]
pub trait Command: Sync + Send {
    async fn prefixes(&self) -> &'static [&'static str];
    async fn help(&self, _: &[Arg<'_>]) -> String;
    async fn run(&self, _: &Handler, _: &Context, _: &Message, _: &[Arg<'_>]);
}
