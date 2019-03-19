#!/usr/bin/env sh
set -ex


# Figure out what we are asked to do and "route" to the correct script.
case "$1" in
  install)
    case "$2" in
      clippy) ci/travis/clippy-install.sh;;
      test) ;;

      *)
        echo "Unsupported install task '$2' received"
        ;;
    esac
    ;;

  script)
    case "$2" in
      clippy) ci/travis/clippy-script.sh;;
      test) ci/travis/test-script.sh;;

      *)
        echo "Unsupported script task '$2' received"
        ;;
    esac
    ;;

  *)
    echo "Unsupported stage '$1' received"
    exit 1
    ;;
esac
