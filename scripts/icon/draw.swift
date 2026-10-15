// tabflip's icon, drawn with AppKit at every size the bundle and the
// packager want, each on its own pixels:
//
//   swift scripts/icon/draw.swift <out-dir>
//
// A stack of window rows on a deep blue square, the second row lit, as
// the switcher looks mid-flip. The square is full and opaque to its
// corners: macOS 26 masks every app icon to its own rounded shape, and
// anything transparent shows as a grey border (emaki's docs/platform.md).
import AppKit

let out = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "."

func draw(_ px: Int) -> Data {
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: px, pixelsHigh: px, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    let s = CGFloat(px)
    let ground = NSGradient(starting: NSColor(srgbRed: 0.16, green: 0.30, blue: 0.78, alpha: 1), ending: NSColor(srgbRed: 0.05, green: 0.10, blue: 0.32, alpha: 1))!
    ground.draw(in: NSRect(x: 0, y: 0, width: s, height: s), angle: -90)

    // The panel, and four rows in it; the second is the one picked.
    let panel = NSRect(x: s * 0.17, y: s * 0.19, width: s * 0.66, height: s * 0.62)
    NSColor(white: 0.08, alpha: 0.55).setFill()
    NSBezierPath(roundedRect: panel, xRadius: s * 0.07, yRadius: s * 0.07).fill()
    let rowH = panel.height * 0.2, gap = (panel.height - 4 * rowH) / 5
    for i in 0..<4 {
        let y = panel.maxY - gap - CGFloat(i + 1) * rowH - CGFloat(i) * gap
        let row = NSRect(x: panel.minX + gap, y: y, width: panel.width - 2 * gap, height: rowH)
        if i == 1 {
            NSColor(srgbRed: 0.04, green: 0.52, blue: 1.0, alpha: 1).setFill()
            NSBezierPath(roundedRect: row, xRadius: rowH * 0.3, yRadius: rowH * 0.3).fill()
        }
        let dot = NSRect(x: row.minX + rowH * 0.25, y: row.minY + rowH * 0.25, width: rowH * 0.5, height: rowH * 0.5)
        NSColor(white: 1, alpha: i == 1 ? 1 : 0.75).setFill()
        NSBezierPath(roundedRect: dot, xRadius: rowH * 0.12, yRadius: rowH * 0.12).fill()
        let words = NSRect(x: dot.maxX + rowH * 0.3, y: row.midY - rowH * 0.1, width: row.width * (i == 1 ? 0.5 : 0.42 - CGFloat(i % 2) * 0.1), height: rowH * 0.2)
        NSBezierPath(roundedRect: words, xRadius: rowH * 0.1, yRadius: rowH * 0.1).fill()
    }
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])!
}

let fm = FileManager.default
let iconset = "\(out)/icon.iconset"
try? fm.removeItem(atPath: iconset)
try! fm.createDirectory(atPath: iconset, withIntermediateDirectories: true)
for (name, px) in [("16x16", 16), ("16x16@2x", 32), ("32x32", 32), ("32x32@2x", 64), ("128x128", 128), ("128x128@2x", 256), ("256x256", 256), ("256x256@2x", 512), ("512x512", 512), ("512x512@2x", 1024)] {
    try! draw(px).write(to: URL(fileURLWithPath: "\(iconset)/icon_\(name).png"))
}
for px in [32, 128, 256, 512, 1024] {
    try! draw(px).write(to: URL(fileURLWithPath: "\(out)/icon-\(px).png"))
}
