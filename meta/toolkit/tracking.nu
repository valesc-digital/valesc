# FIX THIS
#use mod.nu

def get-track-entries []: nothing -> list<string> {
    rg "TRACK: " --json --glob $"!/run"
        | lines
        | each {|entry| $entry |from json}
        | where type == "match"
        | each {|match| $match.data.lines.text }
        | each {|text| $text | parse -r "(?s)\\A.*TRACK: (?P<url>.*?)\\z"}
        | each {|entry| $entry | get url | get 0}
        | each {|entry| $entry | str trim --char "\n"}
}

export def list [] {
    let entries = get-track-entries

    log info "URLs to track:"
    $entries | each { |entry|
        log info $"- ($entry)"
    } | ignore
}