/*
 *  imag - The personal information management suite for the commandline
 *  Copyright (C) 2016    Matthias Beyer <mail@beyermatthias.de>
 *
 *  This library is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU Lesser General Public
 *  License as published by the Free Software Foundation; version
 *  2.1 of the License.
 *
 *  This library is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *  Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public
 *  License along with this library; if not, write to the Free Software
 *  Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

#[macro_use] extern crate log;
extern crate clap;
#[macro_use] extern crate semver;
extern crate toml;
extern crate url;
#[macro_use] extern crate version;

extern crate libimagentrylink;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagerror;

use std::process::exit;
use std::ops::Deref;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagstore::error::StoreError;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagerror::trace::trace_error;
use libimagentrylink::external::ExternalLinker;
use clap::ArgMatches;
use url::Url;

mod ui;

use ui::build_ui;

fn main() {
    let rt = generate_runtime_setup("imag-link",
                                    &version!()[..],
                                    "Link entries",
                                    build_ui);

    rt.cli()
        .subcommand_name()
        .map(|name| {
            match name {
                "internal" => handle_internal_linking(&rt),
                "external" => handle_external_linking(&rt),
                _ => {
                    warn!("No commandline call");
                    exit(1);
                },
            }
        });
}

fn handle_internal_linking(rt: &Runtime) {
    use libimagentrylink::internal::InternalLinker;
    use libimagerror::trace::trace_error;

    debug!("Handle internal linking call");
    let cmd = rt.cli().subcommand_matches("internal").unwrap();

    if cmd.is_present("list") {
        debug!("List...");
        for entry in cmd.value_of("list").unwrap().split(',') {
            debug!("Listing for '{}'", entry);
            match get_entry_by_name(rt, entry) {
                Ok(e) => {
                    e.get_internal_links()
                        .map(|links| {
                            for (i, link) in links.iter().map(|l| l.to_str()).filter_map(|x| x).enumerate() {
                                println!("{: <3}: {}", i, link);
                            }
                        })
                        .map_err(|e| trace_error(&e))
                        .ok();
                },

                Err(e) => {
                    trace_error(&e);
                    break;
                },
            }
        }
        debug!("Listing ready!");
    } else {
        let mut from = {
            let from = get_from_entry(&rt);
            if from.is_none() {
                warn!("No 'from' entry");
                exit(1);
            }
            from.unwrap()
        };
        debug!("Link from = {:?}", from.deref());

        let to = {
            let to = get_to_entries(&rt);
            if to.is_none() {
                warn!("No 'to' entry");
                exit(1);
            }
            to.unwrap()
        };
        debug!("Link to = {:?}", to.iter().map(|f| f.deref()).collect::<Vec<&Entry>>());

        match cmd.subcommand_name() {
            Some("add") => {
                for mut to_entry in to {
                    if let Err(e) = to_entry.add_internal_link(&mut from) {
                        trace_error(&e);
                        exit(1);
                    }
                }
            },

            Some("remove") => {
                for mut to_entry in to {
                    if let Err(e) = to_entry.remove_internal_link(&mut from) {
                        trace_error(&e);
                        exit(1);
                    }
                }
            },

            _ => unreachable!(),
        };
    }
}

fn get_from_entry<'a>(rt: &'a Runtime) -> Option<FileLockEntry<'a>> {
    rt.cli()
        .subcommand_matches("internal")
        .unwrap() // safe, we know there is an "internal" subcommand"
        .subcommand_matches("add")
        .unwrap() // safe, we know there is an "add" subcommand
        .value_of("from")
        .and_then(|from_name| {
            match get_entry_by_name(rt, from_name) {
                Err(e) => {
                    debug!("We couldn't get the entry from name: '{:?}'", from_name);
                    trace_error(&e); None
                },
                Ok(e) => Some(e),
            }

        })
}

fn get_to_entries<'a>(rt: &'a Runtime) -> Option<Vec<FileLockEntry<'a>>> {
    rt.cli()
        .subcommand_matches("internal")
        .unwrap() // safe, we know there is an "internal" subcommand"
        .subcommand_matches("add")
        .unwrap() // safe, we know there is an "add" subcommand
        .values_of("to")
        .map(|values| {
            let mut v = vec![];
            for entry in values.map(|v| get_entry_by_name(rt, v)) {
                match entry {
                    Err(e) => trace_error(&e),
                    Ok(e) => v.push(e),
                }
            }
            v
        })
}

fn get_entry_by_name<'a>(rt: &'a Runtime, name: &str) -> Result<FileLockEntry<'a>, StoreError> {
    use libimagstore::storeid::build_entry_path;
    build_entry_path(rt.store(), name)
        .and_then(|path| rt.store().retrieve(path))
}

fn handle_external_linking(rt: &Runtime) {
    use libimagerror::trace::trace_error;

    let scmd       = rt.cli().subcommand_matches("external").unwrap();
    let entry_name = scmd.value_of("id").unwrap(); // enforced by clap
    let entry      = get_entry_by_name(rt, entry_name);
    if entry.is_err() {
        trace_error(&entry.unwrap_err());
        exit(1);
    }
    let mut entry = entry.unwrap();

    if scmd.is_present("add") {
        debug!("Adding link to entry!");
        add_link_to_entry(rt.store(), scmd, &mut entry);
        return;
    }

    if scmd.is_present("remove") {
        debug!("Removing link from entry!");
        remove_link_from_entry(rt.store(), scmd, &mut entry);
        return;
    }

    if scmd.is_present("set") {
        debug!("Setting links in entry!");
        set_links_for_entry(rt.store(), scmd, &mut entry);
        return;
    }

    if scmd.is_present("list") {
        debug!("Listing links in entry!");
        list_links_for_entry(rt.store(), &mut entry);
        return;
    }

    panic!("Clap failed to enforce one of 'add', 'remove', 'set' or 'list'");
}

fn add_link_to_entry(store: &Store, matches: &ArgMatches, entry: &mut FileLockEntry) {
    let link = matches.value_of("add").unwrap();

    let link = Url::parse(link);
    if link.is_err() {
        debug!("URL parsing error...");
        trace_error(&link.unwrap_err());
        debug!("Exiting");
        exit(1);
    }
    let link = link.unwrap();

    if let Err(e) = entry.add_external_link(store, link) {
        debug!("Error while adding external link...");
        trace_error(&e);
    } else {
        debug!("Everything worked well");
        info!("Ok");
    }
}

fn remove_link_from_entry(store: &Store, matches: &ArgMatches, entry: &mut FileLockEntry) {
    let link = matches.value_of("remove").unwrap();

    let link = Url::parse(link);
    if link.is_err() {
        trace_error(&link.unwrap_err());
        exit(1);
    }
    let link = link.unwrap();

    if let Err(e) = entry.remove_external_link(store, link) {
        trace_error(&e);
    } else {
        info!("Ok");
    }
}

fn set_links_for_entry(store: &Store, matches: &ArgMatches, entry: &mut FileLockEntry) {
    let links = matches
        .value_of("links")
        .map(String::from)
        .unwrap()
        .split(',')
        .map(|uri| {
            match Url::parse(uri) {
                Err(e) => {
                    warn!("Could not parse '{}' as URL, ignoring", uri);
                    trace_error(&e);
                    None
                },
                Ok(u) => Some(u),
            }
        })
        .filter_map(|x| x)
        .collect();

    if let Err(e) = entry.set_external_links(store, links) {
        trace_error(&e);
    } else {
        info!("Ok");
    }
}

fn list_links_for_entry(store: &Store, entry: &mut FileLockEntry) {
    let res = entry.get_external_links(store)
        .and_then(|links| {
            for (i, link) in links.iter().enumerate() {
                println!("{: <3}: {}", i, link);
            }
            Ok(())
        });

    match res {
        Err(e) => {
            trace_error(&e);
        },
        Ok(_) => {
            info!("Ok");
        },
    }
}

