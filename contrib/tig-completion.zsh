#compdef tig
#
# zsh completion wrapper for tig
# ==============================
#
# You need to install this script to zsh fpath with tig-completion.bash.
#
# The recommended way to install this script is to copy this and tig-completion.bash
# to '~/.zsh/_tig' and '~/.zsh/tig-completion.bash' and
# then add following to your ~/.zshrc file:
#
#  fpath=(~/.zsh $fpath)
#
# You also need Git's Zsh completion installed:
#
# https://github.com/felipec/git-completion/blob/master/git-completion.zsh


_tig () {
  local e dir

  dir=$(dirname ${funcsourcetrace[1]%:*})

  e=$dir/tig-completion.bash
  if [ -f $e ]; then
    # Temporarily override __git_complete so the bash script doesn't complain
    local old="$functions[__git_complete]"
    functions[__git_complete]=:
    . $e
    functions[__git_complete]="$old"
  fi

  # tig-completion.bash is written against Git's bash completion and expects
  # the git-completion.zsh wrapper (felipec). Most Zsh setups ship a native
  # `_git` completion, so try to bootstrap the wrapper when needed.
  if ! (( $+functions[__git_complete_command] )); then
    local cand old_git_def old_git_was_autoload=0

    case "$(whence -v _git 2>/dev/null)" in
    (*autoload*) old_git_was_autoload=1 ;;
    esac
    old_git_def="$functions[_git]"

    for cand in \
      "$dir/git-completion.zsh" \
      "/Library/Developer/CommandLineTools/usr/share/git-core/git-completion.zsh" \
      "/Applications/Xcode.app/Contents/Developer/usr/share/git-core/git-completion.zsh" \
      "/usr/share/git-core/git-completion.zsh"; do
      if [ -f "$cand" ]; then
        # Source the wrapper to get __git_complete_command, __gitcomp, etc.
        . "$cand"

        # Keep a dedicated wrapper for tig so we don't clobber the user's `_git`.
        functions[_tig_git]="$functions[_git]"
        compdef _tig_git tig

        if (( old_git_was_autoload )); then
          unfunction _git 2>/dev/null
          autoload -Uz _git 2>/dev/null
        elif [ -n "$old_git_def" ]; then
          functions[_git]="$old_git_def"
        else
          unfunction _git 2>/dev/null
        fi

        return 0
      fi
    done
  fi

  # Finish the completion on the first tab press.
  if (( $+functions[__git_complete_command] )); then
    compdef _git tig
    _git
  else
    # Fallback: basic tig options and commands.
    compadd -Q -S '' -- \
      -v --version \
      -h --help \
      -C \
      blame grep log reflog refs stash status show
  fi
}
