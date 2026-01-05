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
# If your Tig bash completion file is installed elsewhere (often as `tig` in a
# bash-completion completions directory), you can point this wrapper to it:
#
#  zstyle ':completion:*:*:tig:*' script /path/to/tig
#
# You also need Git's Zsh completion installed:
#
# https://github.com/felipec/git-completion/blob/master/git-completion.zsh


_tig () {
  local e dir script bash_completion
  local -a locations

  dir=$(dirname "${funcsourcetrace[1]%:*}")

  zstyle -s ":completion:*:*:tig:*" script script
  if [ -n "$script" ]; then
    locations=("$script")
  else
    bash_completion=$(pkg-config --variable=completionsdir bash-completion 2>/dev/null) ||
      bash_completion='/usr/share/bash-completion/completions/'

    locations=(
      "$dir/tig-completion.bash"
      "$dir/tig"
      "$HOME/.local/share/bash-completion/completions/tig"
      "$bash_completion/tig"
      "/opt/homebrew/share/bash-completion/completions/tig"
      "/usr/local/share/bash-completion/completions/tig"
      "/opt/homebrew/etc/bash_completion.d/tig"
      "/usr/local/etc/bash_completion.d/tig"
      '/etc/bash_completion.d/tig' # old debian
    )
  fi

  for e in "${locations[@]}"; do
    if [ -f "$e" ]; then
      # Temporarily override __git_complete so the bash script doesn't complain
      local old="$functions[__git_complete]"
      functions[__git_complete]=:
      . "$e"
      if [ -n "$old" ]; then
        functions[__git_complete]="$old"
      else
        unfunction __git_complete 2>/dev/null
      fi
      break
    fi
  done

  # tig-completion.bash is written against Git's bash completion and expects
  # the git-completion.zsh wrapper (felipec). Most Zsh setups ship a native
  # `_git` completion, so try to bootstrap the wrapper when needed.
  if (( $+functions[__tig_main] )) && ! (( $+functions[__git_complete_command] )); then
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
  if (( $+functions[__tig_main] )) && (( $+functions[__git_complete_command] )); then
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
