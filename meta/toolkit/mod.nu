export module log.nu
export module tracking.nu

# The absolute path of the root of the repository
export const ROOT_PATH = (path self ../..)


# This function should be constant to be able to use it on constant functions,
# but it's not possible at the moment on Nushell
# TRACK: https://github.com/nushell/nushell/issues/15074
# Get an absolute path from the root of the monorepo
export def from-root [
    path: string # The relative to the monorepo root, you MUST NOT prepend `/` (slash) before it
]: nothing -> string {
    return $"($ROOT_PATH)/($path)"
}