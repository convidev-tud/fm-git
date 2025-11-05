_fm_git() {
  COMPREPLY=($(fm-git __completion -- "${COMP_WORDS[@]}"))
}
complete -F _fm_git fm-git
