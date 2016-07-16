use std::path::PathBuf;
use std::fmt::{Debug, Formatter, Error as FmtError};

use toml::Value;
use git2::{Repository, Error as Git2Error};

use libimagstore::storeid::StoreId;
use libimagstore::hook::Hook;
use libimagstore::hook::error::HookError;
use libimagstore::hook::error::HookErrorKind;
use libimagstore::hook::error::CustomData as HECD;
use libimagstore::hook::result::HookResult;
use libimagstore::hook::position::HookPosition;
use libimagstore::hook::accessor::{HookDataAccessor, HookDataAccessorProvider};
use libimagstore::hook::accessor::StoreIdAccessor;
use libimagerror::trace::trace_error;
use libimagerror::into::IntoError;

use vcs::git::error::GitHookErrorKind as GHEK;
use vcs::git::error::GitHookError as GHE;

pub struct CreateHook<'a> {
    storepath: &'a PathBuf,

    repository: Option<Repository>,

    position: HookPosition,
    config: Option<Value>,
}

impl<'a> CreateHook<'a> {

    pub fn new(storepath: &'a PathBuf, p: HookPosition) -> CreateHook<'a> {
        let r = match Repository::open(storepath) {
            Ok(r) => Some(r),
            Err(e) => {
                trace_error(&e);
                None
            },
        };
        CreateHook {
            storepath: storepath,
            repository: r,
            position: p,
            config: None,
        }
    }

}

impl<'a> Debug for CreateHook<'a> {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        write!(fmt, "CreateHook(storepath={:?}, repository={}, pos={:?}, cfg={:?}",
               self.storepath,
               (if self.repository.is_some() { "Some(_)" } else { "None" }),
               self.position,
               self.config)
    }
}

impl<'a> Hook for CreateHook<'a> {

    fn name(&self) -> &'static str {
        "stdhook_git_create"
    }

    fn set_config(&mut self, config: &Value) {
        self.config = Some(config.clone());
    }

}

impl<'a> HookDataAccessorProvider for CreateHook<'a> {

    fn accessor(&self) -> HookDataAccessor {
        HookDataAccessor::StoreIdAccess(self)
    }
}

impl<'a> StoreIdAccessor for CreateHook<'a> {

    fn access(&self, id: &StoreId) -> HookResult<()> {
        use vcs::git::config::abort_on_repo_init_err;

        debug!("[GIT CREATE HOOK]: {:?}", id);
        if self.repository.is_none() {
            debug!("Repository isn't initialized... creating error object now");
            let he = GHEK::MkRepo.into_error();
            let he = HookError::new(HookErrorKind::HookExecutionError, Some(Box::new(he)));
            let custom = HECD::default().aborting(abort_on_repo_init_err(self.config.as_ref()));
            return Err(he.with_custom_data(custom));
        }
        let repository = self.repository.as_ref().unwrap();

        if !repository.head().is_branch() {
            return Err(GHEK::NotOnBranch.into_error()).map_err_into(HEK::HookExecutionError)
        }

        // Check out appropriate branch ... or fail
        match ensure_branch(self.config.as_ref()) {
            Ok(Some(s)) => {
                match repository.head().name().map(|name| name == s) {
                    Some(b) => {
                        if b {
                            debug!("Branch already checked out.");
                        } else {
                            debug!("Branch not checked out.");
                            unimplemented!()
                        }
                    }

                    None => return Err(GHEK::RepositoryBranchNameFetchingError.into_error())
                        .map_err_into(GHEK::RepositoryBranchError)
                        .map_err_into(GHEK::RepositoryError),
                }
            },
            Ok(None) => {
                debug!("No branch to checkout");
            },

            Err(e) => return Err(e),
        }

        // Now to the create() hook action

        unimplemented!()

        Ok(())
    }

}

