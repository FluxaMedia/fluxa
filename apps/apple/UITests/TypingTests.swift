import XCTest

final class TypingTests: XCTestCase {
    func testTypingReachesWelcomeEmailField() throws {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)"]
        app.launch()
        sleep(15)
        attach(app, name: "launch")

        let intro = app.buttons["Continue"]
        if intro.waitForExistence(timeout: 5) {
            intro.tap()
        }

        let screen = app.frame
        app.coordinate(withNormalizedOffset: .zero)
            .withOffset(CGVector(dx: screen.width / 2, dy: screen.height * 0.396))
            .tap()
        sleep(2)
        attach(app, name: "focused")

        let keys = app.keyboards.keys
        XCTAssertTrue(keys["h"].waitForExistence(timeout: 10), app.debugDescription)
        for key in "hello" {
            keys[String(key)].tap()
        }
        sleep(2)
        attach(app, name: "typed")

        keys["delete"].tap()
        sleep(2)
        attach(app, name: "deleted")
    }

    private func attach(_ app: XCUIApplication, name: String) {
        let shot = XCTAttachment(screenshot: app.screenshot())
        shot.name = name
        shot.lifetime = .keepAlways
        add(shot)
    }
}
