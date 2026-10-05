from pathlib import Path
from PIL import Image, ImageDraw

BG = "#11151C"
BLUE = "#3B82F6"
GRAY = "#6B7685"
WHITE = "#E6EDF3"

LINES = [
    (172, 72, 172, 184, 18, BLUE),
    (148, 72, 196, 72, 18, BLUE),
    (148, 184, 196, 184, 18, BLUE),
    (72, 100, 124, 100, 16, GRAY),
    (52, 128, 124, 128, 16, WHITE),
    (84, 156, 124, 156, 16, GRAY),
]
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]
SS = 8


def render(size):
    k = size * SS / 256
    img = Image.new("RGBA", (size * SS, size * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([0, 0, size * SS - 1, size * SS - 1], radius=56 * k, fill=BG)
    for x1, y1, x2, y2, w, c in LINES:
        h = w / 2
        d.rounded_rectangle(
            [(min(x1, x2) - h) * k, (min(y1, y2) - h) * k, (max(x1, x2) + h) * k, (max(y1, y2) + h) * k],
            radius=h * k,
            fill=c,
        )
    return img.resize((size, size), Image.LANCZOS)


def main():
    out = Path(__file__).parent
    imgs = [render(s) for s in SIZES]
    imgs[-1].save(out / "icon.png")
    imgs[-1].save(out / "icon.ico", sizes=[(s, s) for s in SIZES], append_images=imgs[:-1])


if __name__ == "__main__":
    main()
