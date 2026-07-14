#!/bin/bash
# संस्कृता Shorts #2 & #3 — Gītā meter demo + Kāraka feature (one run, both narrations)
# Run:  bash make_voice_v6.sh
set -e
cd "$(dirname "$0")"
mkdir -p v4
python3 -m pip install --user --quiet edge-tts

VOICE="hi-IN-SwaraNeural"
speak() {
    python3 -m edge_tts --voice "$VOICE" --rate="-4%" --text "$2" --write-media "v4/$1.mp3"
    echo "✓ v4/$1.mp3"
}

# --- Short: Gītā meter demo ---
speak g1 "क्या कोई प्रोग्रामिंग भाषा भगवद्गीता का छन्द पहचान सकती है? संस्कृता कर सकती है।"
speak g2 "यह रहा गीता का पहला श्लोक। और यह रहा — सिर्फ़ एक पंक्ति का code।"
speak g3 "उत्तर: अनुष्टुभ्! बत्तीस अक्षर — बिल्कुल सही।"
speak g4 "संस्कृत की शक्ति, अब code में। Link नीचे है। जयतु संस्कृतम्!"

# --- Short: Kāraka arguments ---
speak k1 "हर प्रोग्रामिंग भाषा में arguments का order fix होता है। ग़लत order — ग़लत जवाब।"
speak k2 "लेकिन संस्कृत में शब्दों का क्रम मायने नहीं रखता — क्योंकि कारक ख़ुद भूमिका बताते हैं।"
speak k3 "संस्कृता में भी! कर्म — यानी क्या। सम्प्रदान — यानी किसको। करण — यानी किसके द्वारा।"
speak k4 "अब order उलट दीजिए — जवाब वही का वही! पाणिनि का व्याकरण, code में ज़िंदा।"
speak k5 "दुनिया की किसी और programming भाषा में यह नहीं है। जयतु संस्कृतम्!"

echo ""
echo "सिद्धम् ✓ Both shorts narrated (video/v4/). Tell Claude!"
