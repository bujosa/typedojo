import AppKit
// Original TypeDojo bracket seal. The same geometry is painted by src/design.rs.
let image = NSImage(size: NSSize(width: 1024, height: 1024))
image.lockFocus()
NSColor(calibratedRed: 0.051, green: 0.082, blue: 0.122, alpha: 1).setFill()
NSBezierPath(roundedRect: NSRect(x: 48, y: 48, width: 928, height: 928), xRadius: 210, yRadius: 210).fill()
NSColor(calibratedRed: 0.118, green: 0.310, blue: 0.569, alpha: 1).setFill()
NSBezierPath(roundedRect: NSRect(x: 154, y: 154, width: 716, height: 716), xRadius: 152, yRadius: 152).fill()
NSColor(calibratedRed: 0.949, green: 0.929, blue: 0.882, alpha: 1).setStroke()
func line(_ points: [(CGFloat, CGFloat)]) {
    let path = NSBezierPath()
    if let first = points.first { path.move(to: NSPoint(x: first.0, y: first.1)) }
    for point in points.dropFirst() { path.line(to: NSPoint(x: point.0, y: point.1)) }
    path.lineWidth = 42
    path.lineCapStyle = .square
    path.lineJoinStyle = .miter
    path.stroke()
}
line([(390, 708), (304, 708), (304, 316), (390, 316)])
line([(634, 708), (720, 708), (720, 316), (634, 316)])
line([(466, 366), (558, 658)])
image.unlockFocus()
if let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff), let png = bitmap.representation(using: .png, properties: [:]) {
    try png.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
} else {
    throw NSError(domain: "TypeDojo icon", code: 1)
}
