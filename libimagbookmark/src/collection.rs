//! BookmarkCollection module
//!
//! A BookmarkCollection is nothing more than a simple store entry. One can simply call functions
//! from the libimagentrylink::external::ExternalLinker trait on this to generate external links.
//!
//! The BookmarkCollection type offers helper functions to get all links or such things.
use std::ops::Deref;
use std::ops::DerefMut;

use regex::Regex;

use error::BookmarkErrorKind as BEK;
use error::MapErrInto;
use result::Result;
use module_path::ModuleEntryPath;

use libimagstore::store::Store;
use libimagstore::storeid::IntoStoreId;
use libimagstore::store::FileLockEntry;
use libimagentrylink::external::ExternalLinker;
use libimagentrylink::internal::InternalLinker;
use libimagentrylink::internal::Link as StoreLink;
use libimagerror::into::IntoError;
use url::Url;

use link::Link;

pub struct BookmarkCollection<'a> {
    fle: FileLockEntry<'a>,
    store: &'a Store,
}

/// {Internal, External}Linker is implemented as Deref is implemented
impl<'a> Deref for BookmarkCollection<'a> {
    type Target = FileLockEntry<'a>;

    fn deref(&self) -> &FileLockEntry<'a> {
        &self.fle
    }

}

impl<'a> DerefMut for BookmarkCollection<'a> {

    fn deref_mut(&mut self) -> &mut FileLockEntry<'a> {
        &mut self.fle
    }

}

impl<'a> BookmarkCollection<'a> {

    pub fn new(store: &'a Store, name: &str) -> Result<BookmarkCollection<'a>> {
        let id = ModuleEntryPath::new(name).into_storeid();
        store.create(id)
            .map(|fle| {
                BookmarkCollection {
                    fle: fle,
                    store: store,
                }
            })
            .map_err_into(BEK::StoreReadError)
    }

    pub fn get(store: &'a Store, name: &str) -> Result<BookmarkCollection<'a>> {
        let id = ModuleEntryPath::new(name).into_storeid();
        store.get(id)
            .map_err_into(BEK::StoreReadError)
            .and_then(|fle| {
                match fle {
                    None => Err(BEK::CollectionNotFound.into_error()),
                    Some(e) => Ok(BookmarkCollection {
                        fle: e,
                        store: store,
                    }),
                }
            })
    }

    pub fn delete(store: &Store, name: &str) -> Result<()> {
        store.delete(ModuleEntryPath::new(name).into_storeid()).map_err_into(BEK::StoreReadError)
    }

    pub fn links(&self) -> Result<Vec<Url>> {
        self.fle.get_external_links(&self.store).map_err_into(BEK::LinkError)
    }

    pub fn link_entries(&self) -> Result<Vec<StoreLink>> {
        use libimagentrylink::external::is_external_link_storeid;

        self.fle
            .get_internal_links()
            .map(|v| v.into_iter().filter(|id| is_external_link_storeid(id)).collect())
            .map_err_into(BEK::StoreReadError)
    }

    pub fn add_link(&mut self, l: Link) -> Result<()> {
        use link::IntoUrl;

        l.into_url()
            .and_then(|url| self.add_external_link(self.store, url).map_err_into(BEK::LinkingError))
            .map_err_into(BEK::LinkError)
    }

    pub fn get_links_matching(&self, r: Regex) -> Result<Vec<Link>> {
        self.get_external_links(self.store)
            .map_err_into(BEK::LinkError)
            .map(|v| {
                v.into_iter()
                    .map(Url::into_string)
                    .filter(|urlstr| r.is_match(&urlstr[..]))
                    .map(Link::from)
                    .collect()
            })
    }

    pub fn remove_link(&mut self, l: Link) -> Result<()> {
        use link::IntoUrl;

        l.into_url()
            .and_then(|url| {
                self.remove_external_link(self.store, url).map_err_into(BEK::LinkingError)
            })
            .map_err_into(BEK::LinkError)
    }

}

