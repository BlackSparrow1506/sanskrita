#!/usr/bin/env python3
"""
Video 2.6 — "लघु-गुरु is binary"  (1080×1920 vertical, ~60s)

Pure-information cut: explains syllable weight → two states → one bit →
Piṅgala's enumeration → अनुष्टुभ् as a 32-bit pattern.

All syllable/weight data below is REAL output from the संस्कृतम् module
(verified with sanskrita.py, 26 July 2026) — nothing here is hand-faked.

Run:
    bash make_voice_v10.sh          # writes v8/line1..6.mp3   (needs network)
    python3 gen_v10_chandas.py      # builds short_chandas_binary.mp4

Without the mp3s present it still renders, using the fallback timings
below, so you can preview the visuals before recording narration.
"""

import os
import subprocess
import shutil
import unicodedata
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
VOICE_DIR = os.path.join(HERE, "v8")
# frames go to fast local scratch, not the project folder (hundreds of PNGs)
FRAME_DIR = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_frames_v10")
OUT = os.path.join(HERE, "short_chandas_binary.mp4")
FONT_PATH = os.path.join(REPO, "NotoSansDevanagari-VariableFont_wdth,wght.ttf")

W, H = 1080, 1920
FPS = 12

# Palette sampled directly from the published videos (short_gita_meter_v2.mp4,
# short_karaka_v2.mp4) so this cut matches videos 1–7 exactly.
BG = (0x1a, 0x17, 0x13)     # warm near-black background
PANEL = (0x10, 0x0c, 0x09)  # code/box fill, darker than the bg
FG = (0xf0, 0xeb, 0xe6)     # body text, warm white
ACCENT = (0xec, 0xa6, 0x3c)  # saffron — headings and गुरु
DIM = (0x8a, 0x86, 0x82)    # secondary text
RULE = (0x2e, 0x28, 0x22)

GURU = ACCENT               # heavy  = saffron
LAGHU = FG                  # light  = warm white
BRAND = "ॐ संस्कृता"

# fallback seconds per line if the mp3s aren't there yet
FALLBACK = [7.5, 9.5, 7.0, 11.5, 12.0, 11.0]


# ---------------------------------------------------------------- verified data
# from:  वद("अक्षराणि:", सं.अक्षराणि("धर्मक्षेत्रे कुरुक्षेत्रे"))
PADA1 = ["ध", "र्म", "क्षे", "त्रे", "कु", "रु", "क्षे", "त्रे"]
# from:  वद("मात्राः:", सं.मात्राः("धर्मक्षेत्रे कुरुक्षेत्रे"))
PADA1_W = ["ग", "ग", "ग", "ग", "ल", "ग", "ग", "ग"]

# from:  वद("मात्राः:", सं.मात्राः(श्लोकः))   — all 32
FULL_W = ["ग", "ग", "ग", "ग", "ल", "ग", "ग", "ग",
          "ल", "ल", "ग", "ग", "ल", "ग", "ल", "ग",
          "ग", "ल", "ग", "ग", "ल", "ग", "ग", "ल",
          "ल", "ल", "ग", "ल", "ल", "ग", "ल", "ल"]

VERSE_LINES = ["धर्मक्षेत्रे कुरुक्षेत्रे", "समवेता युयुत्सवः",
               "मामकाः पाण्डवाश्चैव", "किमकुर्वत सञ्जय"]


def nfc(s):
    return unicodedata.normalize("NFC", s)


_font_cache = {}


def font(size):
    if size not in _font_cache:
        _font_cache[size] = ImageFont.truetype(FONT_PATH, size)
    return _font_cache[size]


def text(d, xy, s, size, fill=FG, anchor="mm", weight=None):
    f = font(size)
    if weight is not None:
        try:
            f.set_variation_by_axes([100.0, float(weight)])
        except Exception:
            pass
    d.text(xy, nfc(s), font=f, fill=fill, anchor=anchor,
           language="hi", features=["dlig", "akhn", "rphf", "blwf", "half"])


def width_of(d, s, size):
    return d.textlength(nfc(s), font=font(size), language="hi")


def ease(t):
    """smoothstep"""
    t = max(0.0, min(1.0, t))
    return t * t * (3 - 2 * t)


