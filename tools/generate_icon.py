from pathlib import Path
from PIL import Image, ImageDraw

out_dir = Path("src-tauri/icons")
out_dir.mkdir(parents=True, exist_ok=True)

sizes = [16, 24, 32, 48, 64, 128, 256, 512]
images = []
for size in sizes:
    image = Image.new("RGBA", (size, size), (8, 8, 12, 255))
    draw = ImageDraw.Draw(image)
    margin = max(2, size // 8)
    draw.rounded_rectangle(
        (margin, margin, size - margin, size - margin),
        radius=max(2, size // 6),
        outline=(167, 139, 250, 255),
        width=max(1, size // 16),
    )
    draw.line(
        (size * 0.28, size * 0.68, size * 0.50, size * 0.38, size * 0.72, size * 0.68),
        fill=(192, 132, 252, 255),
        width=max(1, size // 10),
        joint="curve",
    )
    images.append(image)

# Native Windows icon.
images[-1].save(out_dir / "icon.ico", format="ICO", sizes=[(s, s) for s in sizes])
# Square PNG assets for Linux AppImage and other desktop bundlers.
images[-1].save(out_dir / "icon.png", format="PNG")
images[-1].save(out_dir / "icon-512.png", format="PNG")
print(f"Generated {out_dir / 'icon.ico'}, {out_dir / 'icon.png'}, and {out_dir / 'icon-512.png'}")
