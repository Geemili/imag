function __imag_executables() {
    echo -n "$PATH" | \
    xargs \
        -d: \
        -I{} \
        -r \
        -- find \
            -L {} \
            -maxdepth 1 \
            -mindepth 1 \
            -type f \
            -executable \
            -printf '%P\n' 2>/dev/null | \
    sort -u | grep "^imag"
}

_imag() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="$()" # find the other imag-* binaries from $PATH here

    if [[ -z "$cur" || $(echo $opts | grep $cur) ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
}
complete -F _imag imag
