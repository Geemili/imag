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

use std::ops::DerefMut;

use runtime::Runtime;
use error::RuntimeError;
use error::RuntimeErrorKind;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Entry;

use libimagerror::into::IntoError;

pub type EditResult<T> = Result<T, RuntimeError>;

pub trait Edit {
    fn edit_content(&mut self, rt: &Runtime) -> EditResult<()>;
}

impl Edit for String {

    fn edit_content(&mut self, rt: &Runtime) -> EditResult<()> {
        edit_in_tmpfile(rt, self).map(|_| ())
    }

}

impl Edit for Entry {

    fn edit_content(&mut self, rt: &Runtime) -> EditResult<()> {
        edit_in_tmpfile(rt, self.get_content_mut())
            .map(|_| ())
    }

}

impl<'a> Edit for FileLockEntry<'a> {

    fn edit_content(&mut self, rt: &Runtime) -> EditResult<()> {
        self.deref_mut().edit_content(rt)
    }

}

pub fn edit_in_tmpfile(rt: &Runtime, s: &mut String) -> EditResult<()> {
    use tempfile::NamedTempFile;
    use std::io::Seek;
    use std::io::Read;
    use std::io::SeekFrom;
    use std::io::Write;

    let file      = try!(NamedTempFile::new());
    let file_path = file.path();
    let mut file  = try!(file.reopen());

    try!(file.write_all(&s.clone().into_bytes()[..]));
    try!(file.sync_data());

    if let Some(mut editor) = rt.editor() {
        let exit_status = editor.arg(file_path).status();

        match exit_status.map(|s| s.success()).map_err(Box::new) {
            Ok(true)  => {
                file.sync_data()
                    .and_then(|_| file.seek(SeekFrom::Start(0)))
                    .and_then(|_| {
                        let mut new_s = String::new();
                        let res = file.read_to_string(&mut new_s);
                        *s = new_s;
                        res
                    })
                    .map(|_| ())
                    .map_err(Box::new)
                    .map_err(|e| RuntimeErrorKind::IOError.into_error_with_cause(e))
            },
            Ok(false) => Err(RuntimeErrorKind::ProcessExitFailure.into()),
            Err(e)    => Err(RuntimeErrorKind::IOError.into_error_with_cause(e)),
        }
    } else {
        Err(RuntimeErrorKind::Instantiate.into())
    }
}
