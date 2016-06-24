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

extern crate clap;
extern crate glob;
#[macro_use] extern crate log;
extern crate semver;
extern crate toml;
#[macro_use] extern crate version;

extern crate libimagrt;
extern crate libimagstore;
#[macro_use] extern crate libimagerror;

use std::result::Result as RResult;
use std::process::exit;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagstore::store::FileLockEntry;
use libimagerror::trace::trace_error;

mod error;
mod ui;
mod viewer;

use error::{ViewError, ViewErrorKind};
use ui::build_ui;
use viewer::Viewer;
use viewer::ViewInformation;
use viewer::stdout::StdoutViewer;

type Result<T> = RResult<T, ViewError>;

fn main() {
    let rt = generate_runtime_setup( "imag-view",
                                     &version!()[..],
                                     "View entries (readonly)",
                                     build_ui);

    let entry_id = rt.cli().value_of("id").unwrap(); // enforced by clap

    if rt.cli().is_present("versions") {
        if let Err(e) = view_versions_of(entry_id, &rt) {
            trace_error(&e);
            exit(1); // we can afford not-executing destructors here
        }
    } else {
        let entry_version   = rt.cli().value_of("version");
        let view_header     = rt.cli().is_present("view-header");
        let view_content    = rt.cli().is_present("view-content");
        let view_copy       = rt.cli().is_present("view-copy");
        let keep_copy       = rt.cli().is_present("keep-copy");

        let scmd = rt.cli().subcommand_matches("view-in");
        if scmd.is_none() {
            debug!("No commandline call");
            exit(1); // we can afford not-executing destructors here
        }
        let scmd = scmd.unwrap();

        let viewer = {
            if scmd.is_present("view-in-stdout") {
                Box::new(StdoutViewer::new())
            } else if scmd.is_present("view-in-ui") {
                warn!("Viewing in UI is currently not supported, switch to stdout");
                Box::new(StdoutViewer::new())
            } else if scmd.is_present("view-in-browser") {
                warn!("Viewing in browser is currently not supported, switch to stdout");
                Box::new(StdoutViewer::new())
            } else if scmd.is_present("view-in-texteditor") {
                warn!("Viewing in texteditor is currently not supported, switch to stdout");
                Box::new(StdoutViewer::new())
            } else if scmd.is_present("view-in-custom") {
                warn!("Viewing in custom is currently not supported, switch to stdout");
                Box::new(StdoutViewer::new())
            } else {
                Box::new(StdoutViewer::new())
            }
        };

        let entry = load_entry(entry_id, entry_version, &rt);
        if entry.is_err() {
            trace_error(&entry.unwrap_err());
            exit(1); // we can afford not-executing destructors here
        }
        let entry = entry.unwrap();

        let view_info = ViewInformation {
            entry:          entry,
            view_header:    view_header,
            view_content:   view_content,
            view_copy:      view_copy,
            keep_copy:      keep_copy,
        };

        viewer.view(view_info);
    }
}

// TODO: This is a shameless adaption of imag-store/src/util.rs
fn load_entry<'a>(id: &str,
                  version: Option<&str>,
                  rt: &'a Runtime)
    -> Result<FileLockEntry<'a>>
{
    debug!("Checking path element for version");

    let version = {
        if version.is_none() {
            let r = id.split('~').last();
            if r.is_none() {
                warn!("No version");
                return Err(ViewError::new(ViewErrorKind::NoVersion, None));
            } else {
                r.unwrap()
            }
        } else {
            version.unwrap()
        }
    };

    debug!("Building path from {:?} and {:?}", id, version);
    let mut path = rt.store().path().clone();

    if id.starts_with('/') {
        path.push(format!("{}~{}", &id[1..id.len()], version));
    } else {
        path.push(format!("{}~{}", id, version));
    }

    // the above is the adaption...

    rt.store().retrieve(path)
        .map_err(|e| ViewError::new(ViewErrorKind::StoreError, Some(Box::new(e))))
}

fn view_versions_of(id: &str, rt: &Runtime) -> Result<()> {
    use glob::glob;

    let mut path = rt.store().path().clone();

    if id.starts_with('/') {
        path.push(format!("{}~*", &id[1..id.len()]));
    } else {
        path.push(format!("{}~*", id));
    }

    if let Some(path) = path.to_str() {
        match glob(path) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => println!("{}", path.file_name().and_then(|s| s.to_str()).unwrap()),
                        Err(e)   => trace_error(e.error()),
                    }
                }
                Ok(())
            },
            Err(e) => {
                debug!("Error in pattern");
                Err(ViewError::new(ViewErrorKind::PatternError, Some(Box::new(e))))
            },
        }
    } else {
        warn!("Could not build glob() argument!");
        Err(ViewError::new(ViewErrorKind::GlobBuildError, None))
    }
}

