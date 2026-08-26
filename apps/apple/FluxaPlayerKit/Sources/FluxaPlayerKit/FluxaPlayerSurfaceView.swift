import QuartzCore

#if canImport(UIKit)
import UIKit
public typealias FluxaPlatformView = UIView
#elseif canImport(AppKit)
import AppKit
public typealias FluxaPlatformView = NSView
#endif

public final class FluxaPlayerSurfaceView: FluxaPlatformView {
    private var hostedLayer: CALayer?

    public override init(frame: CGRect) {
        super.init(frame: frame)
        configure()
    }

    @available(*, unavailable)
    public required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    func host(_ layer: CALayer) {
        hostedLayer?.removeFromSuperlayer()
        hostedLayer = layer
        backingLayer?.addSublayer(layer)
        resizeHostedLayer()
    }

    func unhost() {
        hostedLayer?.removeFromSuperlayer()
        hostedLayer = nil
    }

    private func resizeHostedLayer() {
        guard let hostedLayer else { return }
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        hostedLayer.frame = bounds
        CATransaction.commit()
    }

    #if canImport(UIKit)
    private var backingLayer: CALayer? { layer }

    private func configure() {
        backgroundColor = .black
        isUserInteractionEnabled = false
    }

    public override func layoutSubviews() {
        super.layoutSubviews()
        resizeHostedLayer()
    }
    #elseif canImport(AppKit)
    private var backingLayer: CALayer? { layer }

    private func configure() {
        wantsLayer = true
        layer?.backgroundColor = NSColor.black.cgColor
    }

    public override func layout() {
        super.layout()
        resizeHostedLayer()
    }
    #endif
}
