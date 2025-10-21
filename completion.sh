_fm_git() {
    COMPREPLY=($(COMP_CWORD=$COMP_CWORD COMP_LINE=$COMP_LINE COMP_POINT=$COMP_POINT \
        fm-git __completion -- "${COMP_WORDS[@]}"))
}
complete -F _fm_git fm-git

