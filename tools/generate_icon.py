from pathlib import Path
from PIL import Image, ImageDraw

out = Path("src-tauri/icons/icon.ico")
out.parent.mkdir(parents=True, exist_ok=True)

sizes = [16, 24, 32, 48, 64, 128, 256]
images = []
for size in sizes:
    image = Image.new("RGBA", (size, size), (8, 8, 12, 255))
    draw = ImageDraw.Draw(image)
    margin = max(1, size // 8)
    draw.rounded_rectangle((margin, margin, size - margin, size - margin), radius=max(2, size // 6), outline=(167, 139, 250, 255), width=max(1, size // 16))
    draw.line((size * .28, size * .68, size * .50, size * .38, size * .72, size * .68), fill=(192, 132, 252, 255), width=max(1, size // 10), joint="curve")
    images.append(image)

images[-1].save(out, format="ICO", sizes=[(s, s) for s in sizes])
print(f"Generated {out}")
