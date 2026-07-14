#!/bin/bash
# संस्कृता Video #1 — narration generator (female voice, macOS built-in)
# Run:  bash make_voice.sh     — creates voice files in this folder.
set -e
cd "$(dirname "$0")"

# pick best available female voice: Lekha (Hindi) > Veena (Indian English) > Samantha
if say -v '?' | grep -q "^Lekha"; then
    VOICE="Lekha"; LANG_USED="hi"
elif say -v '?' | grep -q "^Veena"; then
    VOICE="Veena"; LANG_USED="en"
else
    VOICE="Samantha"; LANG_USED="en"
fi
echo "voice: $VOICE ($LANG_USED)" > voice_used.txt

narrate() {  # narrate <n> <hindi text> <english text>
    local n="$1"; local hi="$2"; local en="$3"
    local text="$en"
    [ "$LANG_USED" = "hi" ] && text="$hi"
    say -v "$VOICE" -o "line$n.aiff" "$text"
    echo "✓ line$n.aiff"
}

narrate 1 "क्या आप जानते हैं? यह प्रोग्रामिंग भाषा पूरी तरह संस्कृत में लिखी जाती है।" \
          "Did you know? This programming language is written entirely in Sanskrit."

narrate 2 "इसका नाम है संस्कृता। देखिए — वद का अर्थ है बोलो।" \
          "It is called Sanskrita. Watch — vada means: speak."

narrate 3 "चर, लूप, फलन — सब पाणिनि के व्याकरण के साथ।" \
          "Variables, loops, functions — all built on Panini's grammar."

narrate 4 "शून्य दशमलव एक, जोड़ें शून्य दशमलव दो — उत्तर: ठीक शून्य दशमलव तीन। पाइथन यह गलत करता है!" \
          "Zero point one plus zero point two — exactly zero point three. Python gets this wrong!"

narrate 5 "और यह देखिए — इसने अभी भगवद्गीता के पहले श्लोक का छन्द पहचाना: अनुष्टुभ्!" \
          "And watch this — it just recognized the meter of the Bhagavad Gita's first verse: Anushtubh!"

narrate 6 "संस्कृता। ओपन सोर्स। लिंक नीचे है। जयतु संस्कृतम्!" \
          "Sanskrita. Open source. Link below. Jayatu Sanskritam!"

echo ""
echo "सिद्धम् ✓ All narration files created in: $(pwd)"
echo "Now tell Claude the files are ready."
