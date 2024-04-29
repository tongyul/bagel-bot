pub const THE_BATCH_COMMAND_HELP: &'static str = "\
**Aliases** batch
**Usages**
- run mutiple commands separated by <sep>: batch (<sep> <command>...)+
- print this help message: batch
**Caveats**
- the separator needs to stay consistent throughout one usage of `batch`; the first occurrence is\
    taken to be the expected separator.
- it is best practice to put spaces before and after the separator
- it is best to avoid using (parentheses), [brackets], {braces}, 'apostrophes', \"quotes\",\
    \\`ticks\\`, and the bar '|' within separators since they're used as string delimiters, and\
    commands cannot see a string's delimiters.
- unquoted newlines may not be used as separators, since they are not visible to commands.
**Added** 2024-04-20
**Updated** (initial version)
";
