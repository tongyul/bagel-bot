use serenity::{ async_trait, prelude::*, model::channel::Message };
use crate::{ Handler, arg::Arg, cmd::Command };

pub struct Help;

const HELP_HELP: &'static str = "\
__SIGNATURE__
with ***<this>*** being either **help** or just **h**,
- ***<this>*** — print general help about the bot.
- ***<this> <thing> <args...>*** — print help associatd with ***<thing> <args...>***, where
    ***<thing>*** can be a command or a topic.
__ADDED__ 2024-05-11
__UPDATED__ (initial version)
";

#[async_trait]
impl Command for Help {
    async fn prefixes(&self) -> &'static [&'static str] { &["h", "help"] }
    async fn help(&self, _: &[Arg<'_>]) -> String { HELP_HELP.to_owned() }
    async fn run(&self, h: &Handler, c: &Context, m: &Message, a: &[Arg<'_>]) {
        let a = &a[1..];
        if a.len() == 0 {
            let mut keys = h.command_table.read()
                .map(|ct| ct.keys().map(|s| s.to_owned()).collect::<Vec<_>>())
                .expect("(Help) poisoned command table on read()");
            keys.sort();
            let mut reply = String::new();
            reply.push_str("__GENERAL SYNTAX__ ");
            reply.push_str(h.default_prefix);
            reply.push_str(" ***<command> <args...>***\n");
            reply.push_str("__FOR HELP__ … (**h** or **help**) ***<command or topic> <...>***\n");
            reply.push_str("__AVAILABLE TOPICS AND COMMANDS__\n");
            for k in keys {
                reply.push_str("- **");
                reply.push_str(k);
                reply.push_str("**\n");
            }
            h.try_say(c, &m.channel_id, reply).await
        } else {
            let name = format!("{}", &a[0]);
            let cmd = h.command_table.read()
                .map_err(|_| panic!("(Help) poisoned command table on read()"))
                .ok().and_then(|ct| ct.get(&name[..]).map(|c| c.clone()));
            match cmd {
                Some(cm) => {
                    let reply = cm.help(a).await;
                    h.try_say(c, &m.channel_id, reply).await
                }
                None => h.try_say(c, &m.channel_id, format!("Didn't find topic \"{}\"", &a[0])).await,
            }
        }
    }
}
