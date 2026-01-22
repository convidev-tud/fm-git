_tangl() {
  COMPREPLY=($(tangl __completion -- "${COMP_WORDS[@]}"))
}
complete -F _tangl tangl
