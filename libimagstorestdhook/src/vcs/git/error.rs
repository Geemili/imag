generate_error_module!(
    generate_error_types!(GitHookError, GitHookErrorKind,
        ConfigError => "Configuration Error",
        ConfigTypeError => "Configuration value type wrong",
        RepositoryError                   => "Error while interacting with git repository",
        RepositoryBranchError             => "Error while interacting with git branch(es)",
        RepositoryBranchNameFetchingError => "Error while fetching branch name",
        MkRepo => "Repository creation error"
    );
);

pub use self::error::GitHookError;
pub use self::error::GitHookErrorKind;
pub use self::error::MapErrInto;

