import XCTest

final class TypingTests: XCTestCase {
    func testTypingReachesWelcomeEmailField() throws {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)"]
        app.launch()
        sleep(20)
        attach(app, name: "launch")

        let intro = app.buttons["Continue"]
        if intro.waitForExistence(timeout: 5) {
            intro.tap()
        }

        let window = app.windows.firstMatch.frame
        let field = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == %@", "you@example.com"))
            .firstMatch
        var y = window.height * 0.25
        var step = 0
        while !field.exists && y < window.height * 0.5 {
            app.coordinate(withNormalizedOffset: .zero)
                .withOffset(CGVector(dx: window.width / 2, dy: y))
                .tap()
            sleep(2)
            attach(app, name: "tap-\(step)")
            y += 30
            step += 1
        }
        XCTAssertTrue(field.exists, "email field never focused")

        let keys = app.keyboards.keys
        XCTAssertTrue(keys["h"].waitForExistence(timeout: 10), app.debugDescription)
        for key in "hello" {
            keys[String(key)].tap()
        }
        sleep(2)
        attach(app, name: "after-typing")

        let typed = app.descendants(matching: .any)
            .matching(NSPredicate(format: "value == %@", "hello"))
            .firstMatch
        XCTAssertTrue(typed.waitForExistence(timeout: 10), "typed text not shown\n" + app.debugDescription)

        keys["delete"].tap()
        let trimmed = app.descendants(matching: .any)
            .matching(NSPredicate(format: "value == %@", "hell"))
            .firstMatch
        XCTAssertTrue(trimmed.waitForExistence(timeout: 10), "backspace had no effect\n" + app.debugDescription)
    }

    private func attach(_ app: XCUIApplication, name: String) {
        let shot = XCTAttachment(screenshot: app.screenshot())
        shot.name = name
        shot.lifetime = .keepAlways
        add(shot)
        let tree = XCTAttachment(string: app.debugDescription)
        tree.name = "\(name)-tree"
        tree.lifetime = .keepAlways
        add(tree)
    }
}
