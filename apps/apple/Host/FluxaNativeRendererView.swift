import FluxaRendererFFI
import GameController
import QuartzCore
import UIKit

@MainActor
final class FluxaNativeRendererView: UIView, UIKeyInput {
    override class var layerClass: AnyClass { CAMetalLayer.self }

    var onActions: ((String) -> Void)?
    let video = FluxaNativeVideo()

    nonisolated(unsafe) private let renderer: OpaquePointer
    private var displayLink: CADisplayLink?
    private var surfaceAttached = false
    private var activeTouch: UITouch?
    nonisolated(unsafe) private var controllerObservers: [NSObjectProtocol] = []

    private var metalLayer: CAMetalLayer { layer as! CAMetalLayer }
    private var accessNodes: [FluxaAccessNode] = []
    private var accessFocused: UInt64?
    private var accessSignature = ""
    private var accessTicks = 0

    override init(frame: CGRect) {
        let cacheDir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first?
            .appendingPathComponent("fluxa-artwork").path ?? ""
        renderer = fluxa_renderer_create(Float(UIScreen.main.scale), cacheDir)!
        super.init(frame: frame)
        #if !os(tvOS)
        isMultipleTouchEnabled = false
        #endif
        metalLayer.isOpaque = false
        metalLayer.pixelFormat = .bgra8Unorm_srgb
        fluxa_renderer_set_form_factor(
            renderer,
            traitCollection.userInterfaceIdiom == .tv ? "tv" : "mobile"
        )
        video.onMediaCommand = { [weak self] command, value in
            self?.pushAction("{\"type\":\"mediaCommand\",\"command\":\"\(command)\",\"value\":\(value)}")
        }
        installIndirectInput()
        observeControllers()
        observeAppActivity()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    deinit {
        controllerObservers.forEach { NotificationCenter.default.removeObserver($0) }
        fluxa_renderer_surface_destroyed(renderer)
        fluxa_renderer_destroy(renderer)
    }

    func startSession(dataDir: String) {
        fluxa_renderer_start_session(renderer, dataDir)
    }

    func pushAction(_ json: String) {
        fluxa_renderer_push_action(renderer, json)
    }

    func back() -> Bool {
        fluxa_renderer_back(renderer)
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        if window == nil {
            displayLink?.invalidate()
            displayLink = nil
            fluxa_renderer_surface_destroyed(renderer)
            surfaceAttached = false
            return
        }
        let link = CADisplayLink(target: self, selector: #selector(tick))
        link.preferredFrameRateRange = CAFrameRateRange(minimum: 60, maximum: 120, preferred: 120)
        link.add(to: .main, forMode: .common)
        displayLink = link
        setNeedsLayout()
        becomeFirstResponder()
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        let scale = window?.screen.scale ?? UIScreen.main.scale
        contentScaleFactor = scale
        metalLayer.contentsScale = scale
        let width = UInt32(max(1, (bounds.width * scale).rounded()))
        let height = UInt32(max(1, (bounds.height * scale).rounded()))
        metalLayer.drawableSize = CGSize(width: CGFloat(width), height: CGFloat(height))
        fluxa_renderer_set_safe_bottom_inset(renderer, Float(safeAreaInsets.bottom))
        fluxa_renderer_set_safe_top_inset(renderer, Float(safeAreaInsets.top))
        guard window != nil else { return }
        if surfaceAttached {
            fluxa_renderer_surface_changed(renderer, width, height)
        } else {
            fluxa_renderer_surface_created(
                renderer,
                Unmanaged.passRetained(metalLayer).toOpaque(),
                width,
                height
            )
            surfaceAttached = true
        }
    }

    @objc private func tick() {
        video.tick()
        fluxa_renderer_render(renderer)
        accessTicks += 1
        if accessTicks % 15 == 0, UIAccessibility.isVoiceOverRunning || UIAccessibility.isSwitchControlRunning {
            refreshAccessibility()
        }
        if let actions = take(fluxa_renderer_poll_actions(renderer)), actions != "[]" {
            onActions?(actions)
        }
        if let icon = take(fluxa_renderer_take_app_icon(renderer)) {
            applyAppIcon(icon)
        }
        if let link = take(fluxa_renderer_take_open_url(renderer)), let url = URL(string: link) {
            #if os(iOS)
            UIApplication.shared.open(url)
            #endif
        }
    }

    private func applyAppIcon(_ id: String) {
        #if os(iOS)
        let name = id == "ember" ? nil : "AppIcon-\(id)"
        guard UIApplication.shared.supportsAlternateIcons,
              UIApplication.shared.alternateIconName != name else { return }
        UIApplication.shared.setAlternateIconName(name)
        #endif
    }

    private func take(_ value: UnsafeMutablePointer<CChar>?) -> String? {
        guard let value else { return nil }
        defer { fluxa_renderer_string_free(value) }
        return String(cString: value)
    }

    private func pointer(_ phase: UInt32, _ touch: UITouch) {
        let point = touch.location(in: self)
        fluxa_renderer_pointer(renderer, phase, Float(point.x), Float(point.y))
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard activeTouch == nil, let touch = touches.first else { return }
        activeTouch = touch
        pointer(FLUXA_POINTER_DOWN, touch)
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = activeTouch, touches.contains(touch) else { return }
        pointer(FLUXA_POINTER_MOVE, touch)
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = activeTouch, touches.contains(touch) else { return }
        pointer(FLUXA_POINTER_UP, touch)
        activeTouch = nil
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        touchesEnded(touches, with: event)
    }

    private func installIndirectInput() {
        #if os(iOS)
        let scroll = UIPanGestureRecognizer(target: self, action: #selector(indirectScroll))
        scroll.allowedScrollTypesMask = .all
        scroll.allowedTouchTypes = []
        addGestureRecognizer(scroll)
        addGestureRecognizer(UIHoverGestureRecognizer(target: self, action: #selector(hover)))
        #endif
    }

    @objc private func indirectScroll(_ gesture: UIPanGestureRecognizer) {
        let delta = gesture.translation(in: self)
        gesture.setTranslation(.zero, in: self)
        fluxa_renderer_scroll(renderer, Float(-delta.y))
    }

    #if os(iOS)
    @objc private func hover(_ gesture: UIHoverGestureRecognizer) {
        let point = gesture.location(in: self)
        fluxa_renderer_pointer(renderer, FLUXA_POINTER_MOVE, Float(point.x), Float(point.y))
    }
    #endif

    override var canBecomeFirstResponder: Bool { true }

    override var isAccessibilityElement: Bool {
        get { false }
        set {}
    }

    override var accessibilityElements: [Any]? {
        get {
            if accessNodes.isEmpty { refreshAccessibility() }
            return accessNodes.map { element(for: $0) }
        }
        set {}
    }

    private var focusedAccessNode: FluxaAccessNode? {
        accessNodes.first { $0.id == accessFocused && $0.isTextInput }
    }

    var keyboardType: UIKeyboardType {
        get {
            switch focusedAccessNode?.role {
            case "url_input": return .URL
            case "search_input": return .webSearch
            case "number_input": return .numberPad
            default: return .default
            }
        }
        set {}
    }

    var isSecureTextEntry: Bool {
        get { focusedAccessNode?.role == "password_input" }
        set {}
    }

    var returnKeyType: UIReturnKeyType {
        get {
            switch focusedAccessNode?.role {
            case "search_input": return .search
            case "multiline_input": return .default
            default: return .done
            }
        }
        set {}
    }

    var autocorrectionType: UITextAutocorrectionType {
        get { .no }
        set {}
    }

    var autocapitalizationType: UITextAutocapitalizationType {
        get { .none }
        set {}
    }

    private func refreshAccessibility() {
        guard let json = take(fluxa_renderer_accessibility_tree(renderer)), json != accessSignature,
              let data = json.data(using: .utf8),
              let tree = try? JSONDecoder().decode(FluxaAccessTree.self, from: data) else { return }
        let structureChanged = tree.nodes.map(\.id) != accessNodes.map(\.id)
        let focusChanged = tree.focused != accessFocused
        accessSignature = json
        accessNodes = tree.nodes
        accessFocused = tree.focused
        guard UIAccessibility.isVoiceOverRunning else { return }
        if structureChanged {
            UIAccessibility.post(notification: .layoutChanged, argument: nil)
        } else if focusChanged, let focused = tree.focused,
                  let node = accessNodes.first(where: { $0.id == focused }) {
            UIAccessibility.post(notification: .layoutChanged, argument: element(for: node))
        }
    }

    private func element(for node: FluxaAccessNode) -> UIAccessibilityElement {
        let element = FluxaAccessElement(accessibilityContainer: self)
        element.node = node
        element.owner = self
        element.accessibilityLabel = node.label
        element.accessibilityValue = node.value
        element.accessibilityFrameInContainerSpace = CGRect(
            x: node.bounds[0], y: node.bounds[1], width: node.bounds[2], height: node.bounds[3]
        )
        var traits: UIAccessibilityTraits = node.isTextInput ? [] : .button
        if node.role == "text" { traits = .staticText }
        if node.role == "heading" { traits = [.staticText, .header] }
        if node.role == "search_input" { traits.insert(.searchField) }
        if node.role == "slider" { traits = .adjustable }
        if node.selected || node.checked == true { traits.insert(.selected) }
        element.accessibilityTraits = traits
        if node.role == "switch" {
            element.accessibilityValue = node.checked == true ? "1" : "0"
        }
        return element
    }

    fileprivate func activate(_ node: FluxaAccessNode) {
        if node.isStatic { return }
        fluxa_renderer_focus_node(renderer, node.id)
        if node.isTextInput {
            becomeFirstResponder()
            reloadInputViews()
        } else {
            fluxa_renderer_activate_node(renderer, node.id)
        }
    }

    fileprivate func seek(by seconds: Double, from node: FluxaAccessNode) {
        guard let range = node.range else { return }
        let target = min(max(range[2] + seconds, range[0]), range[1])
        pushAction("{\"type\":\"mediaCommand\",\"command\":\"seekTo\",\"value\":\(target)}")
    }

    var hasText: Bool { true }

    func insertText(_ text: String) {
        if text == "\n" {
            key(FLUXA_KEY_ENTER)
        } else {
            fluxa_renderer_text_input(renderer, text)
        }
    }

    func deleteBackward() {
        key(FLUXA_KEY_BACKSPACE)
    }

    private func key(_ code: UInt32) {
        fluxa_renderer_key_down(renderer, code)
    }

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        var handled = false
        for press in presses {
            guard let code = keyCode(for: press) else { continue }
            key(code)
            handled = true
        }
        if !handled {
            super.pressesBegan(presses, with: event)
        }
    }

    private func keyCode(for press: UIPress) -> UInt32? {
        switch press.type {
        case .upArrow: return FLUXA_GAMEPAD_DPAD_UP
        case .downArrow: return FLUXA_GAMEPAD_DPAD_DOWN
        case .leftArrow: return FLUXA_GAMEPAD_DPAD_LEFT
        case .rightArrow: return FLUXA_GAMEPAD_DPAD_RIGHT
        case .select: return FLUXA_KEY_ENTER
        case .menu: return FLUXA_KEY_BACK
        case .playPause: return FLUXA_GAMEPAD_START
        default: break
        }
        guard let key = press.key else { return nil }
        switch key.keyCode {
        case .keyboardUpArrow: return FLUXA_KEY_UP
        case .keyboardDownArrow: return FLUXA_KEY_DOWN
        case .keyboardLeftArrow: return FLUXA_KEY_LEFT
        case .keyboardRightArrow: return FLUXA_KEY_RIGHT
        case .keyboardReturnOrEnter, .keypadEnter: return FLUXA_KEY_ENTER
        case .keyboardEscape: return FLUXA_KEY_ESCAPE
        case .keyboardTab: return key.modifierFlags.contains(.shift) ? FLUXA_KEY_SHIFT_TAB : FLUXA_KEY_TAB
        default: return nil
        }
    }

    private func observeControllers() {
        GCController.controllers().forEach(bind)
        controllerObservers.append(
            NotificationCenter.default.addObserver(
                forName: .GCControllerDidConnect,
                object: nil,
                queue: .main
            ) { [weak self] note in
                guard let controller = note.object as? GCController else { return }
                MainActor.assumeIsolated { self?.bind(controller) }
            }
        )
    }

    private func observeAppActivity() {
        for (name, focused) in [
            (UIApplication.didBecomeActiveNotification, true),
            (UIApplication.willResignActiveNotification, false),
        ] {
            controllerObservers.append(
                NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) {
                    [weak self] _ in
                    MainActor.assumeIsolated {
                        guard let self else { return }
                        fluxa_renderer_set_window_focused(self.renderer, focused)
                    }
                }
            )
        }
    }

    private func bind(_ controller: GCController) {
        guard let pad = controller.extendedGamepad else { return }
        let buttons: [(GCControllerButtonInput, UInt32)] = [
            (pad.buttonA, FLUXA_GAMEPAD_SOUTH),
            (pad.buttonB, FLUXA_GAMEPAD_EAST),
            (pad.buttonX, FLUXA_GAMEPAD_WEST),
            (pad.buttonY, FLUXA_GAMEPAD_NORTH),
            (pad.dpad.up, FLUXA_GAMEPAD_DPAD_UP),
            (pad.dpad.down, FLUXA_GAMEPAD_DPAD_DOWN),
            (pad.dpad.left, FLUXA_GAMEPAD_DPAD_LEFT),
            (pad.dpad.right, FLUXA_GAMEPAD_DPAD_RIGHT),
            (pad.buttonMenu, FLUXA_GAMEPAD_START),
        ]
        for (button, code) in buttons {
            button.pressedChangedHandler = { [weak self] _, _, pressed in
                guard pressed else { return }
                MainActor.assumeIsolated { self?.key(code) }
            }
        }
        pad.buttonOptions?.pressedChangedHandler = { [weak self] _, _, pressed in
            guard pressed else { return }
            MainActor.assumeIsolated { self?.key(FLUXA_GAMEPAD_SELECT) }
        }
    }
}

struct FluxaAccessNode: Decodable {
    let id: UInt64
    let role: String
    let label: String
    let value: String?
    let checked: Bool?
    let selected: Bool
    let bounds: [Double]
    let range: [Double]?

    var isTextInput: Bool { role.hasSuffix("_input") }
    var isStatic: Bool { role == "text" || role == "heading" }
}

struct FluxaAccessTree: Decodable {
    let nodes: [FluxaAccessNode]
    let focused: UInt64?
    let caret: [Double]?
    let selection: [Int]?
}

final class FluxaAccessElement: UIAccessibilityElement {
    var node: FluxaAccessNode?
    weak var owner: FluxaNativeRendererView?

    override func accessibilityActivate() -> Bool {
        guard let node, let owner else { return false }
        MainActor.assumeIsolated { owner.activate(node) }
        return true
    }

    override func accessibilityIncrement() {
        guard let node, let owner else { return }
        MainActor.assumeIsolated { owner.seek(by: 10, from: node) }
    }

    override func accessibilityDecrement() {
        guard let node, let owner else { return }
        MainActor.assumeIsolated { owner.seek(by: -10, from: node) }
    }
}
