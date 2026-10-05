import argparse
import tempfile
from pathlib import Path

from fontTools.merge import Merger
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

SUBSETS = ["archivo-latin.woff2", "archivo-latin-ext.woff2"]
WEIGHTS = {"regular": 400, "medium": 500, "semibold": 600, "bold": 700}


def instance(path, wght, wdth, out):
    font = TTFont(path)
    static = instantiateVariableFont(font, {"wght": wght, "wdth": wdth}, inplace=False)
    static.flavor = None
    static.save(out)


def build(src, dest, wdth, weights, tmp):
    for name in weights:
        parts = []
        for i, subset in enumerate(SUBSETS):
            part = Path(tmp) / f"{name}-{i}.ttf"
            instance(Path(src) / subset, WEIGHTS[name], wdth, part)
            parts.append(str(part))
        merged = Merger().merge(parts)
        merged.save(Path(dest) / f"archivo_{name}.ttf")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default="../fluxa-web/public/fonts")
    ap.add_argument("--dest", default="apps/android/app/src/main/res/font")
    ap.add_argument("--wdth", type=float, default=92)
    ap.add_argument("--weights", nargs="+", default=list(WEIGHTS))
    args = ap.parse_args()
    Path(args.dest).mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=Path.home() / ".cache/claude-tmp") as tmp:
        build(args.src, args.dest, args.wdth, args.weights, tmp)
