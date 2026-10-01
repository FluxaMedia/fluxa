
fn sample_payload() -> &'static str {
    r##"{
        "filters": [
            {"name":"4K","pattern":"\\b4k\\b","imageURL":"https://example.com/4k.png"},
            {"name":"HDR","pattern":"\\bhdr\\b"},
            {"name":"Disabled","pattern":"nope","isEnabled":false}
        ],
        "groups": [{"id":"g1","name":"Quality","color":"#fff"}]
    }"##
}








