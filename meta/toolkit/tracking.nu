use log.nu
use constants.nu

def get-track-entries []: nothing -> list<string> {
    cd $ROOT_PATH

    # CHECK: This will not work if we want to add TRACK information in this file
    # A proper CLI tracking tool should be made in the future, also removing the
    # need for the `/run todo` subcommand
    rg "TRACK: " --json --glob $"!/meta/toolkit/tracking.nu"
        | lines
        | each {|entry| $entry |from json}
        | where type == "match"
        | each {|match| $match.data.lines.text }
        | each {|text| $text | parse -r "(?s)\\A.*TRACK: (?P<url>.*?)\\z"}
        | each {|entry| $entry | get url | get 0}
        | each {|entry| $entry | str trim --char "\n"}
}

def handle-url [url_text: string]: nothing -> string {
    let url = $url_text | url parse

    if $url.host != "github.com" {
        return $url_text    
    }
    
    # Good candidate for record destructuring in the future
    # TRACK: https://github.com/nushell/nushell/issues/6021
    let issue_data = $url | get path | parse "/{owner}/{repo}/issues/{id}"

    if ($issue_data | length) <= 0 {
        return $url_text
    }

    let issue_data = $issue_data | get 0

    let issue_status = curl -s $"https://api.github.com/repos/($issue_data.owner)/($issue_data.repo)/issues/($issue_data.id)"
        | from json
        | get state
    
    mut prefix = "Github Issue (Open)"
    if $issue_status == "closed" {
        $prefix = "Github Issue (Closed!)"
    }

    return $"($prefix): ($url_text)"
}

export def list [] {
    let entries = get-track-entries

    log info "URLs to track:"
    $entries | each { |entry|
        log info $"- (handle-url $entry)"
    } | ignore
}