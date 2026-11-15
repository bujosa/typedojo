import AppKit
let size = 1024
let image = NSImage(size: NSSize(width: size, height: size))
image.lockFocus()
NSColor(calibratedRed: 0.07, green: 0.10, blue: 0.16, alpha: 1).setFill()
NSBezierPath(roundedRect: NSRect(x: 48, y: 48, width: 928, height: 928), xRadius: 210, yRadius: 210).fill()
NSColor(calibratedRed: 0.36, green: 0.65, blue: 1, alpha: 1).setStroke()
let ring = NSBezierPath(roundedRect: NSRect(x: 100, y: 100, width: 824, height: 824), xRadius: 164, yRadius: 164)
ring.lineWidth = 12
ring.stroke()
let attrs: [NSAttributedString.Key: Any] = [.font: NSFont.monospacedSystemFont(ofSize: 400, weight: .bold), .foregroundColor: NSColor.white]
let text = "T_" as NSString
let textSize = text.size(withAttributes: attrs)
text.draw(at: NSPoint(x: (1024 - textSize.width)/2, y: (1024 - textSize.height)/2 + 12), withAttributes: attrs)
image.unlockFocus()
if let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff), let png = bitmap.representation(using: .png, properties: [:]) {
    try png.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
} else {
    throw NSError(domain: "TypeDojo icon", code: 1)
}
