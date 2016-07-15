use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::io::BufRead;

use toml::Value;
use uuid::Uuid;

use task_hookrs::task::Task as TTask;
use task_hookrs::import::{import_task, import_tasks};

use libimagstore::store::{FileLockEntry, Store};
use libimagstore::storeid::{IntoStoreId, StoreIdIterator, StoreId};
use module_path::ModuleEntryPath;

use error::{TodoError, TodoErrorKind, MapErrInto};
use result::Result;

/// Task struct containing a `FileLockEntry`
#[derive(Debug)]
pub struct Task<'a>(FileLockEntry<'a>);

impl<'a> Task<'a> {

    /// Concstructs a new `Task` with a `FileLockEntry`
    pub fn new(fle: FileLockEntry<'a>) -> Task<'a> {
        Task(fle)
    }

    pub fn import<R: BufRead>(store: &'a Store, mut r: R) -> Result<(Task<'a>, Uuid)> {
        let mut line = String::new();
        r.read_line(&mut line);
        import_task(&line.as_str())
            .map_err_into(TodoErrorKind::ImportError)
            .and_then(|t| {
                let uuid = t.uuid().clone();
                t.into_task(store).map(|t| (t, uuid))
            })
    }

    /// Get a task from an import string. That is: read the imported string, get the UUID from it
    /// and try to load this UUID from store.
    ///
    /// Possible return values are:
    ///
    /// * Ok(Ok(Task))
    /// * Ok(Err(String)) - where the String is the String read from the `r` parameter
    /// * Err(_)          - where the error is an error that happened during evaluation
    ///
    pub fn get_from_import<R: BufRead>(store: &'a Store, mut r: R) -> Result<RResult<Task<'a>,
    String>> {
        unimplemented!()
    }

    /// Get a task from a String. The String is expected to contain the JSON-representation of the
    /// Task to get from the store (only the UUID really matters in this case)
    pub fn get_from_string(store: &'a Store, s: String) -> Result<Task<'a>> {
        unimplemented!()
    }

    /// Get a task from an UUID.
    pub fn get_from_uuid(store: &'a Store, uuid: Uuid) -> Result<Task<'a>> {
        unimplemented!()
    }

    /// Same as Task::get_from_import() but uses Store::retrieve() rather than Store::get(), to
    /// implicitely create the task if it does not exist.
    pub fn retrieve_from_import<R: BufRead>(store: &'a Store, mut r: R) -> Result<Task<'a>> {
        unimplemented!()
    }

    /// Retrieve a task from a String. The String is expected to contain the JSON-representation of
    /// the Task to retrieve from the store (only the UUID really matters in this case)
    pub fn retrieve_from_string(store: &'a Store, s: String) -> Result<Task<'a>> {
        unimplemented!()
    }

    /// Call `Task::update_from_imports_with()` function with `Task::copy_information()`
    pub fn update_from_imports<R: BufRead>(store: &'a Store, mut r: R) -> Result<(Task<'a>,
                                                                                  Task<'a>)> {
        unimplemented!()
    }

    /// Call `Task::update_from_strings_with()` function with `Task::copy_information()`
    pub fn update_from_strings<R>(store: &'a Store, old_t: String, new_t: String) -> Result<(Task<'a>, Task<'a>)>
        where R: BufRead
    {
        unimplemented!()
    }

    /// Call `Task::update_from_uuid_and_string_with()` function with `Task::copy_information()`
    pub fn update_from_uuid_and_string<R>(store: &'a Store, old_t: Uuid, new_t: String) -> Result<(Task<'a>, Task<'a>)>
        where R: BufRead
    {
        unimplemented!()
    }

    /// Update a task from imports.
    ///
    /// Expects the `r` variable to return two JSON objects, the first one beeing the Task before
    /// the modification, the second one after the modification.
    ///
    /// It then tries to load the first task, getting the corrosponding object from the store,
    /// and updating the task with the new information. That is
    ///
    ///  * Writing a new task object to the store
    ///  * Copying all information from the header of the old task to the new task via
    ///    the passed copy-functionality
    ///  * Returning both Task objects in a tuple (old, new)
    ///
    pub fn update_from_imports_with<R, F>(store: &'a Store, mut r: R, f: F) -> Result<(Task<'a>, Task<'a>)>
        where R: BufRead,
              F: Fn(&mut Task<'a>, &mut Task<'a>) -> Result<()>
    {
        unimplemented!()
    }

    /// Same as `Task::update_from_imports_with()` but with two Strings rather than reading from a
    /// BufRead. Expects both Strings to contain readable JSON.
    pub fn update_from_strings_with<R, F>(store: &'a Store, old_t: String, new_t: String, f: F) -> Result<(Task<'a>, Task<'a>)>
        where R: BufRead,
              F: Fn(&mut Task<'a>, &mut Task<'a>) -> Result<()>
    {
        unimplemented!()
    }

    /// Same as `Task::update_from_strings_with()` but with the parameter for the old task beeing
    /// the uuid of the task.
    pub fn update_from_uuid_and_string_with<R, F>(store: &'a Store, old_t: Uuid, new_t: String, f: F) -> Result<(Task<'a>, Task<'a>)>
        where R: BufRead,
              F: Fn(&mut Task<'a>, &mut Task<'a>) -> Result<()>
    {
        unimplemented!()
    }

    /// Copy information from one task to the second task. This function can be used to copy
    /// information from one task to another.
    ///
    /// Two kinds of informations are copied:
    ///
    /// 1. links (via libimagentrylink)
    /// 1. tags (via libimagentrytag)
    ///
    pub fn copy_information(old_task: &mut Task<'a>, new_task: &mut Task<'a>) -> Result<()> {
        unimplemented!()
    }

    pub fn delete_by_uuid(store: &Store, uuid: Uuid) -> Result<()> {
        store.delete(ModuleEntryPath::new(format!("taskwarrior/{}", uuid)).into_storeid())
            .map_err(|e| TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
    }

    pub fn all_as_ids(store: &Store) -> Result<StoreIdIterator> {
        store.retrieve_for_module("todo/taskwarrior")
            .map_err(|e| TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
    }

    pub fn all(store: &Store) -> Result<TaskIterator> {
        Task::all_as_ids(store)
            .map(|iter| TaskIterator::new(store, iter))
    }

}

impl<'a> Deref for Task<'a> {
    type Target = FileLockEntry<'a>;

    fn deref(&self) -> &FileLockEntry<'a> {
        &self.0
    }

}

impl<'a> DerefMut for Task<'a> {

    fn deref_mut(&mut self) -> &mut FileLockEntry<'a> {
        &mut self.0
    }

}

/// A trait to get a `libimagtodo::task::Task` out of the implementing object.
/// This Task struct is merely a wrapper for a `FileLockEntry`, therefore the function name
/// `into_filelockentry`.
pub trait IntoTask<'a> {

    /// # Usage
    /// ```ignore
    /// use std::io::stdin;
    ///
    /// use task_hookrs::task::Task;
    /// use task_hookrs::import::import;
    /// use libimagstore::store::{Store, FileLockEntry};
    ///
    /// if let Ok(task_hookrs_task) = import(stdin()) {
    ///     // Store is given at runtime
    ///     let task = task_hookrs_task.into_filelockentry(store);
    ///     println!("Task with uuid: {}", task.flentry.get_header().get("todo.uuid"));
    /// }
    /// ```
    fn into_task(self, store : &'a Store) -> Result<Task<'a>>;

}

impl<'a> IntoTask<'a> for TTask {

    fn into_task(self, store : &'a Store) -> Result<Task<'a>> {
        let uuid     = self.uuid();
        let store_id = ModuleEntryPath::new(format!("taskwarrior/{}", uuid)).into_storeid();

        match store.retrieve(store_id) {
            Err(e) => return Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e)))),
            Ok(mut fle) => {
                {
                    let mut header = fle.get_header_mut();
                    match header.read("todo") {
                        Ok(None) => {
                            if let Err(e) = header.set("todo", Value::Table(BTreeMap::new())) {
                                return Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
                            }
                        }
                        Ok(Some(_)) => { }
                        Err(e) => {
                            return Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
                        }
                    }

                    if let Err(e) = header.set("todo.uuid", Value::String(format!("{}",uuid))) {
                        return Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
                    }
                }

                // If none of the errors above have returned the function, everything is fine
                Ok(Task::new(fle))
            }
        }
    }

}

trait FromStoreId {
    fn from_storeid<'a>(&'a Store, StoreId) -> Result<Task<'a>>;
}

impl<'a> FromStoreId for Task<'a> {

    fn from_storeid<'b>(store: &'b Store, id: StoreId) -> Result<Task<'b>> {
        match store.retrieve(id) {
            Err(e) => Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e)))),
            Ok(c)  => Ok(Task::new( c )),
        }
    }
}

pub struct TaskIterator<'a> {
    store: &'a Store,
    iditer: StoreIdIterator,
}

impl<'a> TaskIterator<'a> {

    pub fn new(store: &'a Store, iditer: StoreIdIterator) -> TaskIterator<'a> {
        TaskIterator {
            store: store,
            iditer: iditer,
        }
    }

}

impl<'a> Iterator for TaskIterator<'a> {
    type Item = Result<Task<'a>>;

    fn next(&mut self) -> Option<Result<Task<'a>>> {
        self.iditer.next().map(|id| Task::from_storeid(self.store, id))
    }
}