def new_frame():
    """Background + the persistent ॐ संस्कृता brand line, as in videos 1–7."""
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    text(d, (W // 2, 100), BRAND, 46, ACCENT)
    return img, d


def panel(d, x0, y0, x1, y1, label=None):
    """Rounded dark box — the house 'code panel'."""
    d.rounded_rectangle([x0, y0, x1, y1], radius=28, fill=PANEL,
                        outline=RULE, width=2)
    if label:
        text(d, ((x0 + x1) // 2, y0 + 44), label, 34, DIM)


def caption(d, s, size=40):
    """small persistent label at the very bottom"""
    if s:
        text(d, (W // 2, H - 130), s, size, DIM)


# ------------------------------------------------------------------- segments

def seg1(t, d):
    """Every syllable is one of two things."""
    text(d, (W // 2, 430), "हर अक्षर", 92, ACCENT)

    # two boxes drop in
    a = ease((t - 0.15) / 0.35)
    b = ease((t - 0.35) / 0.35)
    for i, (label, colour, prog) in enumerate(
            [("लघु", LAGHU, a), ("गुरु", GURU, b)]):
        if prog <= 0:
            continue
        cx = W // 2 + (-250 if i == 0 else 250)
        cy = 820 - int((1 - prog) * 60)
        r = 190
        panel(d, cx - r, cy - r, cx + r, cy + r)
        text(d, (cx, cy - 30), label, 104, colour)
        text(d, (cx, cy + 78), "light" if i == 0 else "heavy", 40, DIM)

    if t > 0.72:
        text(d, (W // 2, 1300), "तीसरा विकल्प नहीं", 62, FG)
        text(d, (W // 2, 1390), "no third option", 42, DIM)
    caption(d, "छन्दःशास्त्र")


def seg2(t, d):
    """The rule for guru."""
    text(d, (W // 2, 330), "गुरु कब?", 86, GURU)

    rows = [("दीर्घ स्वर", "आ  ई  ऊ  ए  ऐ  ओ  औ"),
            ("संयुक्त व्यंजन से पहले", "…क्ष   …त्र   …र्म")]
    for i, (head, ex) in enumerate(rows):
        p = ease((t - 0.12 - i * 0.26) / 0.3)
        if p <= 0:
            continue
        y = 620 + i * 300
        d.line([(160, y - 110), (160 + int(760 * p), y - 110)],
               fill=RULE, width=3)
        text(d, (W // 2, y), head, 58, FG)
        text(d, (W // 2, y + 88), ex, 62, GURU)

    if t > 0.78:
        text(d, (W // 2, 1400), "शेष सर्वम् — लघु", 62, LAGHU)
    caption(d, "नियम")


def seg3(t, d):
    """Two states = one bit."""
    text(d, (W // 2, 400), "दो अवस्थाएँ", 78, ACCENT)

    p = ease((t - 0.2) / 0.4)
    if p > 0:
        for i, (syl, bit, colour) in enumerate(
                [("ल", "०", LAGHU), ("ग", "१", GURU)]):
            y = 700 + i * 220
            text(d, (W // 2 - 110, y), syl, 150, colour)
            d.line([(W // 2 - 20, y), (W // 2 + 60, y)], fill=DIM, width=4)
            text(d, (W // 2 + 160, y), bit, 130, DIM)

    if t > 0.6:
        text(d, (W // 2, 1240), "= १ bit", 84, FG)
        text(d, (W // 2, 1370), "श्लोकः = bit-string", 58, DIM)
    caption(d, "")


def seg4(t, d):
    """Piṅgala enumerated the combinations."""
    text(d, (W // 2, 300), "पिङ्गलः", 84, ACCENT)
    text(d, (W // 2, 385), "छन्दःशास्त्र", 46, DIM)

    # enumerate 3-syllable combinations, one row at a time
    combos = [("ल", "ल", "ल"), ("ग", "ल", "ल"), ("ल", "ग", "ल"),
              ("ग", "ग", "ल"), ("ल", "ल", "ग"), ("ग", "ल", "ग"),
              ("ल", "ग", "ग"), ("ग", "ग", "ग")]
    shown = int(ease((t - 0.08) / 0.6) * len(combos) + 0.001)
    for i in range(shown):
        y = 620 + i * 108
        for j, c in enumerate(combos[i]):
            x = W // 2 - 220 + j * 130
            text(d, (x, y), c, 66, GURU if c == "ग" else LAGHU)
        text(d, (W // 2 + 220, y),
             "".join("1" if c == "ग" else "0" for c in combos[i]),
             52, DIM)

    if t > 0.8:
        text(d, (W // 2, 1560), "binary enumeration", 58, FG)
        text(d, (W // 2, 1645), "computers से ~२००० वर्ष पूर्व", 44, DIM)
    caption(d, "")


def seg5(t, d):
    """First pāda, syllable by syllable, weights revealed."""
    text(d, (W // 2, 250), "भगवद्गीता १.१", 52, DIM)
    text(d, (W // 2, 370), "धर्मक्षेत्रे कुरुक्षेत्रे", 76, ACCENT)

    n = len(PADA1)
    revealed = ease((t - 0.1) / 0.62) * n
    y_syl, y_w = 700, 880

    # lay the 8 syllables out on two rows of four (vertical format)
    for i, syl in enumerate(PADA1):
        row, col = divmod(i, 4)
        x = 180 + col * 240
        yy = y_syl + row * 350
        on = revealed > i
        text(d, (x, yy), syl, 82, FG if on else RULE)
        if on:
            w = PADA1_W[i]
            colour = GURU if w == "ग" else LAGHU
            text(d, (x, yy + 118), w, 68, colour)
            d.rounded_rectangle([x - 62, yy + 168, x + 62, yy + 186],
                                radius=9, fill=colour)

    if t > 0.8:
        text(d, (W // 2, 1560), "ग ग ग ग  ल ग ग ग", 56, FG)
    caption(d, "सं.मात्राः()")


def seg6(t, d):
    """All 32 as a grid → अनुष्टुभ्."""
    text(d, (W // 2, 250), "पूर्णः श्लोकः", 62, ACCENT)

    # 4 pādas × 8 syllables
    p = ease(t / 0.5)
    cell, gap = 96, 18
    grid_w = 8 * cell + 7 * gap
    x0 = (W - grid_w) // 2
    y0 = 460
    for i, w in enumerate(FULL_W):
        row, col = divmod(i, 8)
        if p * 32 <= i:
            continue
        x = x0 + col * (cell + gap)
        y = y0 + row * (cell + gap)
        colour = GURU if w == "ग" else LAGHU
        d.rounded_rectangle([x, y, x + cell, y + cell], radius=14, fill=colour)
        text(d, (x + cell // 2, y + cell // 2), w, 52, BG)

    if t > 0.45:
        for i, line in enumerate(VERSE_LINES):
            text(d, (W // 2, 980 + i * 64), line, 42, DIM)

    if t > 0.62:
        d.line([(220, 1268), (860, 1268)], fill=RULE, width=3)
        text(d, (W // 2, 1360), "३२ अक्षराणि", 62, FG)
        text(d, (W // 2, 1478), "अनुष्टुभ्", 110, GURU)

    if t > 0.85:
        text(d, (W // 2, 1618), "छन्द-पहचान = pattern matching", 46, FG)
        text(d, (W // 2, 1686), "on a bit-string", 42, DIM)
    caption(d, "सं.छन्दः()")


SEGMENTS = [seg1, seg2, seg3, seg4, seg5, seg6]


# ---------------------------------------------------------------------- build

def duration_of(path):
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "csv=p=0", path],
        capture_output=True, text=True).stdout.strip()
    return float(out)


def main():
    if not os.path.exists(FONT_PATH):
        raise SystemExit(f"font not found: {FONT_PATH}")

    voices, durations = [], []
    for i in range(1, 7):
        mp3 = os.path.join(VOICE_DIR, f"line{i}.mp3")
        if os.path.exists(mp3):
            voices.append(mp3)
            durations.append(duration_of(mp3) + 0.45)   # small tail pause
        else:
            voices.append(None)
            durations.append(FALLBACK[i - 1])

    have_voice = all(v for v in voices)
    print("narration:", "found" if have_voice
          else "MISSING — rendering silent preview")

    if os.path.isdir(FRAME_DIR):
        shutil.rmtree(FRAME_DIR)
    os.makedirs(FRAME_DIR)

    idx = 0
    for seg, dur in zip(SEGMENTS, durations):
        n = max(1, int(round(dur * FPS)))
        for k in range(n):
            img, d = new_frame()
            seg(k / max(1, n - 1), d)
            img.save(os.path.join(FRAME_DIR, f"f{idx:05d}.png"))
            idx += 1
        print(f"  {seg.__name__}: {dur:.2f}s  ({n} frames)")
    print(f"total {idx} frames = {idx / FPS:.1f}s")

    silent = os.path.join(HERE, "_silent_v10.mp4")
    subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(FPS),
         "-i", os.path.join(FRAME_DIR, "f%05d.png"),
         "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", silent],
        check=True)

    if have_voice:
        concat = os.path.join(HERE, "_voice_v10.txt")
        with open(concat, "w") as fh:
            for v in voices:
                fh.write(f"file '{v}'\n")
        audio = os.path.join(HERE, "_voice_v10.mp3")
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat",
                        "-safe", "0", "-i", concat, "-c", "copy", audio],
                       check=True)
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", silent,
                        "-i", audio, "-c:v", "copy", "-c:a", "aac",
                        "-shortest", OUT], check=True)
        os.remove(concat)
        print(f"\nसिद्धम् ✓  {OUT}")
    else:
        shutil.move(silent, OUT)
        print(f"\nsilent preview ✓  {OUT}"
              f"\n(run make_voice_v10.sh, then re-run this for the final cut)")

    shutil.rmtree(FRAME_DIR)


if __name__ == "__main__":
    main()
