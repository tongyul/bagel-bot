use serenity::{ async_trait, prelude::*, model::channel::Message };
use crate::{ Handler, arg::Arg, cmd::Command };

pub struct Echo;

const ECHO_HELP: &'static str = "\
__SIGNATURE__
***<this> <args...>***
where ***<this>*** may be
- **echo** — print arguments, space-separated.
- **join** — print arguments with no separator.
- **printargs**, **print_args**, or **printArgs** — pretty print arguments, for debugging and\
    learning purposes.
__ADDED__ 2024-04-20
__UPDATED__ 2024-05-11
__SEE ALSO__
- An **arguments** help-page is on the to-dos for this bot.
";

#[async_trait]
impl Command for Echo {
    async fn prefixes(&self) -> &'static [&'static str] {
        &["echo", "join", "printargs", "print_args", "printArgs"]
    }
    async fn help(&self, _: &[Arg<'_>]) -> String { ECHO_HELP.to_owned() }
    async fn run(&self, h: &Handler, c: &Context, m: &Message, a: &[Arg<'_>]) {
        assert!(a.len() >= 1, "(Echo) called on no arguments; how is this possible?");
        match &a[0] {
            Arg::Pos(name) if self.prefixes().await.iter().any(|p| name == p) => {
                let name = *name;
                let a = &a[1..];
                let reply = if a.len() == 0 {
                    "(empty)".to_owned()
                } else if name == "echo" || name == "join" {
                    let is_echo = name == "echo";
                    a.iter().fold(String::new(), |mut acc, x| {
                        if is_echo { acc.push(' '); }
                        acc.push_str(&format!("{}", x));
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
                    a.iter().enumerate()
                        .fold(String::new(), |mut acc, (i, x)| {
                            acc.push_str(&format!("{}. ", i + 1));
                            acc.push_str(&__custom_format(x));
                            acc.push('\n');
                            acc
                        })
                };
                h.try_say(c, &m.channel_id, reply).await
            }
            _ => panic!("(Echo) called on wrong arguments; how is this possible?"),
        }
    }
}
