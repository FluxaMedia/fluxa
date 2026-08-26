import Foundation
import Testing
@testable import FluxaPlayerKit

@Test
func parsesWebVttCueIdentifiersAndMarkup() {
    let source = """
    WEBVTT

    cue-1
    00:00:01.250 --> 00:00:03.500
    <b>Hello</b>\nworld
    """

    let cues = FluxaSubtitleParser.parse(Data(source.utf8))
    #expect(cues == [
        FluxaSubtitleCue(start: 1.25, end: 3.5, text: "Hello\nworld")
    ])
}

@Test
func parsesSrtCommaMillisecondsAndSkipsInvalidCues() {
    let source = """
    1
    00:00:02,000 --> 00:00:04,250
    First line

    2
    invalid --> timing
    ignored

    3
    00:00:05,000 --> 00:00:06,000
    Second line
    """

    let cues = FluxaSubtitleParser.parse(Data(source.utf8))
    #expect(cues == [
        FluxaSubtitleCue(start: 2, end: 4.25, text: "First line"),
        FluxaSubtitleCue(start: 5, end: 6, text: "Second line")
    ])
}
