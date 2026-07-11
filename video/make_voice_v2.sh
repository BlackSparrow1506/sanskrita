#!/bin/bash
# संस्कृता Video #1 — narration v2, realistic neural female voice (hi-IN Swara)
# Run:  bash make_voice_v2.sh
set -e
cd "$(dirname "$0")"

echo "installing edge-tts (one-time)…"
python3 -m pip install --user --quiet edge-tts

VOICE="hi-IN-SwaraNeural"
speak() {
    python3 -m edge_tts --voice "$VOICE" --rate="-4%" --text "$2" --write-media "line$1.mp3"
    echo "✓ line$1.mp3"
}

speak 1 "क्या आप जानते हैं? यह प्रोग्रामिंग भाषा पूरी तरह संस्कृत में लिखी जाती है।"
speak 2 "इसका नाम है संस्कृता। देखिए — वद का अर्थ है: बोलो।"
speak 3 "चर, लूप, फलन — सब पाणिनि के व्याकरण के साथ।"
speak 4 "शून्य दशमलव एक, जोड़ें शून्य दशमलव दो — उत्तर: ठीक शून्य दशमलव तीन। पाइथन यह ग़लत करता है!"
speak 5 "और यह देखिए — इसने अभी भगवद्गीता के पहले श्लोक का छन्द पहचाना: अनुष्टुभ्!"
speak 6 "संस्कृता। ओपन सोर्स। लिंक नीचे है। जयतु संस्कृतम्!"

echo ""
echo "सिद्धम् ✓ Neural narration ready. Tell Claude!"
