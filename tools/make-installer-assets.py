"""Make the installer assets from the logo in the repository root.

The `.ico` and the wizard banner are derived rather than kept as separate files,
so they cannot drift from the logo and so the step that produces them is visible
instead of being folklore. Run this after changing `platipus.png`:

    python tools/make-installer-assets.py

It writes `installer/platipus.ico` and `installer/wizard.bmp`, which are the two
paths `installer/platipus.iss` names.
"""

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "platipus.png"
OUT_DIR = ROOT / "installer"

# The sizes Windows asks for in different places: the shell shows 16, the taskbar
# 24, an Explorer large icon 32 or 48, and the installer itself 256.
ICON_SIZES = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

# Inno Setup scales the wizard bitmap to 164 pixels wide on its own, but writing
# it at that size keeps the file small and avoids the resampler.
BANNER_SIZE = (164, 164)


def square(image: Image.Image) -> Image.Image:
    """Centre the logo on a transparent square.

    The logo is a roundel on transparency, so padding it to a square rather than
    stretching it keeps its proportions and leaves no grey corners.
    """
    side = max(image.size)
    canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas.paste(image, ((side - image.width) // 2, (side - image.height) // 2), image)
    return canvas


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"missing {SOURCE}")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    mark = square(Image.open(SOURCE).convert("RGBA"))
    mark.thumbnail((256, 256), Image.LANCZOS)

    icon = OUT_DIR / "platipus.ico"
    mark.save(icon, format="ICO", sizes=ICON_SIZES, bitmap_format="png")
    print(f"wrote {icon.relative_to(ROOT)}")

    banner = OUT_DIR / "wizard.bmp"
    mark.resize(BANNER_SIZE, Image.LANCZOS).save(banner, format="BMP")
    print(f"wrote {banner.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
