use serenity::{ async_trait, prelude::*, model::channel::Message };
use crate::{ Handler, arg::Arg, cmd::Command };

pub struct Batch;

pub const BATCH_HELP: &'static str = "\
__SIGNATURE__
with ***<this>*** being **bat** or **batch**,
***<this> (<sep>+ <command> <args...>)\\**** — run all attached commands sequentially,\
    separated by ***<sep>***.
__CAVEATS__
- the separator needs to stay consistent throughout one usage of `batch`; the first occurrence is\
    taken to be the expected separator.
- it is best practice to put spaces before and after the separator.
- it is best to avoid using (parentheses), [brackets], {braces}, 'apostrophes', \"quotes\",\
    \\`ticks\\`, and the bar '|' within separators since they're used as string delimiters, and\
    commands cannot see a string's delimiters.
- unquoted newlines may not be used as separators, since they are not visible to commands.
__ADDED__ 2024-04-20
__UPDATED__ 2024-05-11
__SEE ALSO__
- An **arguments** help-page is on the to-dos for this bot.
";

#[async_trait]
impl Command for Batch {
    async fn prefixes(&self) -> &'static [&'static str] { &["batch", "bat"] }
    async fn help(&self, _: &[Arg<'_>]) -> String { BATCH_HELP.to_owned() }
    async fn run(&self, h: &Handler, c: &Context, m: &Message, a: &[Arg<'_>]) {
        let a = &a[1..];
        if a.len() == 0 {
            return h.try_say(c, &m.channel_id, "(**batch** | no arguments, not even a separator)").await
        }
        let sep = &a[0];
        if let Arg::Pos(_) = sep {} else {
            let reply = format!(
                "```\nExpected positional-argument separator; found {} ({:?})\n```", sep, sep);
            h.try_say(c, &m.channel_id, reply).await;
            return
        }
        let mut j = 1;
        let mut acc = vec![];
        for i in 1..=a.len() {
            if i == a.len() || &a[i] == sep {
                if j + 1 < i {
                    acc.push(&a[j..i]);
                }
                j = i + 1;
            }
        }
        if acc.len() == 0 {
            return h.try_say(c, &m.channel_id, "(**batch** | no commands, only separators)").await
        }
        for a_ in acc.into_iter() {
            h.run_command(c, m, a_).await
        }
    }
}
