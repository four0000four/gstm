use chrono::DateTime;
use clap::{crate_authors, crate_version, App, Arg, SubCommand};

use log;

use gstm;

#[tokio::main]
async fn main() {
    let matches = App::new("gstm")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Gist manipulator")
        .subcommands(vec![
            SubCommand::with_name("create")
                .about("Create a new Gist")
                .arg(Arg::with_name("files").multiple(true).required(true))
                .arg(
                    Arg::with_name("private")
                        .short("-p")
                        .long("--private")
                        .help("Make your new Gist private"),
                )
                .arg(
                    Arg::with_name("description")
                        .short("-d")
                        .long("--description")
                        .takes_value(true)
                        .help("The description of your Gist"),
                ),
            SubCommand::with_name("list")
                .about("Retrieve a listing of Gists")
                .arg(
                    Arg::with_name("user")
                        .short("-u")
                        .long("--user")
                        .takes_value(true)
                        .help("Filter by username"),
                )
                .arg(
                    Arg::with_name("since")
                        .short("-s")
                        .long("--since")
                        .takes_value(true)
                        .help("Limit to Gists uploaded after an RFC 3339 (ISO 8601) timestamp (YYYY-MM-DDTHH:MM:SSZ)"),
                ),
                // .arg( TODO implement pagination
                //     Arg::with_name("count")
                //         .short("-c")
                //         .long("--count")
                //         .takes_value(true)
                //         .default_value("3000")
                //         .help("Retrieve [count] many values."),
                // )
            SubCommand::with_name("get")
                .about("Retrieve the content of a single Gist")
                .arg(
                    Arg::with_name("id")
                    .required(true)
                    .help("The ID of the given Gist")
                )
                .arg(
                    Arg::with_name("greedy")
                    .short("-g")
                    .long("--greedy")
                    .help("Attempt to retrieve files larger than 1MB in size")
                )
                .arg(
                    Arg::with_name("destination")
                    .short("-d")
                    .long("--destination")
                    .help("Write to a directory, rather than stdout. Implies -g")
                )
        ])
        .arg(Arg::with_name("verbosity")
            .short("v")
            .long("verbosity")
            .multiple(true)
            .help("Sets the level of verbosity")
        )
        .get_matches();

    loggerv::init_with_verbosity(matches.occurrences_of("verbosity")).unwrap();

    handle_matches(matches).await;
}

async fn handle_matches(matches: clap::ArgMatches<'_>) {
    match matches.subcommand() {
        ("create", Some(sc)) => handle_create_command(sc).await,
        ("list", Some(sc)) => handle_list_command(sc).await,
        ("get", Some(sc)) => {
            let _id = String::from(sc.value_of("id").unwrap());

            // TODO GET the Gist with the given ID as a list of files w/ content
            // If this fails, error out here.

            // TODO Carry out un-truncation here, if greedy or non-stdout

            // TODO write either to stdout or files in a dir
            // If stdout: how to break between files?
            // If to a directory: error if it doesn't exist
        }
        _ => {}
    }
}

async fn handle_create_command(sc: &clap::ArgMatches<'_>) {
    // Parse input
    let files: Vec<String> = sc.values_of("files").unwrap().map(String::from).collect();
    let is_public: bool = sc.is_present("private");
    let description: Option<String> = sc.value_of("description").map(|x| x.to_string()).take();
    // Process parsed input
    let res: Result<gstm::Gist, _> = gstm::create(files, is_public, description).await;
    // Print output
    match res {
        Ok(value) => println!("Gist available at {}", value.html_url),
        Err(e) => log::error!("Gist creation failed:\n\t{:?}", e),
    };
}

async fn handle_list_command(sc: &clap::ArgMatches<'_>) {
    // Parse input
    let user = sc.value_of("user").map(|x| x.to_string()).take();
    let since = sc
        .value_of("since")
        .map(|x| DateTime::parse_from_rfc3339(x).unwrap())
        .take();
    // Process input
    let gists = gstm::list(user, since).await;
    // Show output
    match gists {
        Ok(gs) => {
            for g in gs {
                // TODO Accurate method of printing w/ variable length truncation
                let description = match g.description {
                    Some(d) => {
                        let mut desc = d.replace("\n", " ");
                        let max_description_length = {
                            if let Some((w, _)) = term_size::dimensions() {
                                w / 3
                            } else {
                                40
                            }
                        };
                        if desc.len() > max_description_length {
                            desc.truncate(max_description_length);
                            desc.push_str("...");
                        }
                        desc
                    }
                    _ => String::new(),
                };

                let username: String = g.owner.map_or(String::new(), |o| o.login);
                println!("{} {} {} {}", g.created_at, username, g.id, description);
            }
        }
        Err(e) => log::error!("Retrieving gist listing failed:\n\t{:?}", e),
    }
}
