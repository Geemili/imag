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

use std::path::PathBuf;
use std::io::stdin;
use std::fs::OpenOptions;
use std::result::Result as RResult;
use std::io::Read;
use std::ops::DerefMut;
use std::io::Write;
use std::io::stderr;
use std::process::exit;

use clap::ArgMatches;

use libimagrt::runtime::Runtime;
use libimagstore::store::Entry;
use libimagstore::store::EntryHeader;
use libimagstore::storeid::build_entry_path;
use libimagerror::trace::trace_error;

use error::StoreError;
use error::StoreErrorKind;
use util::build_toml_header;

type Result<T> = RResult<T, StoreError>;

pub fn create(rt: &Runtime) {
    rt.cli()
        .subcommand_matches("create")
        .map(|scmd| {
            debug!("Found 'create' subcommand...");

            // unwrap is safe as value is required
            let path = scmd.value_of("path").or_else(|| scmd.value_of("id"));
            if path.is_none() {
                warn!("No ID / Path provided. Exiting now");
                write!(stderr(), "No ID / Path provided. Exiting now").ok();
                exit(1);
            }

            let path = build_entry_path(rt.store(), path.unwrap());
            if path.is_err() {
                trace_error(&path.unwrap_err());
                exit(1);
            }
            let path = path.unwrap();
            debug!("path = {:?}", path);

            if scmd.subcommand_matches("entry").is_some() {
                create_from_cli_spec(rt, scmd, &path)
                    .or_else(|_| create_from_source(rt, scmd, &path))
                    .or_else(|_| create_with_content_and_header(rt,
                                                                &path,
                                                                String::new(),
                                                                EntryHeader::new()))
            } else {
                create_with_content_and_header(rt, &path, String::new(), EntryHeader::new())
            }
            .unwrap_or_else(|e| debug!("Error building Entry: {:?}", e))
        });
}

fn create_from_cli_spec(rt: &Runtime, matches: &ArgMatches, path: &PathBuf) -> Result<()> {
    let content = matches.subcommand_matches("entry")
        .map_or_else(|| {
            debug!("Didn't find entry subcommand, getting raw content");
            matches.value_of("from-raw")
                .map_or_else(String::new, string_from_raw_src)
        }, |entry_subcommand| {
            debug!("Found entry subcommand, parsing content");
            entry_subcommand
                .value_of("content")
                .map_or_else(|| {
                    entry_subcommand.value_of("content-from")
                        .map_or_else(String::new, string_from_raw_src)
                }, String::from)
        });
    debug!("Got content with len = {}", content.len());

    let header = matches.subcommand_matches("entry")
        .map_or_else(EntryHeader::new,
            |entry_matches| build_toml_header(entry_matches, EntryHeader::new()));

    create_with_content_and_header(rt, path, content, header)
}

fn create_from_source(rt: &Runtime, matches: &ArgMatches, path: &PathBuf) -> Result<()> {
    let content = matches
        .value_of("from-raw")
        .ok_or(StoreError::new(StoreErrorKind::NoCommandlineCall, None))
        .map(string_from_raw_src);

    if content.is_err() {
        return content.map(|_| ());
    }
    let content = content.unwrap();
    debug!("Content with len = {}", content.len());

    Entry::from_str(path.clone(), &content[..])
        .and_then(|new_e| {
            let r = rt.store()
                .create(path.clone())
                .map(|mut old_e| {
                    *old_e.deref_mut() = new_e;
                });

            debug!("Entry build");
            r
        })
        .map_err(|serr| StoreError::new(StoreErrorKind::BackendError, Some(Box::new(serr))))
}

fn create_with_content_and_header(rt: &Runtime,
                                  path: &PathBuf,
                                  content: String,
                                  header: EntryHeader) -> Result<()>
{
    debug!("Creating entry with content");
    rt.store()
        .create(PathBuf::from(path))
        .map(|mut element| {
            {
                let mut e_content = element.get_content_mut();
                *e_content = content;
                debug!("New content set");
            }
            {
                let mut e_header  = element.get_header_mut();
                *e_header = header;
                debug!("New header set");
            }
        })
        .map_err(|e| StoreError::new(StoreErrorKind::BackendError, Some(Box::new(e))))
}

fn string_from_raw_src(raw_src: &str) -> String {
    let mut content = String::new();
    if raw_src == "-" {
        debug!("Reading entry from stdin");
        let res = stdin().read_to_string(&mut content);
        debug!("Read {:?} bytes", res);
    } else {
        debug!("Reading entry from file at {:?}", raw_src);
        let _ = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(raw_src)
            .and_then(|mut f| f.read_to_string(&mut content));
    }
    content
}
