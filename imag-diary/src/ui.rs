use clap::{Arg, ArgGroup, App, SubCommand};

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
       .arg(Arg::with_name("diaryname")
            .long("diary")
            .short("d")
            .takes_value(true)
            .required(false)
            .help("Use other than default diary"))

        .subcommand(SubCommand::with_name("create")
                   .about("Create a diary entry")
                   .version("0.1")
                   .arg(Arg::with_name("no-edit")
                        .long("no-edit")
                        .short("e")
                        .takes_value(false)
                        .required(false)
                        .help("Do not edit after creating"))

                   .arg(Arg::with_name("timed")
                        .long("timed")
                        .short("t")
                        .takes_value(true)
                        .required(false)
                        .help("By default, one entry is created per day. With --timed=h[ourly] or
                        --timed=m[inutely] one can create per-hour and per-minute entries (more like
                        a microblog then"))

                   .arg(Arg::with_name("hour")
                        .long("hour")
                        .takes_value(true)
                        .required(false)
                        .help("When using --timed, override the hour component"))
                   .arg(Arg::with_name("minute")
                        .long("minute")
                        .takes_value(true)
                        .required(false)
                        .help("When using --timed, override the minute component"))

                   // When using --hour or --minute, --timed must be present
                   .group(ArgGroup::with_name("timing-hourly")
                            .args(&["hour"])
                            .requires("timed"))
                   .group(ArgGroup::with_name("timing-minutely")
                            .args(&["minute"])
                            .requires("timed"))
                   )

        .subcommand(SubCommand::with_name("edit")
                   .about("Edit a diary entry")
                   .version("0.1")
                   .arg(Arg::with_name("datetime")
                        .long("datetime")
                        .short("d")
                        .takes_value(true)
                        .required(false)
                        .help("Specify the date and time which entry should be edited. If none is
                        specified, the last entry is edited. If the diary entry does not exist for
                        this time, this fails. Format: YYYY-MM-DDT[HH[:mm[:ss]]]"))
                   )

        .subcommand(SubCommand::with_name("list")
                   .about("List diary entries")
                   .version("0.1"))

        .subcommand(SubCommand::with_name("delete")
                   .about("Delete a diary entry")
                   .version("0.1")
                   .arg(Arg::with_name("datetime")
                        .long("datetime")
                        .short("d")
                        .takes_value(true)
                        .required(false)
                        .help("Specify the date and time which entry should be deleted. If none is
                        specified, the last entry is deleted. If the diary entry does not exist for
                        this time, this fails. Format: YYYY-MM-DDT[HH[:mm[:ss]]]"))

                   .arg(Arg::with_name("select")
                        .long("select")
                        .short("s")
                        .takes_value(false)
                        .required(false)
                        .help("Use interactive selection"))

                   .arg(Arg::with_name("yes")
                        .long("yes")
                        .short("y")
                        .takes_value(false)
                        .required(false)
                        .help("Do not ask for confirmation."))
                )
}

