import XCTest

final class TypingTests: XCTestCase {
    func testTypingReachesWelcomeEmailField() throws {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)"]
        app.launch()

        let field = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == %@", "you@example.com"))
            .firstMatch
        XCTAssertTrue(field.waitForExistence(timeout: 90), app.debugDescription)
        field.tap()
        sleep(2)
        attach(app, name: "after-tap")

        if app.keyboards.count > 0 {
            for key in "hello" {
                app.keyboards.keys[String(key)].tap()
            }
        } else {
            app.typeText("hello")
        }
        sleep(2)
        attach(app, name: "after-typing")

        let typed = app.descendants(matching: .any)
            .matching(NSPredicate(format: "value == %@", "hello"))
            .firstMatch
        XCTAssertTrue(typed.waitForExistence(timeout: 10), app.debugDescription)

        if app.keyboards.count > 0 {
            app.keyboards.keys["delete"].tap()
        }
        let trimmed = app.descendants(matching: .any)
            .matching(NSPredicate(format: "value == %@", "hell"))
            .firstMatch
        XCTAssertTrue(trimmed.waitForExistence(timeout: 10), app.debugDescription)
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
